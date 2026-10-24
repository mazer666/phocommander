# Übergabe — M0: Cargo-Workspace, Crate-Grenzen, öffentliche API, ADRs

**Rolle:** Rust Architect · **Projekt:** Phocommander · **Repository:** `/workspace/phocommander` (Branch `main`)
**Datum:** 24.10.2026 · **Bindende Grundlage:** `docs/REQUIREMENTS.md` (Fassung 1.1) — Abschnitte *Crate-Struktur*, *Benutzerschnittstelle und Plattformanpassung*, *Konfigurationsdatei*, *Projektplan*
**Sprache:** dieses Dokument auf Deutsch, alle Repository-Inhalte auf Englisch

> **Hinweis zu `deliverable.md`:** Vor Beginn meiner Arbeit enthielt `deliverable.md` die Übergabe der QA-Rolle. Sie ist als `deliverable-qa-test-engineer.md` unverändert erhalten und wird von dort nicht entfernt. Diese Datei ist jetzt die Übergabe der Architekten-Rolle.

---

## 0. Kurzfassung

Der Workspace steht: 14 Kisten, eine gemeinsame Versionskatalogeinstellung, vier Gate-Skripte und vier Architecture Decision Records. Alle Gates sind grün und mit echter Ausgabe belegt; das Grenzgate hat einen Negativtest, der beweist, dass es Zähne hat.

Drei Dinge muss der Owner entscheiden, bevor M1 Code schreibt (Abschnitt 10):

1. **Der Widerspruch in der Flachbau-Regel.** Die Spezifikation verbietet *jede* Abhängigkeit zwischen zwei Kisten unter `crates/` — damit auch die Abhängigkeit auf `phocommander-core`. Die referenzierte Schwester-Ecke Photocraft macht genau das (24 von 27 Kisten). `docs/architecture/README.md` rät den Beitragenden, gemeinsame Teile in `phocommander-core` zu legen — was das Gate verbietet. Ich habe die strikte Lesart implementiert (wie beauftragt) und den Widerspruch in ADR-0001 §4 als Entscheidungsvorlage dokumentiert.
2. **Der Maßstab des Grenzgates ist die strikte Lesart; die Schaffung eines gemeinsamen Vokabular-Kastens kostet M1.** Wer die strikte Lesart behält, muss gemeinsame Typen über den Kompositionswurzel-Kasten (`apps/*`) reichen.
3. **`unmaintained = "warn"` aus dem Auftrag ist mit cargo-deny 0.20.2 unmöglich.** Gemessen: die Konfiguration bricht mit `error[unexpected-value]` ab, *bevor* überhaupt geprüft wurde. Die Datei `deny.toml` steht jetzt auf `unmaintained = "none"` (Änderung des QA-Agenten, von mir nachgemessen und bestätigt).

**Nicht geliefert und nicht behauptet:** keine Implementierung von Produktfunktionen, keine Änderung an `docs/REQUIREMENTS.md`, kein Commit, kein Push, kein Tag, kein Release.

---

## 1. Dateiliste

Alle Dateien, die ich in dieser Sitzung angelegt habe. Zeilenzahlen sind mit `wc -l` gemessen.

### 1.1 Kisten (`crates/`, `platform/`, `apps/`, `xtask/`)

