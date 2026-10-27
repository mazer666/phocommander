# Acceptance of the milestone M3 — Windows, Linux and BSD

**State:** 26.10.2026 · **Examiner:** Lead instance · **Result:** accepted with two open points

---

## 1. What this milestone covers

M3 brings the same application to the other desktop platforms, out of the same code base: the
**Windows port** with the manifest for the long paths and a plain-language message on the path
length, the **Linux port** with the Wayland path and the fallback to X11, the **FreeBSD and NetBSD
builds** in the CI matrix and the **packaging** for the three platforms.

## 2. The measured state

| What | How it was measured | Result |
|---|---|---|
| The tests of the workspace | `cargo test --workspace --locked` | **1277 green, 0 red, 0 skipped** |
| The formatting | `cargo fmt --all -- --check` | **without a deviation** |
| The warnings of the linter | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **without a warning** |
| The licences | `cargo deny check` | **advisories, bans, licences, sources: ok** |
| The boundaries of the crates | `scripts/check-crate-boundaries.sh` | **13 dependencies, 0 violations** |
| No telemetry | `scripts/check-no-telemetry.sh` | **0 violations** |
| The consistency gates | `scripts/check-*.py` | **all PASS** |
| The gates prove themselves | `scripts/check-gates-negative.sh` | **38 gates**, each turns red and green again |

## 3. What this milestone found

**A finding that touches the security architecture.** The hardened extraction used the crate
`cap-std`, and **the crate needs a nightly compiler**: it demands features of the compiler that
the stable compiler does not allow — in *every* version. The old versions do not help, they pull a
`rustix` that uses an attribute the compiler removed long ago.

The decision ADR-0004 named `rustix` as an alternative already, and it offers the same access —
the descriptor of the directory, the relative opening, the refusing of the symbolic links —
**without a single line of `unsafe`** in this repository. The change is done, and **the negative
tests of the hardening rules H-02 to H-06 hold against it**.

**Two errors in the chain of the messages.** The catalogue key of an error and the error code are
two different things: the code carries a dot (`path.too_long`), because it names a case in a table
of cases, the key carries a hyphen (`path-too-long`), because the format of the catalogue allows no
dot. And a key that carried two meanings was refused by the catalogue — and with it the **whole
catalogue was unusable**, on which almost all i18n tests failed.

**A finding that touches the interface.** The job list had **no source**: the interface knew the
trait `JobSource`, but the desktop app never implemented it. The list stayed empty, and the user
never saw that a job had failed.

## 4. The two open points

| Point | Reason | When |
|---|---|---|
| **The first Windows build** | The manifest `longPathAware` and the embedded resource file are proven with `rust-lld`, not with the linker of Windows. `link.exe` takes the same `.res` file, but the first build of the owner is the proof | at the first Windows build |
| **The manual test protocol for the platform-specific behaviour** | The sandbox has a Linux system and no screen, no Windows and no BSD. The protocol needs these machines | with the machines of the owner |

The remaining points of the roadmap are fulfilled: the manifest, the message of the path length,
the decision of Wayland against X11, the builds of FreeBSD and NetBSD and the packaging.

## 5. The decisions that this milestone made

| Decision | Why |
|---|---|
| **The hardened extraction uses `rustix` instead of `cap-std`** | `cap-std` needs a nightly compiler. A nightly compiler is no option for a project that writes its security argument in safe Rust. ADR-0004 named `rustix` already |
| **The extension of a file does not decide the format of a container** | The magic bytes decide. A file that is called `.zip` and is no ZIP shall be recognised as what it is |
| **A script is pushed over the API with the mode, not over the connector** | The connector does not carry the mode, and a gate script without the executable bit does not run |

## 6. What comes next

1. The **property test of the rename (FR-33)** — it is written now and checked on its effect: with a
   real randomness instead of the derived value, both properties fall.
2. The **benchmarks NFR-02 and NFR-03** in the release profile.
3. The **acceptance of M1**, as soon as the manual protocol has run on a macOS machine.

---

*The acceptance holds for the state of the commit with which the protocols were written. Every
later change of the sources makes the measurement invalid.*
