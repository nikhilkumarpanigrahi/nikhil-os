# Security Policy

## Reporting a vulnerability

NIKHIL//OS is an interactive portfolio, but it is also a demonstration of
systems security (sandboxed AI tool execution, permission boundaries, IPC).
We take vulnerabilities seriously.

Please **do not open a public issue** for security problems. Report privately:

- GitHub private vulnerability reporting: use the **Security** tab on the
  repository → **Report a vulnerability**.
- Or email the maintainer at the address listed on the profile page.

You should receive an acknowledgment within 72 hours.

## Scope

In scope:

- Sandbox escape from the simulated OS into the host page/application.
- AI tool execution bypassing permission validation or schema validation.
- Prompt injection that yields privileged action.
- Path traversal or privilege escalation within the virtual filesystem.
- Data exfiltration of the embedded profile data or visitor session data.

Out of scope (by design):

- The simulated OS is a browser application; it cannot provide real process
  or memory isolation. Treat the Web Edition as a display environment.

## Security model

Every AI action passes through:

```text
Intent → Tool → Schema validation → Permission validation → OS service → Audit
```

The model never receives arbitrary shell, filesystem, database, or
JavaScript access. For the full threat model and test list, see
[docs/04-ML-AI-SPEC.md](docs/04-ML-AI-SPEC.md) §11 and
[docs/02-ARCHITECTURE.md](docs/02-ARCHITECTURE.md) §22.