| Pfad | Zeilen | Zweck |
|---|---:|---|
| `crates/phocommander-core/Cargo.toml` | 25 | Manifest; einzige Abhängigkeit `thiserror` |
| `crates/phocommander-core/src/lib.rs` | 32 | Crate-Kopf (WAS/WARUM/GRENZE), Modulverweise, Re-Exports |
| `crates/phocommander-core/src/config.rs` | 98 | **Kanonische Grenzwerte** (`MAX_EXTRACTED_BYTES`, `MAX_ARCHIVE_ENTRIES`, `MAX_VIEWER_FILE_SIZE`, `MAX_VIEWER_TEXT_SIZE`) + 4 Tests |
| `crates/phocommander-core/src/error.rs` | 151 | Fehlertyp `Error` (`NoConfigDirectory`, `TooLarge`, `Io`) + 3 Tests |
| `crates/phocommander-config/Cargo.toml` | 27 | Manifest; `serde`, `toml`, `thiserror` |
| `crates/phocommander-config/src/lib.rs` | 37 | Crate-Kopf, Modulverweise, Re-Exports |
| `crates/phocommander-config/src/schema.rs` | 259 | Typisiertes Schema der TOML-Datei, Sektion für Sektion nach `docs/REQUIREMENTS.md` |
| `crates/phocommander-config/src/values.rs` | 386 | 10 Werte-Enums mit Vorgabe-Werten und Datei-Schreibweise + 3 Tests |
| `crates/phocommander-config/src/loader.rs` | 379 | `parse_toml`, `load_file`, `ConfigError`, Validierung + 6 Tests |
| `crates/phocommander-config/tests/shipped_defaults.rs` | 68 | Integrationstest: die ausgelieferte `config/defaults.toml` wird vom Leser akzeptiert |
| `crates/phocommander-keymap/Cargo.toml` | 26 | Manifest |
| `crates/phocommander-keymap/src/lib.rs` | 30 | Crate-Kopf, Re-Exports |
| `crates/phocommander-keymap/src/catalog.rs` | 318 | `Context`, `ActionId` (mit Prüfung der Schreibweise), `ActionIdError` + 7 Tests |
| `crates/phocommander-keymap/src/bindings.rs` | 334 | `Bindings`, `BindingConflict`, `BindingError` + 6 Tests |
| `crates/phocommander-keymap/src/keymap.rs` | 187 | `Keymap` über alle Kontexte + 4 Tests |
| `crates/phocommander-ops/Cargo.toml` | 22 | Manifest (M1: Auftragswarteschlange) |
| `crates/phocommander-ops/src/lib.rs` | 22 | Crate-Kopf mit Verantwortung, Grenze und Meilensteinzuordnung |
| `crates/phocommander-archive/Cargo.toml` | 24 | Manifest |
| `crates/phocommander-archive/src/lib.rs` | 27 | Crate-Kopf, Re-Exports |
| `crates/phocommander-archive/src/entry_name.rs` | 365 | **Gehärtete Namensauflösung** `resolve_entry_name` + 12 Tests (davon 6 Negativtests) |
| `crates/phocommander-viewer/Cargo.toml` | 21 | Manifest (M1: Betrachterregistry) |
| `crates/phocommander-viewer/src/lib.rs` | 22 | Crate-Kopf mit Verantwortung, Grenze und Meilensteinzuordnung |
| `crates/phocommander-remote/Cargo.toml` | 22 | Manifest (M6: Netzwerkquellen) |
| `crates/phocommander-remote/src/lib.rs` | 24 | Crate-Kopf mit Verantwortung, Grenze und Meilensteinzuordnung |
| `crates/phocommander-ui/Cargo.toml` | 23 | Manifest; `egui` 0.37 |
| `crates/phocommander-ui/src/lib.rs` | 27 | Crate-Kopf, Re-Exports |
| `crates/phocommander-ui/src/theme.rs` | 134 | `Theme` + `visuals()` (Übersetzung der Themewahl in egui-Visuals) + 4 Tests |
| `platform/phocommander-platform/Cargo.toml` | 25 | Manifest; `phocommander-core` (Pfad), `dirs` |
| `platform/phocommander-platform/src/lib.rs` | 23 | Crate-Kopf, Re-Exports |
| `platform/phocommander-platform/src/host.rs` | 180 | `Platform`, `host_platform`, `primary_modifier`, `config_directory` + 4 Tests |
| `apps/phocommander-desktop/Cargo.toml` | 33 | Manifest; `phocommander-config`, `phocommander-platform`, Dev-Abhängigkeit `phocommander-core` |
| `apps/phocommander-desktop/src/main.rs` | 34 | `fn main()`: meldet ehrlich den M0-Stand und ruft `std::process::exit(0)` |
| `apps/phocommander-desktop/tests/core_limits.rs` | 113 | Integrationstest: kanonische Grenzwerte gegen die ausgelieferte Konfigurationsdatei |
| `apps/phocommander-android/Cargo.toml` | 28 | Manifest, `crate-type = ["cdylib", "rlib"]` |
| `apps/phocommander-android/src/lib.rs` | 35 | Crate-Kopf mit der Android-Architekturnotiz (SAF-Modus, optionaler Vollzugriff) |
| `apps/phocommander-ios/Cargo.toml` | 28 | Manifest, `crate-type = ["cdylib", "rlib"]` |
| `apps/phocommander-ios/src/lib.rs` | 34 | Crate-Kopf mit der iPadOS-Architekturnotiz (Dokumenten-Picker, security-scoped Bookmarks) |
| `xtask/Cargo.toml` | 22 | Manifest, `publish = false` |
| `xtask/src/main.rs` | 62 | `cargo xtask check` führt `cargo check --workspace` aus und reicht den Exit-Code durch |

### 1.2 Gates, Werkzeuge, Dokumente

| Pfad | Zeilen | Zweck |
|---|---:|---|
| `scripts/check-crate-boundaries.sh` | 296 | **Das Grenzgate**: wertet alle Pfad-Abhängigkeiten aus, prüft die Richtung, Exit-Code 1 bei Verstoß |
| `scripts/README.md` | 75 | Was die beiden Skripte prüfen und wie man sie aufruft |
| `.cargo/config.toml` | 9 | Alias `cargo xtask` |
| `docs/adr/ADR-0001-cargo-workspace-and-module-boundaries.md` | 132 | Workspace, Ebenen, Grenzregel, Entscheidungsvorlage zum Widerspruch |
| `docs/adr/ADR-0002-filesystem-abstraction.md` | 132 | Die Dateisystemabstraktion, die aus der Plattform-Matrix folgt |
| `docs/adr/ADR-0003-error-hierarchy-and-error-codes.md` | 142 | Fehlerhierarchie, Fehlercodes, Protokollregel |
| `docs/adr/ADR-0004-archive-extraction-security.md` | 159 | Sicherheitsstrategie der Archivextraktion (ADR-0002 des Auftrags) |

### 1.3 Bestehende Dateien, die ich geprüft, aber nicht verändert habe

`Cargo.toml` (Workspace-Wurzel, Commit `61fa539` des Owners), `rust-toolchain.toml`, `deny.toml` — alle drei wurden gegen den Auftrag geprüft und sind inhaltlich deckungsgleich. `rust-toolchain.toml` enthält die `targets`-Zeile; ich habe sie **verifiziert** (Abschnitt 9).

### 1.4 Änderungen an Dateien anderer Agenten

| Pfad | Änderung | Grund |
|---|---|---|
| `tests/common/mod.rs` | eine Leerzeile im Doc-Kommentar entfernt | Der leere `//!`-Absatz machte aus dem Folgeabsatz einen Markdown-„indented code block“; rustdoc kompilierte ihn als Rust und `cargo test --workspace` brach ab. Der Fix ist eine Zeile, die Datei gehört dem QA-Agenten und ist inhaltlich unverändert. |

---

## 2. Echte Ausgaben der Gates

Alle Kommandos liefen in der Sandbox mit `PATH=$HOME/.cargo/bin:$PATH`, Werkzeugstand `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1`.

### 2.1 `cargo build --workspace`

```
$ cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 49s
EXIT=0
```

### 2.2 `cargo test --workspace`

