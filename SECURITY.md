# Security Policy

This document describes how to report security vulnerabilities and our approach to handling them for archinstall-rs, a TUI-based Arch Linux installer written in Rust. This project can perform destructive disk operations; please read the safe testing guidance below before reproducing issues.

## Supported Versions

The project is pre-1.0 and under active development. We generally provide security fixes for the latest code on `main` and the most recent tagged release line.

| Version/Branch | Supported |
| -------------- | --------- |
| `main` (unreleased) | ✅ best-effort |
| 0.2.0 | ✅ |
| < 0.1.0 | :x: |

Notes:
- Security fixes may be released directly on `main` and then backported to the latest release line when feasible.
- Older releases will not receive security updates unless there is an exceptional reason.

## Reporting a Vulnerability

Please use responsible disclosure and avoid filing public issues for suspected security problems.

Preferred reporting channel:
1. Open a private report via GitHub Security Advisories for this repository (Security tab → Report a vulnerability).
2. If private reporting is unavailable, open a minimal public issue requesting a private contact channel and do not include sensitive details.

When reporting, include:
- Affected version/commit hash and how you installed or ran the tool
- Environment (Arch ISO build/date, UEFI/BIOS, VM/physical)
- Reproduction steps (see Safe Testing below); attach sanitized logs/output
- Expected impact and observed impact (data loss risk, privilege escalation, etc.)
- Any workarounds you found

We will not ask for secrets or passphrases. Please remove or redact sensitive information (disk identifiers if needed, encryption passphrases, MACs, IPs).

### Our Response Timeline (SLO)
- Acknowledgement: within 72 hours
- Triage and initial assessment: within 7 calendar days
- Fix or mitigation for critical/high severity: target within 30 days
- Fix or mitigation for medium/low severity: target within 90 days
- Coordinated disclosure: we will agree on an embargo timeline and publish advisories once a fix or mitigation is available

If a report is out of scope (see below) or not reproducible, we will communicate that outcome and, when possible, provide guidance.

## Scope and Impact

In scope (non-exhaustive):
- Data-destruction risks bypassing documented safety checks (e.g., writing to mounted devices, ignoring confirmations)
- Command injection or arbitrary code execution via TUI inputs, configuration files (TOML), or environment variables
- Insecure handling of disk encryption parameters (e.g., leaking passphrases to logs, storing secrets at rest)
- Privilege escalation in the installer context
- Logic flaws in dry-run mode that would perform unintended writes
- Path traversal or unsafe temporary file usage

Out of scope:
- Vulnerabilities in third-party dependencies not caused by our usage (please report upstream)
- Issues in the Arch Linux live ISO, kernel, firmware, or external tooling (e.g., `parted`, `cryptsetup`) unless our integration is insecure
- General UX problems that do not have a security impact

## Safe Testing and Reproduction

This installer modifies disks. Reproduce issues only in disposable environments.

Recommended setups:
- Virtual machine with a virtual disk (e.g., QEMU/VirtualBox/VMware)
- Loopback test image on a development system

Example: create a 4GiB loopback disk and attach it as a block device (Linux):

```bash
dd if=/dev/zero of=/tmp/archinstall-rs-test.img bs=1M count=4096 status=progress
LOOPDEV=$(losetup --show -f /tmp/archinstall-rs-test.img)
echo "Loop device: $LOOPDEV"  # e.g., /dev/loop7

# Run the installer and select $LOOPDEV as the target device
# Always start with a dry run to review the plan
./archinstall-rs --dry-run

# When finished testing, detach the loop device
losetup -d "$LOOPDEV"
```

Guidelines:
- Prefer `--dry-run` first to verify the partitioning/mkfs/cryptsetup plan
- Never point the tool at production disks when testing
- Sanitize logs before sharing; do not include secrets

## Data Handling and Privacy

- The tool may handle disk encryption passphrases during runtime to invoke `cryptsetup`. Do not share passphrases with maintainers. Configuration files should contain only password hashes where applicable.
- We strive to avoid logging sensitive data; if you find any sensitive value appearing in logs, please report it.

## Hardening Recommendations for Users

- Run from the official Arch Linux live environment when installing
- Verify binaries/checksums when releases are provided
- Use `--dry-run` to review the exact command plan before applying
- Ensure target disks are not mounted and that you have backups
- Prefer UEFI with an ESP when possible; use encryption for the root device if desired

## Key Areas of Attention (for researchers)

- Partitioning safety checks (mounted devices detection, wipe confirmations)
- Dry-run mode fidelity (ensuring no writes occur in dry-run)
- Btrfs, swap, and ESP creation logic (sizes, flags, alignment)
- LUKS setup (parameters, non-leakage of passphrases)
- TUI input validation and command execution boundaries
- TOML config parsing and serialization safety

## Coordinated Disclosure and Credit

We support coordinated disclosure. With your consent, we are happy to credit reporters in release notes/security advisories after a fix is available. If you prefer to remain anonymous, let us know in your report.

## Contact

Use GitHub Security Advisories for private reporting. If that is not available, open a minimal public issue requesting a private contact channel (without sensitive details). We will respond with next steps.
