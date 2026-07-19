# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | ✅        |

## Reporting a Vulnerability

If you discover a security vulnerability in this project template:

1. **Do NOT** open a public GitHub issue
2. Email the maintainer directly with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (optional)

**Response time**: We aim to respond within 48 hours and provide a fix within 7 days for critical issues.

## Scope

This security policy covers:
- Hooks that execute shell commands
- Secret scanning patterns
- Template files that end up in user projects

Out of scope:
- User's own project code generated from this template
- Third-party integrations (Telegram, MCP servers)

## Security Best Practices (for template users)

- Never commit `.env` files
- Rotate API keys every 90 days
- Use the built-in secret scanner hook
- Run `npm audit` / `pip audit` regularly
- Review WebFetch permissions to prevent data exfiltration