```
$ cargo test --workspace
...
running 12 tests   test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 10 tests   test result: ok. 10 passed; 0 failed; ...
running 3 tests    test result: ok. 3 passed; 0 failed; ...
running 7 tests    test result: ok. 7 passed; 0 failed; ...
running 5 tests    test result: ok. 5 passed; 0 failed; ...
running 3 tests    test result: ok. 3 passed; 0 failed; ...
running 5 tests    test result: ok. 5 passed; 0 failed; ...
running 4 tests    test result: ok. 4 passed; 0 failed; ...
running 8 tests    test result: ok. 8 passed; 0 failed; ...
running 18 tests   test result: ok. 18 passed; 0 failed; ...
running 4 tests    test result: ok. 4 passed; 0 failed; ...
running 4 tests    test result: ok. 4 passed; 0 failed; ...
   Doc-tests phocommander_android    test result: ok. 1 passed; 0 failed; ...
   Doc-tests phocommander_archive     test result: ok. 3 passed; 0 failed; ...
   Doc-tests phocommander_config     test result: ok. 6 passed; 0 failed; ...
   Doc-tests phocommander_core       test result: ok. 5 passed; 0 failed; ...
   Doc-tests phocommander_testkit    test result: ok. 9 passed; 0 failed; ...
   Doc-tests phocommander_ios        test result: ok. 1 passed; 0 failed; ...
   Doc-tests phocommander_keymap     test result: ok. 9 passed; 0 failed; ...
   ...
all doctests ran in 0.28s; merged doctests compilation took 0.27s
```

**Summe: 129 Tests grün, 0 fehlgeschlagen**, verteilt auf 33 Testbinaries und 12 Doc-Test-Binaries. Der Lauf umfasst meine Einheitstests, die Doc-Tests und die Integrationstests des QA-Agenten (Zip-Slip, Dekompressionsbombe, Konfigurationsdatei, Keymap).

**Hinweis zur Messung:** Der abschließende Testlauf wurde mit `CARGO_TARGET_DIR=/tmp/pc-target` in einem **isolierten** Zielverzeichnis gefahren, weil im gemeinsamen `target/`-Verzeichnis zur selben Zeit ein zweiter Agent cargo laufen ließ. Gemessene Folge im gemeinsamen Verzeichnis:

```
error: could not parse/generate dep info at: …/target/debug/deps/crossbeam_utils-e8074372ad24503e.d
error: failed to write `…/target/debug/.fingerprint/polycool-a76a44f7ebaaffa8/invoked.timestamp`
Caused by: No such file or directory (os error 2)
```

Ein Lauf im gemeinsamen Verzeichnis meldete in der Folge **157** statt 129 Tests, weil veraltete Testbinaries doppelt gezählt wurden. Die Zahl **129** stammt aus dem isolierten Lauf und ist die belastbare.

### 2.3 `cargo fmt --all --check`

```
$ cargo fmt --all --check
EXIT=0
```

### 2.4 `cargo clippy --workspace --all-targets -- -D warnings`

```
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.69s
EXIT=0
```

Null Warnungen, null Fehler. Das schließt `unwrap_used`, `expect_used`, `todo`, `unimplemented` (aus `[workspace.lints.clippy]`) und `unsafe_code` ein — auch im Testcode, weil `--all-targets` mitläuft.

### 2.5 `scripts/check-file-size.sh`

```
$ ./scripts/check-file-size.sh
Source files measured : 34 (extensions: rs)
Soft limit            : 400 lines (warning)
Hard limit            : 550 lines (failure)
Soft-limit violations : 0
Hard-limit violations : 0
RESULT: PASS — every source file stays below 400 lines.
EXIT=0
```

Verhalten nachgemessen mit zwei eingepflanzten Dateien (420 und 600 Zeilen):

| Zustand | Ausgabe | Exit-Code |
|---|---|---:|
| 420 Zeilen (weich) | `SOFT  420 lines  …` / `RESULT: PASS with warnings — 1 file(s) exceed the soft limit of 400 lines.` | **0** |
| 600 Zeilen (hart) | `HARD  600 lines  …` / `RESULT: FAIL — a source file exceeds the hard limit of 550 lines. Split it.` | **1** |
| sauber | `RESULT: PASS — every source file stays below 400 lines.` | **0** |

### 2.6 `cargo xtask check`

```
$ cargo xtask check
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
```
Der Task führt `cargo check --workspace` aus und reicht dessen Exit-Code durch (Implementation in `xtask/src/main.rs`, `status.code().unwrap_or(1)`).

---

## 3. Das Grenzskript: Negativtest (rot) und Positivtest (grün)

### 3.1 Positivtest vor dem Verstoß

```
$ ./scripts/check-crate-boundaries.sh
Phocommander - crate boundary gate
rule: apps/* -> platform/* -> crates/*, no crate below crates/ may depend
      on another crate below crates/, registry dependencies are not checked

OK    apps/phocommander-desktop -> crates/phocommander-config (apps -> crates)
OK    apps/phocommander-desktop -> platform/phocommander-platform (apps -> platform)
OK    apps/phocommander-desktop -> crates/phocommander-core (apps -> crates)
OK    platform/phocommander-platform -> crates/phocommander-core (platform -> crates)
OK    tests -> crates/phocommander-archive (tests -> crates)
OK    tests -> crates/phocommander-config (tests -> crates)
OK    tests -> crates/phocommander-core (tests -> crates)
OK    tests -> crates/phocommander-keymap (tests -> crates)

8 path dependencies checked, 0 violations
EXIT=0
```

### 3.2 Negativtest — Verstoß bauen, Gate rot

Eingepflanzt in `crates/phocommander-core/Cargo.toml`:

```toml
[dependencies]
thiserror = { workspace = true }

# TEMPORARY VIOLATION - negative test of scripts/check-crate-boundaries.sh
phocommander-ui = { path = "../phocommander-ui" }
```

```
$ ./scripts/check-crate-boundaries.sh
Phocommander - crate boundary gate
rule: apps/* -> platform/* -> crates/*, no crate below crates/ may depend
      on another crate below crates/, registry dependencies are not checked

ERROR crates/phocommander-core -> crates/phocommander-ui: a crate below crates/ must not depend on another crate below crates/
OK    apps/phocommander-desktop -> crates/phocommander-config (apps -> crates)
…
8 path dependencies checked, 1 violations
EXIT=1
```

### 3.3 Rückbau — Gate wieder grün

