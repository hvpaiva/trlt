# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly.

**Do not open a public issue.**

Report through either channel:

1. **GitHub Private Reporting** (preferred): use the [Report a vulnerability](https://github.com/hvpaiva/trlt/security/advisories/new) button on the Security tab.
2. **Email**: send details to [contact@hvpaiva.dev](mailto:contact@hvpaiva.dev).

Include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact

You should receive a response within 48 hours. We will coordinate with you before any public disclosure.

## Security Measures

This project uses the following automated tools:

| Tool | Purpose |
|------|---------|
| [Dependabot](https://docs.github.com/en/code-security/dependabot) | Alerts for vulnerable dependencies |
| [Secret scanning](https://docs.github.com/en/code-security/secret-scanning) | Prevents accidental credential leaks |
| [CodeQL](https://codeql.github.com/) | Static analysis for vulnerability detection |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | Dependency audit on every push |

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |
