# Security Policy

Phocommander is a local file manager. It has no server, no account, no telemetry and no
analytics. That shapes this policy: there is no database of user data to breach, and the
security work of the project is concentrated in a small number of places — archive extraction,
file operations against a hostile filesystem, and the network sources.

The complete threat model — trust boundary, attack surfaces, the binding hardening rules, the
accepted residual risks, the permission audit and the audit protocol — lives in
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md). This document is the
front door: what is guaranteed, what to report, and how.

The binding source for the security requirements of the project are the sections
**9.2 The hardening rules** and **9.4 Secure deletion** of
[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md), together with the sections *9.5 Privacy* and
*4.8 Remote sources*. Where this policy and the specification disagree, the
specification wins — and the disagreement is a defect in this document.

---

## Supported versions

Phocommander is under active development and has **no released version yet**. The security
policy therefore covers the current `main` branch.

| Version | Supported |
|---|---|
| `main` (development) | Yes |
| Any release, once published | Until a newer release exists; security fixes are applied to the newest release and backported to the previous one for as long as it is supported |

Because the project ships free of charge and without a paid support tier, there is no
"extended support" window and no paid security response. The response commitment below is the
best-effort commitment of a volunteer project and is stated as such.

---

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Use GitHub's private vulnerability reporting:

1. Open the **Security** tab of the repository at <https://github.com/mazer666/phocommander>.
2. Choose **Report a vulnerability**.
3. Fill in the description.

If private reporting is not yet enabled for the repository, open a regular issue that contains
**only** a short note that a security report should be sent privately, together with a way to
reach you — without any technical detail. This keeps the report out of the public record
while the private channel is set up.

### What a good report contains

- The Phocommander version and the platform (OS, architecture).
- The **exact steps** to reproduce, ideally as a command sequence.
- The **proof**: a crafted archive, a malicious server response, a file tree, or a command
  output. A proof is worth a thousand words — a `.zip` that escapes its destination directory
  is the ideal artefact.
- The **impact**: what an attacker achieves (arbitrary file write, credential disclosure,
  privilege escalation, data loss, denial of service).
- Whether the issue is already public.

### What to expect

| Step | Target |
|---|---|
| Acknowledgement of the report | 7 days |
| Assessment and severity rating | 14 days |
| Fix and released version for a critical issue | 30 days from the assessment |
| Fix and released version for a high issue | 90 days from the assessment |
| Public disclosure | Coordinated with the reporter, after the fix is released. The reporter may publish earlier; that is their right. |

These are targets of a volunteer project, not a contractual service level. If a target is
missed, the reason is stated in the issue.

### What is *not* a vulnerability

Phocommander runs with the privileges of the user who started it. The following are **by
design**, not bugs:

- A file that the user can read is readable by Phocommander.
- A remote server sees the requests the user makes to it, including the directory listings
  the user asked for.
- `Full Disk Access` on macOS and all-files access on Android are granted by the user and
  give the app the access the user intends.
- The *Clear* sanitization level of "secure delete" cannot guarantee that data is gone on
  flash media (wear levelling, over-provisioning). The limitation is stated in the user
  interface — see §5 of the threat model.
- The residual race between a path check and the following access, documented as an accepted
  residual risk in §4 of the threat model.

Reports about these behaviours are welcome as documentation feedback, but they are not
security incidents.

### Safe harbour

Good-faith security research on Phocommander is welcome. You may test it on systems you own
or are authorised to test, you may build proof-of-concept archives and servers, and you may
publish your findings after a reasonable disclosure period. The project will not pursue legal
action against researchers who follow this policy and do not compromise users, other
researchers, or the availability of the project.

---

## Hardening guarantees

These are the guarantees the project makes about its own code. Each of them is backed by a
negative test — a test that performs the attack and proves the defence holds. A positive test
proves nothing about a defence.

