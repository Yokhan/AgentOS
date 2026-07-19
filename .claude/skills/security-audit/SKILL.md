---
name: security-audit
description: "Trigger when user asks for security review, vulnerability check, or pentest. Also trigger on keywords: security, audit, vulnerability, CVE, injection, XSS."
---

# Security Audit Skill

Full security audit of the project. Inspired by Shannon pentester (96% XBOW success).

## Process

### Phase 1: Dependency Scan
1. Detect package manager from project files
2. Run appropriate audit:
   - `npm audit --audit-level=high`
   - `pip audit`
   - `cargo audit`
   - `govulncheck ./...`
3. List all HIGH and CRITICAL vulnerabilities
4. Check if automated fix available

### Phase 2: Code Pattern Scan
Launch security-auditor agent to check:
- SQL injection (raw queries with string concatenation)
- XSS (unescaped user input in templates)
- Command injection (user input in exec/system)
- Path traversal (user input in file operations)
- Hardcoded secrets (API keys, passwords, tokens)
- Insecure crypto (md5, sha1 for passwords)

### Phase 3: Configuration Check
- Debug mode in production configs?
- CORS set to `*`?
- Rate limiting configured on auth/sensitive endpoints?
- Security headers: CSP, HSTS, X-Frame-Options (DENY), X-Content-Type-Options (nosniff)?
- CSRF tokens on all state-changing requests?
- SameSite attribute on session/auth cookies?
- RLS enabled on all database tables?
- HTTPS enforced?

### Phase 4: Container Security (if Dockerfile exists)
- Non-root USER instruction?
- Minimal base image (alpine/distroless, no `latest` tag)?
- No secrets in ENV or COPY layers?
- Multi-stage build to minimize final image?

### Phase 5: Report
1. Generate report with severity levels (Critical/High/Medium/Low)
2. Save to `brain/03-knowledge/security/audit-YYYY-MM-DD.md`
3. Create tasks for Critical/High findings in `tasks/current.md`
4. Add any new patterns to `tasks/lessons.md`

## Output Format
```
## Security Audit — [date]
### Critical: [count]
### High: [count]
### Medium: [count]
### Low: [count]
### Top 3 Priority Fixes:
1. ...
2. ...
3. ...
```