```
$ ./scripts/check-crate-boundaries.sh
…
8 path dependencies checked, 0 violations
EXIT=0
```

Der Verstoß wurde zurückgebaut; `git status crates/phocommander-core/Cargo.toml` ist danach leer.

### 3.4 Zweiter Negativtest — versteckte Pfad-Abhängigkeit

Mit `sneaky = { path = "../phocommander-ui" }` außerhalb einer Abhängigkeits-Sektion:

```
ERROR crates/phocommander-ops: the path dependency ../phocommander-ui is written outside a dependency section
8 path dependencies checked, 1 violations
EXIT=1
```

### 3.5 Unabhängige Bestätigung

Der QA-Agent hat mit `scripts/check-gates-negative.sh` einen eigenen Negativtest gegen **mein** Grenzskript geschrieben (mit `phocommander-keymap` als eingepflanzter Abhängigkeit). Der Lauf ist grün:

```
$ bash scripts/check-gates-negative.sh
Gate 4 — scripts/check-crate-boundaries.sh
4.1 negative run (a crate below crates/ gets a dependency on a sibling):
  RESULT: FAIL — the gate is red on a dependency between two crates of the same level
4.2 positive run (the dependency is removed again):
  RESULT: PASS — the gate is green again
…
RESULT: PASS — every gate turned red on a broken state and green again.
EXIT=0
```

---

## 4. Crate-Grenzen-Matrix

| Crate | darf abhängen von | tatsächlich (gemessen) | regelkonform |
|---|---|---|---|
| `phocommander-core` | nichts im Workspace; nur crates.io (`thiserror`) | `thiserror` | ja |
| `phocommander-config` | nichts im Workspace; crates.io (`serde`, `toml`, `thiserror`) | `serde`, `toml`, `thiserror` | ja |
| `phocommander-keymap` | nichts im Workspace; crates.io (`serde`, `toml`, `thiserror`) | `serde`, `toml`, `thiserror` | ja |
| `phocommander-ops` | nichts im Workspace | — | ja |
| `phocommander-archive` | nichts im Workspace; crates.io (`thiserror`) | `thiserror` | ja |
| `phocommander-viewer` | nichts im Workspace | — | ja |
| `phocommander-remote` | nichts im Workspace | — | ja |
| `phocommander-ui` | nichts im Workspace; crates.io (`egui` 0.37) | `egui` | ja |
| `phocommander-platform` | `crates/*` | `phocommander-core` (Pfad), `dirs` | ja |
| `phocommander-desktop` | `platform/*`, `crates/*` | `phocommander-config`, `phocommander-core` (Dev), `phocommander-platform` | ja |
| `phocommander-android` | nichts im Workspace | — | ja |
| `phocommander-ios` | nichts im Workspace | — | ja |
| `xtask` | `crates/*` | — | ja |
| `tests` (Paket des QA-Agenten) | `crates/*` | `phocommander-archive`, `-config`, `-core`, `-keymap` (Dev) | ja |

**Beleg:** Ausgabe des Gates in Abschnitt 3.1 — 8 Pfad-Abhängigkeiten, 0 Verstöße. Alle Kisten unter `crates/` haben **null** Pfad-Abhängigkeiten; genau das ist die strenge Lesart, die ADR-0001 §3 festhält.

**Abweichung vom Auftrag, die ich melden muss:** Mein Auftrag nennt als Workspace-Mitglieder `crates/*`, `platform/phocommander-platform`, `apps/*`, `xtask`. Während meiner Sitzung hat der QA-Agent das Mitglied `tests` ergänzt (Commit noch nicht erfolgt, Arbeitsbaum). Das ist **fachlich richtig** — ein Test, der `phocommander-core` und `phocommander-config` gegeneinander hält, braucht beide, und unter der strengen Lesart darf keine Kiste unter `crates/` das. Ich habe die Grenzregel deshalb um die Ebene `tests` erweitert (`tests → crates/*` erlaubt, `tests → apps|platform|xtask` verboten) und die Matrix oben entsprechend ergänzt.

---

## 5. `cargo deny check` 0.20.2

```
$ cargo deny --version
cargo-deny 0.20.2

$ cargo deny check
advisories ok, bans ok, licenses ok, sources ok
EXIT=0
```

Vier Warnungen, keine Fehler:

| Warnung | Bewertung |
|---|---|
| 3 × `license-not-encountered` (`Apache-2.0 WITH LLVM-exception`, `BSD-3-Clause`, `ISC`) | unkritisch — erlaubte Lizenzen, die derzeit keine Abhängigkeit braucht; sie stehen für spätere Kisten (muda/rfd ziehen LGPL-freie, aber LGPL-umgebende Crates) |
| 1 × `duplicate`: `syn 2.0.119` und `syn 3.0.9` | zwei Versionen von `syn` im Baum (über `zerocopy-derive` bzw. `thiserror-derive`/`serde_derive`); `multiple-versions = "warn"` ist eingestellt, der Build ist grün |

`advisories ok` heißt: **kein bekanntes Advisory trifft die aufgelösten Versionen** — insbesondere nicht RUSTSEC-2025-0168, RUSTSEC-2026-0367, RUSTSEC-2026-0368 oder RUSTSEC-2026-0369. `zip` ist im Baum mit **9.1.0** aufgelöst (Pin `9.0.5`, Untergrenze).

### 5.1 Nachgemessene Abweichung vom Auftrag: `unmaintained`

Der Auftrag verlangt `unmaintained = "warn"`. Mit cargo-deny **0.20.2** ist dieser Wert ungültig — `unmaintained` ist dort ein *Umfang* (`all`/`workspace`/`transitive`/`none`), keine Schweregradstufe. Nachgemessen:

```
$ sed -i 's/^unmaintained = "none"$/unmaintained = "warn"/' deny.toml
$ cargo deny check
error[unexpected-value]: expected '["all", "workspace", "transitive", "none"]'
   ┌─ …/phocommander/deny.toml:56:17
   │
56 │ unmaintained = "warn"
   │                 ━━━━ unexpected value

2026-10-24 12:02:48 [ERROR] failed to deserialize config from '…/phocommander/deny.toml'
DENY_EXIT_WITH_WARN=1
```

