//! Deploy & Server commands: DEPLOY_STATIC, VERIFY, ROLLBACK, SERVER_EXEC, STATUS, NGINX, SSL, DNS.
//! All SSH operations use the yokhan-vps alias from ~/.ssh/config.

use super::claude_runner::{find_bash, silent_cmd};
use crate::state::AppState;

const DANGEROUS_PATTERNS: &[&str] = &[
    "|",
    ";",
    "&&",
    "||",
    "`",
    "$(",
    "${",
    "rm -rf",
    "mkfs",
    "dd if",
    "shutdown",
    "reboot",
    "init 0",
    "init 6",
    "> /dev",
    "chmod 777",
];

fn ssh_exec(host: &str, command: &str) -> Result<String, String> {
    let output = silent_cmd("ssh")
        .args([
            "-o",
            "ConnectTimeout=10",
            "-o",
            "StrictHostKeyChecking=accept-new",
            host,
            command,
        ])
        .output()
        .map_err(|e| format!("SSH error: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(if stdout.is_empty() { stderr } else { stdout })
    } else {
        Err(format!(
            "Exit {}: {}",
            output.status.code().unwrap_or(-1),
            if stderr.is_empty() { &stdout } else { &stderr }
        ))
    }
}

fn scp_upload(host: &str, local: &str, remote: &str) -> Result<String, String> {
    let output = silent_cmd("scp")
        .args([
            "-r",
            "-o",
            "ConnectTimeout=10",
            local,
            &format!("{}:{}", host, remote),
        ])
        .output()
        .map_err(|e| format!("SCP error: {}", e))?;

    if output.status.success() {
        Ok("Upload complete".to_string())
    } else {
        Err(format!(
            "SCP failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

/// DEPLOY_STATIC: SCP files to VPS with backup
pub fn deploy_static(
    state: &AppState,
    project: &str,
    target_path: &str,
    source: &str,
) -> Option<String> {
    let project_dir = state.validate_project(project).ok()?;
    let src = if source.is_empty() {
        project_dir.to_string_lossy().to_string()
    } else {
        source.to_string()
    };
    let host = "yokhan-vps";

    // Backup current
    let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let backup = format!("{}.bak-{}", target_path, ts);
    let _ = ssh_exec(
        host,
        &format!("cp -r {} {} 2>/dev/null || true", target_path, backup),
    );
    crate::log_info!("[deploy] backup {} -> {}", target_path, backup);

    // Upload
    match scp_upload(host, &src, target_path) {
        Ok(_) => {
            crate::log_info!("[deploy] uploaded {} -> {}:{}", src, host, target_path);
            Some(format!(
                "**Deployed** {} -> {}:{}\nBackup: {}",
                project, host, target_path, backup
            ))
        }
        Err(e) => Some(format!("**Deploy failed:** {}", e)),
    }
}

/// DEPLOY_VERIFY: HTTP health check + content validation
pub fn deploy_verify(url: &str, expected: &str) -> Option<String> {
    let start = std::time::Instant::now();
    let output = silent_cmd("curl")
        .args([
            "-sS",
            "-o",
            "-",
            "-w",
            "\n%{http_code}\n%{time_total}",
            "--max-time",
            "10",
            url,
        ])
        .output()
        .ok()?;

    let full = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = full.trim().rsplitn(3, '\n').collect();
    let _time_s = lines
        .first()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let code = lines
        .get(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body = lines.get(2).unwrap_or(&"");
    let elapsed = start.elapsed().as_millis();

    let mut checks = Vec::new();
    checks.push(if code == 200 {
        format!("[ok] HTTP {}", code)
    } else {
        format!("[x] HTTP {}", code)
    });
    checks.push(format!("[ok] Response time: {}ms", elapsed));
    if !expected.is_empty() {
        if body.contains(expected) {
            checks.push(format!("[ok] Contains: \"{}\"", expected));
        } else {
            checks.push(format!("[x] Missing: \"{}\"", expected));
        }
    }
    Some(format!("**Verify {}:**\n{}", url, checks.join("\n")))
}

/// DEPLOY_ROLLBACK: restore from backup
pub fn deploy_rollback(target_path: &str) -> Option<String> {
    let host = "yokhan-vps";
    // Find most recent backup
    let find_cmd = format!("ls -td {}.bak-* 2>/dev/null | head -1", target_path);
    let backup = ssh_exec(host, &find_cmd).ok()?;
    if backup.trim().is_empty() {
        return Some("**Rollback failed:** no backup found".to_string());
    }

    let cmds = format!(
        "mv {} {}.rollback 2>/dev/null; mv {} {}",
        target_path,
        target_path,
        backup.trim(),
        target_path
    );
    match ssh_exec(host, &cmds) {
        Ok(_) => Some(format!(
            "**Rolled back** {} from {}",
            target_path,
            backup.trim()
        )),
        Err(e) => Some(format!("**Rollback failed:** {}", e)),
    }
}

/// SERVER_EXEC: run command on VPS (with safety blocklist)
pub fn server_exec(host: &str, command: &str) -> Option<String> {
    let lower = command.to_lowercase();
    for pattern in DANGEROUS_PATTERNS {
        if lower.contains(pattern) {
            crate::log_warn!("[deploy] SERVER_EXEC blocked: contains '{}'", pattern);
            return Some(format!("**Blocked:** command contains unsafe pattern: `{}`\nUse simple commands without pipes, semicolons, or shell operators.", pattern));
        }
    }
    match ssh_exec(host, command) {
        Ok(out) => Some(format!(
            "**{}$ {}**\n```\n{}\n```",
            host,
            command,
            out.chars().take(2000).collect::<String>()
        )),
        Err(e) => Some(format!("**Error:** {}", e)),
    }
}

/// SERVER_STATUS: resource summary
pub fn server_status(host: &str) -> Option<String> {
    let cmds = "echo '=== UPTIME ==='; uptime; echo '=== DISK ==='; df -h /; echo '=== MEMORY ==='; free -m; echo '=== FAILED ==='; systemctl --failed 2>/dev/null | head -5";
    match ssh_exec(host, cmds) {
        Ok(out) => Some(format!(
            "**Server {}:**\n```\n{}\n```",
            host,
            out.chars().take(2000).collect::<String>()
        )),
        Err(e) => Some(format!("**Error:** {}", e)),
    }
}

/// NGINX_VALIDATE: test nginx config
pub fn nginx_validate(host: &str) -> Option<String> {
    match ssh_exec(host, "nginx -t 2>&1") {
        Ok(out) => Some(format!("**Nginx {}:**\n{}", host, out)),
        Err(e) => Some(format!("**Nginx error:** {}", e)),
    }
}

fn sanitize_domain(d: &str) -> Option<&str> {
    if d.chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
        && d.contains('.')
    {
        Some(d)
    } else {
        None
    }
}

/// SSL_MONITOR: check cert expiry for domains
pub fn ssl_monitor(domains: &[&str]) -> Option<String> {
    let mut results = Vec::new();
    for domain in domains {
        let domain = match sanitize_domain(domain) {
            Some(d) => d,
            None => {
                results.push(format!("  {} -> INVALID DOMAIN", domain));
                continue;
            }
        };
        let output = silent_cmd(&find_bash())
            .args(["-c", &format!("echo | openssl s_client -servername '{}' -connect '{}':443 2>/dev/null | openssl x509 -noout -dates 2>/dev/null", domain, domain)])
            .output()
            .ok();
        let text = output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let expiry = text
            .lines()
            .find(|l| l.starts_with("notAfter="))
            .unwrap_or("unknown");
        results.push(format!("  {} -> {}", domain, expiry));
    }
    Some(format!("**SSL Certificates:**\n{}", results.join("\n")))
}

/// DNS_VERIFY: check DNS records
pub fn dns_verify(domain: &str) -> Option<String> {
    let domain = sanitize_domain(domain)?;
    let output = silent_cmd(&find_bash())
        .args(["-c", &format!(
            "echo '=== A ==='; dig +short '{}' A; echo '=== AAAA ==='; dig +short '{}' AAAA; echo '=== CNAME ==='; dig +short '{}' CNAME; echo '=== MX ==='; dig +short '{}' MX",
            domain, domain, domain, domain
        )])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(format!("**DNS {}:**\n```\n{}\n```", domain, text))
}
