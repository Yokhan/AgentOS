use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

enum AcpEnvelope {
    Json(Value),
    Stderr(String),
    StdoutParse(String),
    Closed,
}

pub struct AcpClient {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<AcpEnvelope>,
    next_id: u64,
    stderr_log: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct AcpInitStatus {
    pub protocol_version: i64,
    pub auth_methods: Vec<Value>,
    pub agent_info: Value,
    pub agent_capabilities: Value,
}

impl AcpClient {
    pub fn spawn(command: &str, args: &[String], cwd: &Path) -> Result<Self, String> {
        let mut cmd = Command::new(command);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);
        let mut child = cmd
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ACP agent '{}': {}", command, e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "ACP agent stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "ACP agent stdout unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "ACP agent stderr unavailable".to_string())?;

        let (tx, rx) = mpsc::channel();
        spawn_stdout_reader(stdout, tx.clone());
        spawn_stderr_reader(stderr, tx);

        Ok(Self {
            child,
            stdin,
            rx,
            next_id: 1,
            stderr_log: Vec::new(),
        })
    }

    pub fn initialize(&mut self) -> Result<AcpInitStatus, String> {
        let result = self.request(
            "initialize",
            json!({
                "protocolVersion": 1,
                "clientCapabilities": {
                    "fs": {
                        "readTextFile": false,
                        "writeTextFile": false
                    },
                    "terminal": false
                },
                "clientInfo": {
                    "name": "agent-os",
                    "title": "Agent OS",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
            Duration::from_secs(15),
            &mut |_notification| {},
        )?;
        Ok(AcpInitStatus {
            protocol_version: result
                .get("protocolVersion")
                .and_then(|v| v.as_i64())
                .unwrap_or(1),
            auth_methods: result
                .get("authMethods")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
            agent_info: result.get("agentInfo").cloned().unwrap_or(Value::Null),
            agent_capabilities: result
                .get("agentCapabilities")
                .cloned()
                .unwrap_or(Value::Null),
        })
    }

    pub fn authenticate(&mut self, method_id: &str) -> Result<Value, String> {
        self.request(
            "authenticate",
            json!({ "methodId": method_id }),
            Duration::from_secs(180),
            &mut |_notification| {},
        )
    }

    pub fn new_session(&mut self, cwd: &Path) -> Result<Value, String> {
        self.request(
            "session/new",
            json!({
                "cwd": cwd.to_string_lossy().to_string(),
                "mcpServers": []
            }),
            Duration::from_secs(15),
            &mut |_notification| {},
        )
    }

    pub fn prompt(&mut self, session_id: &str, prompt: &str) -> Result<String, String> {
        let mut chunks: Vec<String> = Vec::new();
        let _ = self.request(
            "session/prompt",
            json!({
                "sessionId": session_id,
                "prompt": [
                    {
                        "type": "text",
                        "text": prompt
                    }
                ]
            }),
            Duration::from_secs(300),
            &mut |notification| {
                if notification.get("method").and_then(|v| v.as_str()) != Some("session/update") {
                    return;
                }
                let update = notification
                    .get("params")
                    .and_then(|v| v.get("update"))
                    .cloned()
                    .unwrap_or(Value::Null);
                if update.get("sessionUpdate").and_then(|v| v.as_str())
                    != Some("agent_message_chunk")
                {
                    return;
                }
                if let Some(text) = update
                    .get("content")
                    .and_then(|v| v.get("text"))
                    .and_then(|v| v.as_str())
                {
                    chunks.push(text.to_string());
                }
            },
        )?;
        let joined = chunks.join("");
        if joined.trim().is_empty() {
            Err(self.decorate_error("ACP agent returned no message content"))
        } else {
            Ok(joined)
        }
    }

    fn request<F>(
        &mut self,
        method: &str,
        params: Value,
        timeout: Duration,
        on_notification: &mut F,
    ) -> Result<Value, String>
    where
        F: FnMut(&Value),
    {
        let id = self.next_id;
        self.next_id += 1;
        self.send_json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }))?;

        let started = Instant::now();
        loop {
            let remaining = timeout
                .checked_sub(started.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if remaining.is_zero() {
                return Err(self.decorate_error(&format!(
                    "ACP request '{}' timed out after {:?}",
                    method, timeout
                )));
            }
            match self
                .rx
                .recv_timeout(std::cmp::min(remaining, Duration::from_millis(250)))
            {
                Ok(AcpEnvelope::Json(message)) => {
                    if message.get("method").is_some() {
                        if message.get("id").is_some() {
                            self.respond_method_not_supported(&message)?;
                        } else {
                            on_notification(&message);
                        }
                        continue;
                    }
                    let matches = message
                        .get("id")
                        .and_then(|v| v.as_u64())
                        .map(|value| value == id)
                        .unwrap_or(false);
                    if !matches {
                        continue;
                    }
                    if let Some(error) = message.get("error") {
                        let msg = error
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown ACP error");
                        let code = error
                            .get("code")
                            .and_then(|v| v.as_i64())
                            .unwrap_or_default();
                        return Err(self.decorate_error(&format!("{} (code {})", msg, code)));
                    }
                    return Ok(message.get("result").cloned().unwrap_or(Value::Null));
                }
                Ok(AcpEnvelope::Stderr(line)) => {
                    if !line.trim().is_empty() {
                        self.stderr_log.push(line);
                    }
                }
                Ok(AcpEnvelope::StdoutParse(err)) => {
                    self.stderr_log.push(err);
                }
                Ok(AcpEnvelope::Closed) => {
                    return Err(self.decorate_error("ACP agent closed unexpectedly"));
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(self.decorate_error("ACP agent pipe disconnected"));
                }
            }
        }
    }

    fn send_json(&mut self, value: &Value) -> Result<(), String> {
        let line = serde_json::to_string(value).map_err(|e| e.to_string())?;
        self.stdin
            .write_all(line.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|e| self.decorate_error(&format!("ACP write failed: {}", e)))
    }

    fn respond_method_not_supported(&mut self, message: &Value) -> Result<(), String> {
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        self.send_json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": "Method not supported by Agent OS ACP client"
            }
        }))
    }

    fn decorate_error(&self, message: &str) -> String {
        if self.stderr_log.is_empty() {
            message.to_string()
        } else {
            format!(
                "{} | stderr: {}",
                message,
                self.stderr_log
                    .iter()
                    .rev()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        }
    }
}