| # | Guarantee | Where it lives |
|---|---|---|
| H1 | The `zip` dependency is pinned to **9.0.5 or newer**, which fixes `RUSTSEC-2025-0168` (CVE-2025-29787), `RUSTSEC-2026-0367`, `RUSTSEC-2026-0368` and `RUSTSEC-2026-0369`. | `Cargo.toml`, `deny.toml` |
| H2 | The application **never calls `extract()`** from `zip`, `tar` or any other archive crate. It iterates over the entries and writes them itself. | `crates/phocommander-archive` |
| H3 | **Every target path is canonicalised and checked against the destination directory** before anything is written. A path that leaves the destination is rejected, not repaired. | `crates/phocommander-archive` |
| H4 | **Symlinks are never created from archive material.** A symlink entry is reported and skipped. | `crates/phocommander-archive` |
| H5 | **SUID, SGID and sticky bits are never carried over.** The mode is masked by the application. | `crates/phocommander-archive` |
| H6 | **Decompression bombs are bounded** by a limit on total extracted bytes, on the entry count and on the size of a single entry. | `crates/phocommander-archive` |
| H7 | **SMB1 and NTLMv1 are not implemented.** Signing and SMB3 transport encryption are requested; a server that offers only the obsolete versions is rejected. | `crates/phocommander-remote` |
| H8 | **HTTPS certificates are verified strictly, with no opt-out.** A self-signed certificate is only accepted as a deliberately pinned, PIN-protected exception for exactly one host. | `crates/phocommander-remote` |
| H9 | **Every network operation has a time limit.** A server that never answers produces a cancellation, not a stuck job. | `crates/phocommander-remote` |
| H10 | **Credentials live in the OS keychain**, never in the configuration file. | `crates/phocommander-remote`, `platform/phocommander-platform` |
| H11 | **Symlinks are copied as symlinks and unlinked on delete** — never dereferenced, never deleted recursively. | `crates/phocommander-ops` |
| H12 | **Every parser validates its header before allocating.** | `crates/phocommander-viewer`, `crates/phocommander-archive` |
| H13 | **Long paths are handled, not truncated**; `longPathAware` is set in the Windows manifest. | `apps/phocommander-desktop` |
| H14 | **Permissions are requested, never taken.** Nothing is escalated silently. | `platform/phocommander-platform` |
| H15 | **The response size is bounded.** The client reads within a fixed budget and aborts. | `crates/phocommander-remote` |
| H16 | **Remote names are normalised and validated** before they are displayed or turned into paths. | `crates/phocommander-remote` |
| H17 | **Secure deletion follows NIST SP 800-88 Rev. 2** (*Clear*), and the flash-media limitation is stated in the user interface. | `crates/phocommander-ops` |

---

## Dependency security

- Every dependency is checked with **`cargo-deny` 0.20.2** (licences and advisories) and
  **`cargo-audit` 0.22.2** (known vulnerabilities) on every change. Both tools are pinned:
  the same configuration file is read differently by different tool versions.
- The licence allow-list contains **no GPL and no AGPL**, so that the `MIT OR Apache-2.0`
  promise of the project is never broken by a dependency.
- The `zip` crate is pinned to `9.0.5`, the newest published release, which is above every
  RustSec advisory floor for the `zip` crate.
- A gate that was never seen red has not been proven. The CI proves its own effectiveness by
  deliberately breaking a rule and showing the gate fail.

---

## Why we need no telemetry

A security policy for a program that collects nothing is short, and that is the point.

Phocommander is a **local** file manager. Its quality does not depend on knowing anything
about its users: not on how often a feature is used, not on which paths are popular, not on
how many crashes happen. The features that would need telemetry are not features of this
product — the product is judged on its speed, its keyboard handling and its correctness.

Three technical consequences follow, and all three are checkable:

1. **No dependency that phones home.** The licence and dependency policy (`deny.toml`,
   `cargo deny check`) is the gate. A telemetry or analytics crate cannot enter the dependency
   tree unnoticed.
2. **No connection at start-up.** A test asserts that starting the process opens no socket.
   There is no update check, no licence server, no crash ping, no "anonymous statistics"
   endpoint — the only outbound connection is one the user started with a server the user
   named.
3. **No server of our own.** No backend, no account, no login, no data-processing point.
   Every local function works without registration.

The consequence for the user is simple: there is no server holding a profile of you, so
there is no profile to breach, no data to leak, and no consent banner to click away.

See [`PRIVACY.md`](PRIVACY.md) for the complete statement of what is stored, where, for how
long and how to remove it.

---

## Data protection in one paragraph

Phocommander collects nothing. There is no telemetry, no analytics, no usage profile, no
crash report upload and no update check. The only persistent data is the configuration the
user creates, and credentials go into the operating system's keychain. See
[`PRIVACY.md`](PRIVACY.md).

---

*Version 1.0, 24.10.2026.*