Mit dem ungültigen Wert bricht cargo-deny ab, **bevor** es irgendetwas geprüft hat — genau der Fehlalarm, den die Spezifikation für das Schwesterprojekt Vocument beschreibt. Im Baum steht deshalb `unmaintained = "none"` (Änderung des QA-Agenten, von mir nachgemessen und bestätigt). **Das ist eine Abweichung vom wörtlichen Auftrag und braucht die Bestätigung des Owners.**

---

## 6. API-Review

| Element | Ort | Befund | Empfehlung |
|---|---|---|---|
| `phocommander_core::config::{MAX_*}` | `crates/phocommander-core/src/config.rs:28-49` | vier Grenzwerte, genau einmal, mit Namen; von `core` re-exportiert | bleibt kanonisch; die Konfigurationsdatei darf sie nur *anbieten*, nicht besitzen (Abschnitt 7) |
| `phocommander_core::Error` | `crates/phocommander-core/src/error.rs:22` | drei Varianten, jede mit den Daten, die der Leser braucht (`path`, `size`, `limit`); `#[source]` erhält die Ursache | bleibt; die vollständige Code-Tabelle steht in ADR-0003 und ist vor M1 mit dem UI zu veröffentlichen |
| `phocommander_core::Error::io` / `::too_large` | `error.rs:52` / `error.rs:75` | Konstruktoren statt `pub`-Feldern; hält die Varianten konsistent | bleibt |
| `phocommander_config::Config` | `crates/phocommander-config/src/schema.rs:22` | `deny_unknown_fields` auf jeder Sektion; Pflichtfeld `schema_version`; alle anderen Felder optional mit Vorgabe aus `core::config` | bleibt; die Vorgabewerte gehören *nicht* in diese Kiste (flacher Aufbau) |
| `phocommander_config::SUPPORTED_SCHEMA_VERSION` | `loader.rs:31` | liegt in der Kiste, die das Dateiformat besitzt; die Datei `config/defaults.toml` trägt dieselbe Zahl | die Datei ist Nutzerdaten, kein Konstanten-Spiegel — der Test in `apps/phocommander-desktop/tests/core_limits.rs` hält beide gegeneinander |
| `phocommander_config::ConfigError` | `loader.rs:38` | `Io`, `Syntax`, `UnsupportedSchemaVersion`, `InvalidValue`; Meldungen nennen das Feld | bleibt; die Codes aus ADR-0003 kommen in M1 dazu |
| `phocommander_keymap::ActionId::new` | `catalog.rs:63` | prüft Leerstring, fehlenden Trenner, leere Segmente, unerlaubte Zeichen | bleibt; die Prüfung ist strenger als der Auftrag verlangte und verhindert den Tippfehler `panelcopy` |
| `phocommander_keymap::Bindings::from_table` | `bindings.rs:63` | nimmt die Tabellenform der Konfigurationsdatei (`Aktion → Taste`) | bleibt; **M1**: die Zuordnung „Tabelle → Kontext" braucht eine eindeutige Quelle — siehe offener Punkt O4 |
| `phocommander_keymap::Bindings::conflicts` | `bindings.rs:148` | ein Konflikt je Tastenkombination, mit allen betroffenen Aktionen, deterministisch sortiert | bleibt; die Determinismus-Zusage der Spezifikation („die Tastenauflösung entscheidet bei Konflikten deterministisch") ist erfüllt |
| `phocommander_keymap::Keymap::from_context_tables` | `keymap.rs:47` | nimmt eine Map Kontext → Tabelle | bleibt |
| `phocommander_archive::resolve_entry_name` | `entry_name.rs:96` | **einzige** Stelle, die einen Archiv-Namen in einen Pfad übersetzt; sechs Negativtests | bleibt; ADR-0004 §2 ist die normative Beschreibung, die Implementierung deckt sie vollständig ab |
| `phocommander_archive::MAX_ENTRY_NAME_BYTES` | `entry_name.rs:34` | Grenzwert, den nur diese Kiste durchsetzt → lebt in dieser Kiste | bleibt; die Regel „ein Grenzwert, den nur eine Kiste durchsetzt, lebt in dieser Kiste; ein Grenzwert, über den mehrere Kisten verhandeln, lebt in `core::config`" steht in ADR-0001 §3 |
| `phocommander_platform::{host_platform, primary_modifier, config_directory}` | `host.rs:88`, `:114`, `:122` | die drei Unterschiede zwischen den Zielsystemen an einer Stelle; `config_directory` meldet `core::Error::NoConfigDirectory` | bleibt; `primary_modifier` wird von M1 gebraucht, um die Plattformvorgabe der Tastenbelegung zu füllen |
| `phocommander_ui::{Theme, visuals}` | `theme.rs:12`, `:59` | übersetzt die Themewahl in egui-Visuals; `System` folgt der Plattformvorgabe | bleibt; **offener Punkt O3**: die Konfigurationsdatei und die UI-Kiste haben je eine `Theme`-Enumeration — unter dem flachen Aufbau unvermeidbar, die Abbildung gehört in die Kompositionswurzel |
| `phocommander_desktop::main` | `apps/phocommander-desktop/src/main.rs:18` | meldet ehrlich den M0-Stand und beendet mit `exit(0)` | bleibt; wird in M1 durch die egui-Schleife ersetzt |
| `xtask` | `xtask/src/main.rs:20` | `check` reicht den Exit-Code durch; unbekanntes Kommando → Exit 2 | bleibt; `check` ist die einzige Aufgabe, die der Auftrag verlangt |

**Nicht vorhanden und deshalb nicht zu prüfen:** die Dateisystemabstraktion (ADR-0002) ist eine Entscheidung auf Papier — sie kann erst Code werden, wenn O1 entschieden ist (Abschnitt 10).

---

## 7. Doppelte Wahrheiten

| Konstante / Wert | Fundort A | Fundort B | kanonischer Ort | Zustand |
|---|---|---|---|---|
| `MAX_EXTRACTED_BYTES = 2147483648` | `crates/phocommander-core/src/config.rs:28` | `config/defaults.toml:81` | `phocommander_core::config` | **Duplikat vorhanden, maschinell abgesichert** durch `apps/phocommander-desktop/tests/core_limits.rs` — der Test schlägt fehl, sobald eine der beiden Zahlen abweicht |
| `MAX_ARCHIVE_ENTRIES = 100000` | `core/src/config.rs:35` | `config/defaults.toml:83` | `phocommander_core::config` | dito |
| `MAX_VIEWER_FILE_SIZE = 10485760` | `core/src/config.rs:42` | `config/defaults.toml:72` | `phocommander_core::config` | dito |
| `MAX_VIEWER_TEXT_SIZE = 4194304` | `core/src/config.rs:49` | `config/defaults.toml:75` | `phocommander_core::config` | dito |
| `schema_version = 1` | `crates/phocommander-config/src/loader.rs:31` | `config/defaults.toml:21` | `phocommander_config::SUPPORTED_SCHEMA_VERSION` | kein echtes Duplikat: die Datei ist Nutzerdaten, die Konstante ist die gelesene Version. Der Test `apps/.../core_limits.rs` prüft die Übereinstimmung |
| Grenzwerte 400 / 550 Zeilen | `AGENTS.md:115-116` | `scripts/check-file-size.sh:24-25` | `scripts/check-file-size.sh` (ausführbar), `AGENTS.md` (Beschreibung) | kein echtes Duplikat; beide Stellen nennen dieselben Zahlen, das Skript ist die ausführbare Form |
| cargo-deny 0.20.2 | `deny.toml:10` (Kommentar) | `.github/workflows/ci.yml:28` (`CARGO_DENY_VERSION`) | CI-Workflow (ausführbar), `deny.toml` (Kommentar) | kein echtes Duplikat; **Hinweis**: die Version steht in zwei Dateien — bei einem Werkzeugwechsel müssen beide angefasst werden |
| Primär-Modifier `Cmd` / `Ctrl` | `platform/phocommander-platform/src/host.rs:62-63` | `config/defaults.toml` (Kommentar) | `phocommander-platform` | kein echtes Duplikat: die Konfigurationsdatei beschreibt es nur |
| Aktionen der Tastenbelegung | `config/keymap.toml` | `docs/KEYMAP.md` | `config/keymap.toml` (Daten), `docs/KEYMAP.md` (Dokumentation) | **echtes Duplikat ohne Maschinenprüfung** — siehe offener Punkt O4; die QA-Prüfung `scripts/check-keymap-consistency.py` existiert bereits, ist aber von mir nicht verifiziert |

**Regel, die daraus folgt** (ADR-0001 §3): Ein Grenzwert, den nur eine Kiste durchsetzt, lebt in dieser Kiste. Ein Grenzwert, über den mehrere Kisten verhandeln müssen, lebt in `phocommander-core::config` und wird als Wert übergeben. Zwei Kisten dürfen nie dieselbe Zahl selbst wählen.

---

## 8. Die vier ADRs

Die vollständigen Texte liegen als Dateien unter `docs/adr/` (Format nach der Vocument-Vorlage: Status, Datum, Owner-Rolle, Anforderungs-IDs, Context, Decision, Consequences mit Positive/Negative, Alternatives Considered).

| ADR | Titel | Status | Kernentscheidung in einem Satz |
|---|---|---|---|
| [ADR-0001](docs/adr/ADR-0001-cargo-workspace-and-module-boundaries.md) | Cargo workspace and crate boundaries | accepted | Eine Workspace, eine Versionskatalogeinstellung, die strenge Lesart der Flachbau-Regel als ausführbares Gate — **plus die Entscheidungsvorlage O1**, weil die Regel der Spezifikation der Praxis des referenzierten Schwesterprojekts widerspricht |
| [ADR-0002](docs/adr/ADR-0002-filesystem-abstraction.md) | The filesystem abstraction | accepted | Eine Quelle, eine Methode: „list the directory“; die Fehler der Quellen sind ihre eigenen; Schreiben ist ein schmaleres Interface |
| [ADR-0003](docs/adr/ADR-0003-error-hierarchy-and-error-codes.md) | The error hierarchy and the error codes | accepted | Jede Kiste besitzt ihren Fehlertyp; die Codes sind stabile Verträge mit der Oberfläche; `unwrap`/`expect` bleiben per Lint verboten |
| [ADR-0004](docs/adr/ADR-0004-archive-extraction-security.md) | The security strategy for the extraction of archives | accepted | Kein `extract()`, eine einzige Namensauflösung, keine Symlinks aus Archivmaterial, drei Grenzwerte aus `core::config`, deskriptorbasierter Zugriff, Negativtest je Härtung |

---

## 9. Was NOT RUN bleibt — mit exaktem Grund

| Punkt | Grund |
|---|---|
| **Vollständiger Build des gesamten Versionskatalogs** (alle 38 Katalogeinträge gleichzeitig) | Die Auflösung ist belegt (709 Pakete, `cargo generate-lockfile`, Exit 0), der Übersetzen **nicht**: `muda 0.19.3` zieht unter Linux `gtk3 → pango-sys`, und der Sandbox fehlen die GTK-Entwicklungsheader. Gemessene Abbruchmeldung: `The system library 'pango' required by crate 'pango-sys' was not found.` — ein **System**-Mangel, kein Versionsproblem. 273 Kisten wurden vor dem Abbruch übersetzt. Das ist keine Eigenschaft des Katalogs, sondern der Maschine. |
| **Bau des `zip`-Stacks, des `sevenz-rust2`-Stacks, des `russh`-/`smb2`-/`rusty-s3`-Stacks und des `egui`-/`eframe`-Stacks** | Dieselbe Ursache: Die Katalogeinträge werden im Workspace noch nicht benutzt (die Kisten, die sie brauchen, kommen in M1 bzw. M6). Ein Compile-Beweis für die *auflösende* Version dieser Crates wurde deshalb nicht geführt. Der `egui`-Pfad ist belegt: `phocommander-ui` baut gegen `egui 0.37.0` (`cargo build --workspace`, Exit 0). |
| **Bau für `aarch64-linux-android`, `aarch64-apple-ios`, `x86_64-unknown-freebsd`** | Die Zielbibliotheken sind installiert (`rustup target add` für alle drei: Exit 0, `rustup target list --installed` zeigt sie). Der **Bau** wurde nicht geführt, weil in dieser Sitzung kein Android-/iOS-/BSD-Toolchain-Setup (NDK, Xcode, FreeBSD-Werkzeuge) vorhanden ist und M0 laut Spezifikation keine Gradle- und Xcode-Projekte enthält. Die `targets`-Zeile in `rust-toolchain.toml` ist damit als *Vorbereitung* verifiziert, nicht als *Bau*. |
| **`cargo audit`** | Das Werkzeug ist im Register vorhanden (`cargo info cargo-audit` → 0.22.2), aber nicht installiert; die Installation wäre ein eigener, mehrstündiger Build. Der Advisory-Check ist über `cargo deny check` abgedeckt (`advisories ok`, Exit 0). |
| **`cargo deny check` mit dem Standard-`CARGO_HOME`** | Der Advisory-Datenbank-Ordner liegt auf dem NFS-Mount, der kein `flock` unterstützt: `failed to acquire advisory database lock: … NFS do not support locking`. Der Lauf wurde mit `CARGO_HOME=/tmp/cargo-home` (lokales Overlay-Dateisystem) wiederholt und war grün. Das ist eine Eigenschaft der Sandbox, keine Eigenschaft der Konfiguration. |
| **Coverage-Messung** | Nicht gemessen. `cargo llvm-cov` ist in der Sandbox nicht installiert. Ich nenne keine Prozentzahl, weil ich keine gemessen habe. |
| **Leistungsmessung der Kriterien der Spezifikation** | Nicht gemessen. Der Benchmark des QA-Agenten (`benches/directory_listing.rs`) liegt im Baum; ich habe ihn nicht ausgeführt, weil die Messung nicht meine Aufgabe war und derzeit kein Produktcode existiert, den man messen könnte. |

**Umgebungsfehler, die während der Läufe auftraten und die ich nicht wegkonfigurieren konnte** (NFS-Mount `/workspace`, 2 Kerne): dreimal brach ein Übersetzungsschritt mit `failed to build archive … failed to map object file: memory map must have a non-zero length` bzw. `can't find crate for 'crc32fast'` ab. Ein erneuter Lauf war jedes Mal grün. Zusätzlich traten beim parallelen Betrieb zweier cargo-Läufe im selben `target/`-Verzeichnis Fehler wie `failed to write …/invoked.timestamp — No such file or directory (os error 2)` auf. Der abschließende Beweislauf wurde deshalb in einem isolierten Zielverzeichnis gefahren. Das sind Fehler des Dateisystems und der Parallelität, keine Fehler des Codes — ich nenne sie, weil ein Reviewer dieselbe Störung sieht.

---

## 10. Offene Punkte, die eine Owner-Entscheidung brauchen

| # | Punkt | Warum es der Owner entscheidet | Meine Empfehlung |
|---|---|---|---|
| **O1** | **Wie wird die Flachbau-Regel gelesen?** Streng (keine Kiste hängt an einer anderen) oder mit einer benannten Ausnahme für `phocommander-core` als Vokabular-Kiste? | Die Spezifikation (FR-116) und `docs/architecture/README.md` (Regel 2) widersprechen sich; das referenzierte Schwesterprojekt Photocraft macht die Ausnahme. ADR-0001 §4 legt beide Varianten offen. | **Ausnahme für `phocommander-core`.** Sonst braucht M1 für jeden gemeinsamen Typ eine Abbildung in der Kompositionswurzel — und die Oberfläche kann den Typ der Kiste, die ihn besitzt, nicht nennen. |
| **O2** | **Woher kommen die Grenzwerte in die Konfigurationsdatei?** Die Datei enthält die Zahlen, der Code enthält dieselben Zahlen. | Die Datei ist Nutzerdaten; die kanonische Zahl ist im Code. Wer sie erzeugt, entscheidet, ob es eine zweite Wahrheit gibt. | **Die Datei wird aus `core::config` erzeugt** (ein `xtask`-Befehl), und der Integrationstest hält beide gegeneinander. Der Test existiert bereits und ist grün. |
| **O3** | **Doppelte Enums an der Dateigrenze** (`Theme` in `config` und `ui`, `InputContext` in `config` und `keymap`, `OverwritePolicy` in `config` und später `ops`) | Unter der strengen Lesart sind zwei Kisten gezwungen, dieselbe Dateiform zu beschreiben. Das ist eine Folge der Regel, keine Sorglosigkeit. | Mit O1 verschwindet das Problem: der Konfigurationstyp wird in `core::config` gezogen und von allen gelesen. |
| **O4** | **Format des Aktionskatalogs `config/keymap.toml`** | Die Spezifikation beschreibt den Inhalt, nicht das Dateiformat. Die QA-Prüfung `scripts/check-keymap-consistency.py` und `docs/KEYMAP.md` setzen ein Format voraus, das ich nicht verifiziert habe. | Der Katalog bleibt die einzige Quelle; die Kiste `phocommander-keymap` liest die Kontext-Tabellen. Ich brauche eine Bestätigung des Formats, bevor ich den Katalog in der Kiste lese. |
| **O5** | **Die Werkzeugversion von cargo-deny steht an zwei Stellen** (`deny.toml`-Kommentar und `ci.yml`) | Ein Werkzeugwechsel muss beide Stellen treffen, sonst liest die CI eine andere Konfiguration als die Dokumentation. | Eine Stelle: die Version gehört in `ci.yml`; `deny.toml` verweist darauf. |
| **O6** | **Zusätzliches Workspace-Mitglied `tests`** (vom QA-Agenten eingefügt, von mir fachlich bestätigt und im Gate berücksichtigt) | Mein Auftrag nennt vier Mitgliedsgruppen; der Baum hat jetzt fünf. | Beibehalten — es ist die einzige saubere Lösung für grenzüberschreitende Tests unter der Flachbau-Regel. Der Auftragstext sollte nachgezogen werden. |
| **O7** | **`unmaintained = "none"` statt `"warn"`** in `deny.toml` | Der Auftrag verlangt den Wert, der mit der gepinnten Werkzeugversion unmöglich ist (gemessen, Abschnitt 5.1). | Beibehalten und den Auftragstext korrigieren — oder eine andere Werkzeugversion wählen, die „warn" kennt. |

---

## 11. Parallelarbeit im Repository — was ich vorgefunden und was ich angefasst habe

Während meiner Sitzung haben weitere Agenten in dasselbe Repository geschrieben. Das ist kein Vorwurf, es gehört aber in die Übergabe, weil es den Zustand des Baumes erklärt.

| Beobachtung | Konsequenz für mich |
|---|---|
| Zwei Commits sind **während** meiner Sitzung entstanden: `b88a27c` („feat(m0): add CI pipeline, workspace manifest, config files and rename templates“) und `5af10fb` („docs(m0): add threat model, glossary, user guide, governance docs and CI checks“). | Ich habe **nichts selbst committet** — wie beauftragt. Ein Teil meiner Arbeitsdateien (u. a. `crates/phocommander-core/src/*.rs`, `crates/phocommander-config/src/schema.rs`, `scripts/check-file-size.sh`) wurde dabei von fremder Hand mitcommittet. Der Besitzstand ist über `git log --stat` nachvollziehbar. |
| Der QA-Agent hat `tests/` als Workspace-Mitglied angelegt (`phocommander-integration-tests`) und `deny.toml` korrigiert. | Beides fachlich richtig; ich habe mein Gate um die Ebene `tests` erweitert und die `deny.toml`-Änderung nachgemessen (Abschnitt 5.1). |
| Meine Fassung von `scripts/check-file-size.sh` (11:14 UTC) lag über der bereits committeten Fassung des QA-Agenten. | Ich habe **seine** Fassung wiederhergestellt (`git checkout HEAD -- scripts/check-file-size.sh`), weil sie committet war und dieselbe Anforderung erfüllt — plus `--strict` für das weiche Limit. Verhalten nachgemessen (Abschnitt 2.5). Es gibt jetzt nur eine Implementierung. |
| `tests/common/mod.rs` (Fremddatei) brach `cargo test --workspace` durch einen fehlerhaften Doc-Kommentar. | Einzeiliger Fix (Abschnitt 1.4). Die Datei ist inhaltlich unverändert. |
| Die Tests des QA-Agenten erwarten genau meine öffentliche API (`phocommander_core::MAX_*`, `phocommander_config::load_file`, `phocommander_keymap::{ActionId, Context, Keymap}`, `phocommander_archive::{resolve_entry_name, EntryNameError}`) — sie laufen grün. | Unabhängige Bestätigung, dass die von mir gewählte Benennung der öffentlichen API trägt. |

---

## 12. Was ich bewusst nicht getan habe

- **Keine Änderung an `docs/REQUIREMENTS.md`.** Das Dokument gehört dem Owner.
- **Kein Commit, kein Push, kein Tag, kein Release.** Der Patch liegt als Datei im Baum: `deliverable-rust-architect.patch` (Anwendung mit `git apply`).
- **Keine `#[allow]`-Attribute**, um ein Gate grün zu machen. Die einzige Ausnahme im Baum ist keine von mir: `tests/Cargo.toml` schaltet `autotests`/`autobenches` ab, weil die Testdateien in Unterordnern liegen — das ist eine_layout-Entscheidung, keine Lint-Abschaltung.
- **Keine Produktfunktionen** über den M0-Rahmen hinaus: `phocommander-ops`, `-viewer` und `-remote` sind bewusst Kisten ohne Code, mit einem Kopf, der Verantwortung, Grenze und den Meilenstein des Inhalts nennt. Das ist die ausdrücklich erlaubte Form der leeren Kiste; eine Funktion, die nichts tut, hätte ich nicht geschrieben.
- **Kein Umbau des `deny.toml`, kein Umbau der `ci.yml`** — beide gehören dem QA-Agenten bzw. dem Owner.
- **Die Testdateien des QA-Agenten habe ich nicht verschoben.** Sie liegen in einem Paket außerhalb von `crates/`, was der richtige Ort ist; ich habe nur mein Gate darauf erweitert.

---

## 13. Empfohlene nächste Schritte

1. **O1 entscheiden** (Flachbau-Regel). Danach kann ADR-0002 als Code beginnen: die Dateisystemabstraktion in `phocommander-core`, eine Testimplementierung als lokale Quelle, und der Fernzugriff als zweiter Beweis.
2. **O2 und O4 entscheiden**, dann `config/defaults.toml` und `config/keymap.toml` an die Kisten hängen.
3. **M1 beginnen:** Auftragswarteschlange in `phocommander-ops`, Panelzustand in `phocommander-ui`, die Tastenauflösung aus `phocommander-keymap` in die Oberfläche, `phocommander-platform` für Menü, Dialoge und Papierkorb.
4. **Die drei fehlenden Härtungsnegativtests** aus der Spezifikation ergänzen: Dekompressionsbombe *während* der Extraktion (der jetzige Test belegt die Grenzwerte, nicht den Abbruch), Header-Limit, Symlink beim Kopieren und Löschen.

---

*Ende der Übergabe. Alle Ausgaben in diesem Dokument sind kopierte Kommandoausgaben aus der Sandbox; kein Wert ist geschätzt. Wo etwas nicht gemessen wurde, steht es als „nicht gemessen“ mit dem exakten Grund.*