impl Drop for AcpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_stdout_reader(stdout: ChildStdout, tx: Sender<AcpEnvelope>) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<Value>(trimmed) {
                        Ok(value) => {
                            let _ = tx.send(AcpEnvelope::Json(value));
                        }
                        Err(e) => {
                            let _ = tx.send(AcpEnvelope::StdoutParse(format!(
                                "stdout parse error: {} | line={}",
                                e, trimmed
                            )));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AcpEnvelope::StdoutParse(format!(
                        "stdout read error: {}",
                        e
                    )));
                    break;
                }
            }
        }
        let _ = tx.send(AcpEnvelope::Closed);
    });
}

fn spawn_stderr_reader(stderr: ChildStderr, tx: Sender<AcpEnvelope>) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let _ = tx.send(AcpEnvelope::Stderr(line));
                }
                Err(_) => break,
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn installed_codex_acp_path() -> Option<PathBuf> {
        std::env::var("LOCALAPPDATA").ok().and_then(|base| {
            let path = PathBuf::from(base)
                .join("AgentOS")
                .join("tools")
                .join("codex-acp")
                .join("v0.4.2")
                .join(if cfg!(target_os = "windows") {
                    "codex-acp.exe"
                } else {
                    "codex-acp"
                });
            path.is_file().then_some(path)
        })
    }

    #[test]
    fn smoke_codex_acp_initialize_and_prompt_if_installed() {
        let Some(command) = installed_codex_acp_path() else {
            return;
        };
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let mut client = AcpClient::spawn(command.to_string_lossy().as_ref(), &[], &cwd)
            .expect("spawn codex-acp");
        let init = client.initialize().expect("initialize");
        assert!(init.protocol_version >= 1);
        let session = client.new_session(&cwd).expect("session/new");
        let session_id = session
            .get("sessionId")
            .or_else(|| session.get("id"))
            .and_then(|v| v.as_str())
            .expect("sessionId");
        let reply = client
            .prompt(session_id, "Reply with exactly OK.")
            .expect("session/prompt");
        assert!(!reply.trim().is_empty());
    }
}
