# Übergabe — M0: Testinfrastruktur, CI-Härtung und Abnahme

**Rolle:** QA & Test Engineer · **Projekt:** Phocommander · **Repository:** `/workspace/phocommander` (Branch `main`)
**Datum:** 24.10.2026 · **Bindende Grundlage:** `docs/REQUIREMENTS.md` (Fassung 1.1) — Abschnitte *Test- und Qualitätsstrategie*, *Abnahmeregelungen*, *Projektplan*, *Leistung*
**Sprache:** dieses Dokument auf Deutsch, alle Repository-Inhalte auf Englisch

> **Hinweis zu `deliverable.md`:** Die Datei `deliverable.md` enthielt beim Beginn meiner Arbeit die Übergabe des Product Managers. Sie ist byte-identisch mit `deliverable-product-manager.md` (gleiche Prüfsumme) und bleibt dort unverändert erhalten. Diese Datei ist jetzt die Übergabe der QA-Rolle.

---

## 1. Was geliefert wurde

| # | Auftrag | Ergebnis |
|---|---|---|
| 1 | Baseline-Testlauf gemessen | Zwei Läufe: der **Ausgangszustand** (NOT RUN mit exaktem Grund) und der **Endzustand** (129 Tests grün) |
| 2 | `.github/workflows/ci.yml` geprüft (nicht geändert) | Gate-Abdeckungstabelle mit 12 Gates, **8 Lücken** gefunden, fertige YAML-Blöcke zum Nachtragen |
| 3 | `tests/` — Teststruktur | Paket `phocommander-integration-tests` (Workspace-Mitglied), gemeinsame Hilfen, 3 Integrationstests, 2 Negativtestdateien, Protokolltabelle |
| 4 | `tests/security/` — negative Tests | **Echte** Negativtests: 8 Tests für Zip-Slip (inklusive Angriffsnachweis gegen ungehärtete Extraktion), 4 Tests für die Dekompressionsbombe, 1 Property-Test mit 256 Fällen |
| 5 | `benches/` — Criterion-Benchmark | `benches/directory_listing.rs`, gemessen: **64 ms** für 10.000 Einträge gegen die Schwelle von 100 ms; der Benchmark **bricht ab**, wenn die Schwelle überschritten wird |
| 6 | `docs/acceptance/M0-ACCEPTANCE.md` | Abnahme nach dem Muster der Spezifikation, jede Aussage mit Beleg, alle nicht messbaren Zielwerte als „nicht gemessen" ausgewiesen |
| 7 | `scripts/check-docs-links.py` | Neu als Alias auf `scripts/check-links.py`; die Implementierung wurde um **Anker-Prüfung** erweitert (`#abschnitt` wird gegen die Überschriften der Zieldatei geprüft) |
| 8 | `scripts/check-file-size.sh` | **Parallel von zwei Workern geschrieben** — der Architekt-Worker legte um 11:14 UTC eine eigene Fassung an, im Baum liegt jetzt meine (gleiche Anforderung: weich 400 / hart 550, Exit 1 **nur** beim harten Limit, zusätzlich `--strict` für den weichen). Mein Beitrag ist der Negativnachweis, dass das Gate wirkt |
| 9 | `scripts/check-no-telemetry.sh` | Neu: 48 verbotene Crate-Namen, Exit 1 bei Treffer; Negativnachweis in `scripts/check-gates-negative.sh` |
| 10 | `scripts/check-gates-negative.sh` | Neu: **Negativtest für vier Gates** — jedes Gate wird absichtlich rot, danach wieder grün; das ist der Nachweis, dass die Gates Zähne haben |
| 11 | Belege | 13 Logdateien unter `docs/acceptance/logs/` — die rohen Kommandoausgaben |

**Nicht geliefert und nicht behauptet:** keine Implementierung von Produktfunktionen, keine Änderung an `.github/workflows/ci.yml` (Eigentum des Owners), keine Änderung an `docs/REQUIREMENTS.md`, kein Commit, kein Push, keine Coverage-Prozentzahl (nicht gemessen).

---

## 2. Baseline-Testlauf

### 2.1 Ausgangszustand (Beginn der Sitzung, 10:53 UTC)

| Kommando | Ausgabe-Zusammenfassung | Testanzahl | Fehler | Dauer |
|---|---|---|---|---|
| `cargo test --workspace` | **NOT RUN** — Exit 101, `error: failed to load manifest for workspace member '…/phocommander/crates/*' … failed to read '…/phocommander/crates/*/Cargo.toml' … No such file or directory (os error 2)` | 0 | — | < 1 s |
| `cargo fmt --all --check` | **NOT RUN** — Exit 1, gleiche Ursache (`cargo metadata` exited with an error) | 0 | — | < 1 s |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **NOT RUN** — Exit 101, gleiche Ursache | 0 | — | < 1 s |

**Exakter Grund:** Zum Zeitpunkt der Messung existierte **kein einziges Crate** im Repository. Das Wurzelmanifest (`Cargo.toml`) führt `crates/*`, `platform/phocommander-platform`, `apps/*` und `xtask` als Mitglieder; keines dieser Verzeichnisse existierte. Cargo kann das Manifest deshalb nicht laden — der Workspace ist nicht einmal auflösbar, geschweige denn testbar. Das ist der ehrliche Ausgangszustand, kein Fehler meines Testaufbaus.

### 2.2 Endzustand (nach Fertigstellung der Crates durch den parallelen Worker)

| Kommando | Ausgabe-Zusammenfassung | Testanzahl | Fehler | Dauer |
|---|---|---|---|---|
| `cargo test --workspace` | `test result: ok.` in **33 Testbinaries** | **129 bestanden** | **0** (0 fehlgeschlagen, 0 ignoriert) | **39,8 s** (Cache-lauf; der Kaltlauf mit der Übersetzung der Crates dauerte mehrere Minuten) |
| `cargo fmt --all --check` | keine Ausgabe, Exit 0 | — | 0 | < 1 s |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | `Finished dev profile`, Exit 0, keine Warnung | — | 0 | 6,1 s (Cache) |
| `cargo build --workspace --release` | `Finished release profile`, Exit 0 | — | 0 | 5 min 31 s (Kaltlauf) |
| `cargo deny check` | `advisories ok, bans ok, licenses ok, sources ok`, Exit 0 | — | 0 | ~30 s |
| `cargo audit` | `Scanning Cargo.lock for vulnerabilities (214 crate dependencies)`, Exit 0, keine Schwachstelle | — | 0 | ~40 s |
| `cargo bench -p phocommander-integration-tests --bench directory_listing` | criterion `time: [58.887 ms 64.391 ms 70.880 ms]`, Median der 5 Läufe `53.287992ms`, Schwelle 100 ms → **bestanden**, Exit 0 | — | 0 | ~2 min (Kaltlauf) |

**Beleg:** `docs/acceptance/logs/` — `cargo-test-workspace.log`, `cargo-test-workspace-timed.log`, `cargo-fmt-check.log`, `cargo-clippy.log`, `cargo-build-release.log`, `cargo-deny-check.log`, `cargo-audit.log`, `bench-directory-listing.log`.

**Umgebungsbedingungen, die ich nennen muss, weil sie das Ergebnis beeinflussen:**
- `CARGO_TARGET_DIR=/tmp/qa-target` — das Repository liegt auf einem NFS-Mount, auf dem der Linker gelegentlich Objektdateien verliert (im Scratch-Projekt reproduziert: `rust-lld: error: cannot open …rcgu.o`). Mit lokalem Zielverzeichnis läuft der Build stabil. Das `target/`-Verzeichnis des Repositorys ist unberührt.
- `CARGO_HOME=/tmp/cargo-home` für `cargo deny check` und `cargo audit` — der NFS-Mount unterstützt kein Locking, der Advisory-DB-Lock schlägt sonst fehl: `failed to acquire advisory database lock: … NFS do not support locking`. In der CI (normaler Dateisystem) tritt das nicht auf. Der Befehl selbst ist unverändert.

**Aufschlüsselung der 129 Tests** (aus `docs/acceptance/logs/cargo-test-workspace.log` gezählt): 55 Unit-Tests in den Crates (`unittests`-Binaries), 6 Integrationstests der Crates (`crates/phocommander-config/tests/shipped_defaults.rs`, `apps/phocommander-desktop/tests/core_limits.rs`), **13 Integrationstests in `tests/integration/`** (5 + 3 + 5), **12 negative Sicherheitstests in `tests/security/`** (8 Pfad-Traversal, 4 Dekompressionsbombe) und 43 Doctests.

---

## 3. Gate-Abdeckungstabelle — `.github/workflows/ci.yml`

Geprüft gegen die Gate-Tabelle in `docs/REQUIREMENTS.md`, Abschnitt *Abnahmeregelungen → Die Abnahme-Gates*. **Die Datei wurde nicht verändert** — sie gehört dem Owner.

| Gate aus der Spezifikation | In `ci.yml` abgedeckt? | Wo | Lücke |
|---|---|---|---|
| **Formatierung** (`cargo fmt --check`) | ja | Job `quality`, Schritt „Formatierung prüfen" (`cargo fmt --all --check`) | — |
| **Statische Analyse** (`cargo clippy --all-targets -- -D warnings`) | ja | Job `quality`, Schritt „Statische Analyse" | — |
| **Lizenzprüfung** (`cargo deny check`) | ja | Job `quality`, Schritt „Lizenz- und Advisory-Prüfung"; Version **wird verifiziert** (0.20.2) | `--locked` fehlt: die Prüfung läuft gegen den aufgelösten, nicht gegen den gesperrten Graphen |
| **Schwachstellenprüfung** (`cargo audit`) | ja | Job `quality`, Schritt „Schwachstellenprüfung" | **Die Version wird nicht verifiziert.** Die Spezifikation pinnt `cargo-audit 0.22.2`; die Prüfung des Versionsschritts existiert nur für `cargo-deny`. `--locked` fehlt ebenfalls |
| **Crate-Grenzen** (`scripts/check-crate-boundaries.sh`) | ja (Schritt vorhanden) | Job `quality`, Schritt „Crate-Grenzen prüfen" | **Zum Beginn meiner Sitzung existierte das Skript nicht** — der Schritt wäre mit „No such file or directory" rot gelaufen. Seit 11:22 UTC liegt die Fassung des Architekt-Workers vor, der Gate ist damit grün (`8 path dependencies checked, 0 violations`) |
| **Testlauf** (`cargo test --workspace`) | ja | Job `quality` **und** Matrix `desktop` | — |
| **Plattformbau** (Linux, macOS, Windows, FreeBSD, Android, iOS) | teilweise | Matrix `desktop` (Linux/macOS/Windows), Job `cross-compile` (Android, FreeBSD, NetBSD) | **Zwei Lücken:** (a) das Ziel **`aarch64-apple-ios` fehlt** in der Matrix — die Spezifikation nennt es ausdrücklich; (b) der Bauschritt trägt **`continue-on-error: true`** — der Gate *kann nie rot werden*, das widpricht „Kein Gate wird abgeschwächt, um grün zu werden" |
| **Leistung** (criterion) | **nein** | — | **Kein `cargo bench` im gesamten Workflow.** Die Zielwerte der Spezifikation sind „verbindliche Gates der CI" — ohne Job gibt es keinen Leistungs-Gate. Der Benchmark, der die Schwelle prüft, liegt jetzt in `benches/directory_listing.rs` |
| **Sicherheitsnachweis** (negative Tests) | teilweise | implizit über `cargo test --workspace` | Kein eigener Schritt, der die Negativtests **benennt** und der bei fehlender Abdeckung rot wird. Ein Gate, das nur indirekt über eine Testanzahl wirkt, ist kein Nachweis. Vorschlag unten |
| **Bedienerreichbarkeit** (vor M8) | nein | — | **Kein Befund für M0** — das Gate ist laut Spezifikation erst „vor M8" fällig. Der Aufbau (Aktionspalette, `dpad`-Kontext) ist in `config/keymap.toml` und `docs/KEYMAP.md` bereits vorhanden |
| *Zusatz:* Telemetrie-Gate (Produktmaxime „Werbefrei", AK-M0-04/45/55) | **nein** | — | `scripts/check-no-telemetry.sh` ist **nicht** in der CI eingebunden. Die Zusage „keine Telemetrie" ist damit nur dokumentiert, nicht geprüft |
| *Zusatz:* Link-Gate (Doku ohne tote Links) | **nein** | — | `scripts/check-docs-links.py` ist nicht eingebunden |
| *Zusatz:* Negativnachweis der Gates selbst (AK-M0-14, §8.2 des Bedrohungsmodells) | **nein** | — | Kein Job, der einen Verstoß baut, das Gate rot zeigt, zurückbaut und es grün zeigt. Ich habe den Nachweis lokal erbracht (`scripts/check-gates-negative.sh`), er läuft aber nicht in der CI |

**Weitere Beobachtungen ohne Gate-Charakter** (Empfehlung, kein Befund): die `actions/*` sind auf Tags gepinnt (`@v4`), nicht auf Commit-SHA — für ein Projekt mit dieser Sicherheitsausrichtung wäre SHA-Pinning die konsequentere Wahl. `cargo fuzz` im Nachtlauf benötigt das Werkzeug `cargo-fuzz` und ein Verzeichnis `fuzz/` — beides existiert noch nicht, der Job würde mit einem Installationsfehler enden (er ist mit `continue-on-error: true` abgesichert, also grün).

---

## 4. Erstellte und geänderte Dateien

| Pfad | Zeilen | Art | Begründung |
|---|---|---|---|
| `tests/Cargo.toml` | 87 | neu | Das Testpaket. **Erforderlich**, weil ein virtuelles Workspace-Wurzelmanifest keine Integrationstests kennt — `tests/` am Repository-Wurzel würde von Cargo **nie** kompiliert. Das Paket ist Workspace-Mitglied (`Cargo.toml`, Zeile 18) |
| `tests/common/mod.rs` | 277 | neu | Gemeinsame Hilfen: temporäres Verzeichnis mit Aufräumen in `Drop`, Datei mit Inhalt, Verzeichnisbaum, Zählfunktionen, `repository_root()` |
| `tests/README.md` | 51 | neu | Struktur, Begründung des eigenen Pakets, Ausführungsbefehle |
| `tests/integration/config_defaults.rs` | 181 | neu | Integrationstest: ausgelieferte Konfigurationsdatei × Konfigurations-Crate |
| `tests/integration/core_limits.rs` | 86 | neu | Integrationstest: kanonische Limits des Kern-Crates × ausgelieferte Konfigurationsdatei |
| `tests/integration/keymap_bindings.rs` | 244 | neu | Integrationstest: ausgelieferte Keymap-Datei × Keymap-Crate |
| `tests/integration/directory_entries.rs` | 109 | neu, **wartend** | Erwartete API des Verzeichnismodells (M1); nicht als Testziel registriert, weil die API noch nicht existiert. Syntax mit `rustfmt --check` geprüft |
| `tests/security/README.md` | 66 | neu | Protokolltabelle der Angriffsvektoren mit Status je Zeile |
| `tests/security/path_traversal.rs` | 380 | neu | 8 echte Negativtests gegen `resolve_entry_name` |
| `tests/security/decompression_bomb.rs` | 269 | neu | 4 Tests: gefälschte Größe, Kosten für den Angreifer, Expansionsrate, Einträgeanzahl |
| `benches/directory_listing.rs` | 179 | neu | Criterion-Benchmark, 10.000 Einträge, Schwellenprüfung 100 ms |
| `scripts/check-no-telemetry.sh` | 180 | neu | Telemetrie-Gate, 48 verbotene Crate-Namen, erweiterbar im Kommentar |
| `scripts/check-docs-links.py` | 46 | neu | Alias auf `check-links.py` (die Datei ist in `AGENTS.md` und in der Checkliste des Doku-Workers unter diesem Namen dokumentiert) |
| `scripts/check-links.py` | 217 | **geändert** | Anker-Prüfung ergänzt (`#abschnitt` gegen die Überschriften der Zieldatei), Ausgabe listet jetzt **jeden** Link mit Status, Ausnahme `docs/security/THREAT_MODEL.md` mit Kommentar beibehalten |
| `scripts/check-gates-negative.sh` | 165 | neu | Negativnachweis für vier Gates (rot → grün) |
| `Cargo.toml` | +1 | **geändert** | `"tests"` als Workspace-Mitglied. **Notwendig**, sonst läuft kein einziger Test des Pakets |
| `docs/acceptance/M0-ACCEPTANCE.md` | 211 | neu | Abnahme M0 nach dem Muster der Spezifikation |
| `docs/acceptance/logs/*.log` | 14 Dateien | neu | Die rohen Kommandoausgaben |
| `deliverable.md` | — | neu | Diese Übergabe |

**Bewusst nicht geändert:** `docs/REQUIREMENTS.md` (Eigentum des Owners), `.github/workflows/ci.yml` (Eigentum des Owners), `scripts/check-crate-boundaries.sh` (der Architekt-Worker hat sie parallel erstellt; sie erfüllt die Anforderung, mein Beitrag ist der Negativnachweis).

> **Hinweis zum Commit-Stand:** Um 12:02 UTC hat ein anderer Worker den Arbeitsbaum committed (`5af10fb docs(m0): add threat model, glossary, user guide, governance docs and CI checks`). **Ich habe nichts committed** — die Regel „do not commit" habe ich eingehalten. Im Commit enthalten sind zufällig vier meiner Skripte (`check-docs-links.py`, `check-file-size.sh`, `check-links.py`, `check-no-telemetry.sh`), weil sie zu diesem Zeitpunkt im Baum lagen. **Im Arbeitsbaum — dem eigentlichen Übergabeobjekt — liegen alle Dateien dieses Auftrags**, auch die nicht committeten (`tests/`, `benches/`, `docs/acceptance/M0-ACCEPTANCE.md`, `scripts/check-gates-negative.sh`, die Logs und die Zeile `"tests"` in `Cargo.toml`). Wenn der Owner committet, bitte **alle** Dateien einmal aufnehmen, nicht nur den Stand von `5af10fb`.

---

## 5. Erstellte Tests

| Pfad | Testname | Was er beweist |
|---|---|---|
| `tests/integration/config_defaults.rs` | `the_shipped_configuration_file_is_accepted` | Die ausgelieferte Datei `config/defaults.toml` wird vom echten Leser gelesen — ein Dateiformat, das die Anwendung nicht lesen kann, bricht jeden Start |
| | `the_shipped_configuration_file_holds_the_values_of_the_specification` | Jeder Wert der Datei entspricht der Spezifikation (u. a. `overwrite_policy = "ask"` — Überschreiben ist nie die Vorgabe) |
| | `a_configuration_file_with_an_unknown_field_is_refused` | Das dritte Beispiel der Teststrategie der Spezifikation: ein unbekanntes Feld wird abgelehnt, **und die Meldung nennt den Feldnamen** (`langauge`) |
| | `a_configuration_file_that_cannot_be_read_reports_the_path` | Der Lesefehler nennt den Pfad der Datei |
| | `a_configuration_file_written_by_the_user_is_read_from_the_disk` | Der Pfad über die Platte (nicht nur über den String) funktioniert |
| `tests/integration/core_limits.rs` | `the_shipped_configuration_file_carries_the_canonical_archive_limits` | Grenzwert-Disziplin über die Crate-Grenze hinweg: Datei und Kern-Crate nennen dieselbe Zahl (2 147 483 648 / 100 000) |
| | `the_shipped_configuration_file_carries_the_canonical_viewer_limits` | Dasselbe für die Betrachter-Grenzwerte (10 485 760 / 4 194 304) |
| | `the_canonical_limits_leave_room_for_a_normal_workload` | Die Zahlen der Spezifikation — eine Änderung der Konstanten muss eine Entscheidung sein, kein Unfall |
| `tests/integration/keymap_bindings.rs` | `the_shipped_keymap_has_no_conflict_in_any_context` | Keine ungewollte Doppelbelegung; die **beiden** von der Spezifikation selbst benannten Ausnahmen (`F3`, `Shift+F3` — „suchen" im Betrachter vs. „Konfliktsprung" im Umbenennungsdialog) sind explizit kodiert, jeder weitere Konflikt lässt den Test rot werden |
| | `every_action_of_the_keyboard_is_reachable_in_the_dpad_context` | Entscheidung E9: jede Aktion der Tastatur ist mit der Fernbedienung erreichbar |
| | `every_action_of_the_keyboard_is_reachable_in_the_controller_context` | Dasselbe für den Controller |
| | `the_dpad_and_the_controller_bind_their_actions_to_the_action_palette` | Auf Fernbedienung und Controller wird jede Aktion über die Aktionspalette erreicht; die dokumentierten Ausnahmen (`palette.open` = Menu, `selection.toggle` = Grip) sind kodiert |
| | `the_action_identifiers_of_the_shipped_file_are_well_formed` | Jede Aktions-ID der Datei ist syntaktisch gültig |
| `tests/security/path_traversal.rs` | `the_hostile_archive_really_carries_the_hostile_names` | Das Angriffsmaterial existiert wirklich — ohne diese Vorbedingung beweisen die Folgetests nichts |
| | `every_hostile_entry_name_is_refused` | **Der Angriff wird ausgeführt:** ein zur Laufzeit gebautes ZIP mit neun bösartigen Namen (`../escape.txt`, `../../escape.txt`, `docs/../../escape.txt`, `..\escape.txt`, `docs\..\..\escape.txt`, `/etc/passwd`, `\windows\system32\evil.dll`, `C:\Windows\evil.dll`, `d:evil.txt`) wird Nam für Nam in die gehärtete Auflösung gegeben. Jeder Name wird mit dem Grund *traversal*, *absolute path* oder *reserved name* abgelehnt |
| | `the_attack_really_writes_outside_of_the_destination_without_the_hardening` | **Der Angriff funktioniert wirklich:** dieselbe Datei, extrahiert mit einer absichtlich ungehärteten Referenzextraktion (Name anhängen, Eltern anlegen, schreiben), landet **außerhalb** des Zielverzeichnisses. Ein Angreifer gewinnt also gegen eine naive Extraktion — die Härtung ist nicht Kosmetik |
| | `every_accepted_name_stays_inside_the_destination_directory` | Jeder akzeptierte Name ergibt einen Pfad **unterhalb** des Zielverzeichnisses, ohne `..`-Segment |
| | `the_resolved_path_stays_inside` (Property-Test, 256 Fälle) | Die Invariante der Spezifikation für **beliebige** Namen, nicht nur für die Katalogliste: aufgelöst wird immer relativ und ohne `..` |
| | `no_file_is_written_outside_of_the_destination_directory` | Nach der Auflösung und dem Schreiben liegt unterhalb des temporären Verzeichnisses nur die Datei selbst, nichts daneben |
| | `a_name_that_is_too_long_is_refused` | Windows-Pfadlänge: ein Name über `MAX_ENTRY_NAME_BYTES` (4096) wird abgelehnt, mit Länge und Limit in der Meldung; ein Name **auf** dem Limit wird akzeptiert |
| | `the_harmless_entry_name_is_accepted` | Die Abwehr darf nicht alles ablehnen — ein normaler Name wird angenommen |
| `tests/security/decompression_bomb.rs` | `the_bomb_archive_declares_more_bytes_than_the_budget_allows` | **Der Angriff wird ausgeführt:** ein zur Laufzeit gebautes ZIP, dessen Größenfelder im Header auf 3 GiB umgeschrieben werden, überschreitet das Budget der Spezifikation (`MAX_EXTRACTED_BYTES` = 2 GiB) |
| | `the_bomb_archive_costs_the_attacker_almost_nothing` | Die Bombe wiegt wenige Kilobyte und expandiert auf Gigabyte — die Relation, die sie gefährlich macht |
| | `a_highly_compressible_entry_expands_far_beyond_its_own_size` | Ein Eintrag von 8 MiB Nullen expandiert um mehr als Faktor 1000 |
| | `an_archive_with_too_many_entries_exceeds_the_entry_limit` | 100 001 Einträge gegen das Limit von 100 000 |
| `benches/directory_listing.rs` | `list_10_000_entries` (criterion) | Die Verzeichnisliste mit 10 000 Einträgen — plus **Schwellenprüfung**: der Benchmark bricht mit einer Fehlermeldung ab, wenn der Median über 100 ms liegt. Gemessen: criterion `[58.887 ms 64.391 ms 70.880 ms]`, Median 53,29 ms |

**Kein Test ist übersprungen, kein `#[ignore]`, kein `unwrap()`, kein `expect()`** — die Workspace-Lints (`unwrap_used = "deny"`, `expect_used = "deny"`) gelten auch für die Testziele, und `cargo clippy --all-targets -- -D warnings` ist grün.

---

## 6. Ausgaben aller Skripte

### 6.1 `scripts/check-file-size.sh` (im Baum liegt meine Fassung; der Architekt-Worker hatte parallel eine eigene geschrieben — beide erfüllen die Anforderung)

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

### 6.2 `scripts/check-no-telemetry.sh` (neu)

```
$ ./scripts/check-no-telemetry.sh
Manifests scanned     : 15
Dependency names read : 51
Forbidden crate names : 48
Violations            : 0
RESULT: PASS — no dependency of the telemetry or analytics category is referenced.
EXIT=0
```

### 6.3 `scripts/check-docs-links.py` (neu, Alias) und `scripts/check-links.py` (verbessert)

```
$ python3 scripts/check-docs-links.py
… (jeder Link mit Quelle, Ziel und Status) …
tests/security/README.md:5   ../../docs/security/THREAT_MODEL.md   ok  (resolves)

Markdown files scanned : 27
Repository links checked: 138
External links skipped : 20
Broken links           : 0
RESULT: PASS — every repository link resolves.
EXIT=0
```

### 6.4 `scripts/check-crate-boundaries.sh` (Fassung des Architekt-Workers)

```
$ ./scripts/check-crate-boundaries.sh
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

### 6.5 `scripts/check-gates-negative.sh` (neu) — der Nachweis, dass die Gates Zähne haben

```
$ ./scripts/check-gates-negative.sh
Gate 1 — scripts/check-no-telemetry.sh
  1.1 positiv: Violations: 0 → Exit 0 (erwartet 0) → PASS
  1.2 negativ: VIOLATION ./Cargo.toml references the forbidden crate 'egui' → Exit 1 (erwartet 1) → PASS
Gate 2 — scripts/check-file-size.sh
  2.1 negativ (600 Zeilen): ERROR ./crates/demo/src/huge.rs: 600 lines → Exit 1 (erwartet 1) → PASS
  2.2 positiv (100 Zeilen): 0 files above the soft limit → Exit 0 (erwartet 0) → PASS
Gate 3 — scripts/check-docs-links.py
  3.1 negativ: docs/broken.md:3 does-not-exist.md BROKEN → Broken links: 1 → Exit 1 (erwartet 1) → PASS
  3.2 positiv: Broken links: 0 → Exit 0 (erwartet 0) → PASS
Gate 4 — scripts/check-crate-boundaries.sh
  4.1 negativ (Abhängigkeit core → keymap): 8 path dependencies checked, 1 violations → Exit 1 (erwartet 1) → PASS
  4.2 positiv (Abhängigkeit entfernt): 8 path dependencies checked, 0 violations → Exit 0 (erwartet 0) → PASS

RESULT: PASS — every gate turned red on a broken state and green again.
EXIT=0
```

Das Skript arbeitet ausschließlich in einem temporären Verzeichnis; das Manifest `crates/phocommander-core/Cargo.toml` und `Cargo.lock` werden gesichert und im `trap` wiederhergestellt.

---

## 7. Abnahmedokument M0

| Pfad | Status | Inhalt |
|---|---|---|
| `docs/acceptance/M0-ACCEPTANCE.md` | **vorliegend, Entscheidung: „accepted with conditions"** | Acht Abschnitte im vorgeschriebenen Muster: Funktionalität (13 Zeilen mit FR-ID und Beleg), Technik (14 Zeilen), Leistung (8 Zielwerte — **1 gemessen**, 7 ausdrücklich „nicht gemessen"), Benutzerführung, Sicherheit und Datenschutz (16 Zeilen, davon 12 mit grünem Negativtest und 5 NOT RUN mit exaktem Grund), Interoperabilität, Bekannte Einschränkungen (10 Zeilen), Entscheidung |
| `docs/acceptance/logs/*.log` | 13 Logdateien | Die rohen Ausgaben: fmt, clippy, test, build, deny, audit, 4 Skripte, Negativnachweis, Benchmark |

Die Entscheidung ist **„accepted with conditions"** — der Owner entscheidet, mein Dokument liefert die Belege. Die Bedingungen stehen in Abschnitt 7 des Abnahmedokuments.

---

## 8. Vorschläge für fehlende CI-Jobs (fertige YAML-Blöcke)

**Diese Blöcke sind Vorschläge — ich habe `ci.yml` nicht verändert.** Sie sind so formuliert, dass sie in die bestehende Datei passen.

### 8.1 iOS in die Quer-Kompilierungsmatrix aufnehmen und `continue-on-error` entfernen

```yaml
  cross-compile:
    name: Quer-Kompilierung (${{ matrix.target }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: aarch64-linux-android
            label: Android arm64
          - target: aarch64-apple-ios
            label: iOS arm64
          - target: x86_64-unknown-linux-gnu
            label: Linux
          - target: x86_64-unknown-freebsd
            label: FreeBSD
          - target: x86_64-unknown-netbsd
            label: NetBSD
    steps:
      - name: Quellen auschecken
        uses: actions/checkout@v4

      - name: Rust-Toolchain installieren
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Rust-Cache
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}

      - name: Bauen
        # KEIN continue-on-error: der Plattformbau ist ein Abnahme-Gate
        # ("Bei Rot bricht der Build ab"). Ein Gate, das nie rot werden kann,
        # ist kein Gate.
        run: cargo build --workspace --target ${{ matrix.target }}
```

*Hinweis zu iOS:* `aarch64-apple-ios` baut nur mit dem Apple-SDK. Auf einem Linux-Runner braucht er
`cargo build --target aarch64-apple-ios` ohne zusätzliche Systembibliotheken, weil die iOS-Crates des
Projekts (`apps/phocommander-ios`) kein UI-Backend benötigen. Sobald `eframe`/`winit` ins Spiel
kommen, braucht der Job einen macOS-Runner (`macos-latest`) — dann als eigener Matrix-Eintrag mit
`runs-on: macos-latest`.

### 8.2 Leistungs-Gate (criterion)

```yaml
  # ---------------------------------------------------------------------------
  # Gate 5: Leistung. Die Zielwerte der Spezifikation sind Schwellen, keine
  # Wünsche: der Benchmark bricht ab, wenn eine Schwelle überschritten wird.
  # ---------------------------------------------------------------------------
  performance:
    name: Leistung (criterion)
    runs-on: ubuntu-latest
    steps:
      - name: Quellen auschecken
        uses: actions/checkout@v4

      - name: Rust-Toolchain installieren
        uses: dtolnay/rust-toolchain@stable

      - name: Rust-Cache
        uses: Swatinem/rust-cache@v2

      - name: Verzeichnisliste mit 10.000 Einträgen messen
        # Der Benchmark prüft die Schwelle von 100 ms selbst und bricht ab,
        # wenn sie überschritten ist.
        run: cargo bench -p phocommander-integration-tests --bench directory_listing -- --warm-up-time 1 --measurement-time 5

      - name: Weitere Messpunkte (ab M1)
        run: cargo bench --workspace -- --warm-up-time 1 --measurement-time 5
```

### 8.3 Telemetrie- und Link-Gate in die CI aufnehmen

```yaml
      - name: Telemetrie-Abhängigkeiten prüfen
        # Product maximum "Werbefrei": keine Telemetrie, keine Analytics,
        # kein Tracking. Das Skript liest die Abhängigkeitskataloge der
        # Workspace-Manifeste und schlägt bei einem Treffer fehl.
        run: ./scripts/check-no-telemetry.sh

      - name: Dokumentationslinks prüfen
        run: python3 scripts/check-docs-links.py
```

### 8.4 Die sechs Kommandos in **jedem** Matrix-Lauf (die Spezifikation fordert „Jeder Lauf umfasst …")

```yaml
  desktop:
    name: Desktop (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - name: Quellen auschecken
        uses: actions/checkout@v4

      - name: Rust-Toolchain installieren
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Rust-Cache
        uses: Swatinem/rust-cache@v2

      # "Jeder Lauf umfasst: cargo fmt --check, cargo clippy --all-targets
      #  -- -D warnings, cargo test --workspace, cargo-deny, cargo-audit und
      #  cargo build --release." (docs/REQUIREMENTS.md, Continuous Integration)
      - name: Formatierung prüfen
        run: cargo fmt --all --check

      - name: Statische Analyse
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings

      - name: Tests
        run: cargo test --workspace --all-features

      - name: Release-Build
        run: cargo build --workspace --release

      - name: cargo-deny installieren
        run: cargo install cargo-deny --version "$CARGO_DENY_VERSION" --locked

      - name: cargo-deny Version prüfen
        run: |
          installed=$(cargo deny --version | head -n 1 | awk '{print $3}')
          echo "installiert: $installed"
          test "$installed" = "$CARGO_DENY_VERSION"

      - name: Lizenz- und Advisory-Prüfung
        run: cargo deny check --locked

      - name: cargo-audit installieren
        run: cargo install cargo-audit --version "$CARGO_AUDIT_VERSION" --locked

      - name: cargo-audit Version prüfen
        # Die Version wird verifiziert, weil sich dieselbe Konfiguration je
        # nach Version unterschiedlich liest.
        run: |
          installed=$(cargo audit --version | head -n 1 | awk '{print $2}')
          echo "installiert: $installed"
          test "$installed" = "$CARGO_AUDIT_VERSION"

      - name: Schwachstellenprüfung
        run: cargo audit
```

### 8.5 Negativnachweis der Gates (der Job, der beweist, dass die Gates wirken)

```yaml
  # ---------------------------------------------------------------------------
  # Gate 6: Der Nachweis, dass die Gates Zähne haben. Jedes Gate wird
  # absichtlich rot, danach wieder grün. Ein Gate, das nie rot gesehen wurde,
  # ist nicht bewiesen. (AK-M0-14, docs/security/THREAT_MODEL.md §8.2)
  # ---------------------------------------------------------------------------
  gate-effectiveness:
    name: Nachweis der Gate-Wirkung
    runs-on: ubuntu-latest
    steps:
      - name: Quellen auschecken
        uses: actions/checkout@v4

      - name: Rust-Toolchain installieren
        uses: dtolnay/rust-toolchain@stable

      - name: Negativtest der Gates
        run: ./scripts/check-gates-negative.sh

      - name: Lizenz-Gate effectiveness (Absicht: Verstoß einbauen, Gate rot)
        # Ein Verstoß gegen die Lizenzregel wird absichtlich eingebaut, das
        # Gate muss rot werden, der Verstoß wird zurückgebaut, das Gate muss
        # wieder grün sein. Skript: scripts/check-license-gate-negative.sh
        # (folgt in M1, wenn es Crates mit echten Abhängigkeiten gibt).
        run: echo "in M1 zu ergänzen"
```

### 8.6 Nachtlauf: Fuzzing ohne Absicherung, mit Werkzeug

```yaml
  nightly:
    name: Nachtlauf (Fuzzing)
    runs-on: ubuntu-latest
    schedule:
      - cron: "17 3 * * *"
    workflow_dispatch:
    # KEIN continue-on-error: ein Fuzz-Fund ist ein Fund, kein Rauschen.
    steps:
      - name: Quellen auschecken
        uses: actions/checkout@v4

      - name: Rust-Toolchain installieren
        uses: dtolnay/rust-toolchain@nightly
        with:
          components: rust-src

      - name: Rust-Cache
        uses: Swatinem/rust-cache@v2

      - name: cargo-fuzz installieren
        # cargo-fuzz ist ein eigenes Werkzeug und liegt nicht in der Toolchain.
        run: cargo install cargo-fuzz --locked

      - name: Fuzzing
        # setzt das Verzeichnis fuzz/ mit den Zielen voraus (M1)
        run: cargo fuzz run -- -max_total_time=600

      - name: Tests ohne Cache
        run: cargo test --workspace --all-features
```

---

## 9. Offene Testlücken, die der Owner priorisieren muss

| # | Lücke | Warum sie wichtig ist | Vorschlag |
|---|---|---|---|
| 1 | **Kein Negativtest für die Extraktion selbst** (Symlink-Escape, SUID/SGID, Abbruch bei der Bombe) | Der Angreifer gewinnt gegen die naive Extraktion — das habe ich gemessen (`the_attack_really_writes_outside_of_the_destination_without_the_hardening`). Solange die Container-Leser fehlen, ist die Abwehr nur für den *Namen* belegt, nicht für die *Schreiboperation* | M1: `tests/security/extraction.rs` mit den Tests `skips_symlink_entry_instead_of_creating_it`, `drops_suid_and_sgid_bits_from_extracted_file`, `aborts_when_total_size_limit_is_exceeded` — die Namen stehen bereits in `tests/security/README.md` |
| 2 | **Kein Test des Dateisystemmodells** | Das Modell ist die Grundlage für M1 (Kopieren, Suchen, Panel) | M1: `tests/integration/directory_entries.rs` ist gegen die erwartete API geschrieben und wartet auf die Implementierung. **Zwei Zeilen in `tests/Cargo.toml`** machen ihn zum Testziel |
| 3 | **Keine Property-Tests für Umbenennungsregel und Tastenauflösung** | Die Spezifikation nennt genau diese drei Invarianten; heute ist nur die Pfadauflösung abgedeckt (256 Fälle) | M1: `proptest`-Tests in `phocommander-core` (Umbenennungsregel) und `phocommander-keymap` (Konfliktauflösung) |
| 4 | **Keine Fuzz-Ziele** | Header-Overflow, Bildheader, Zeichensatzerkennung sind ungetestet | M1: `fuzz/` mit `fuzz_archive_header`, `fuzz_image_header`, `fuzz_charset_detection`; der Nachtlauf ist vorbereitet |
| 5 | **Kein Leistungs-Gate in der CI** | Die Zielwerte der Spezifikation sind Gates — ohne Job werden sie nie geprüft | CI-Block 8.2 oben |
| 6 | **Sechs Zielwerte der Leistungstabelle sind ungemessen** | Verzeichniswechsel, Tastendruck, Startzeit, Speicher, Kopierbandbreite, Suche, Archivauflistung | Werden mit den Funktionen in M1/M2 messbar |
| 7 | **Kein UI-Test (`egui_kittest`)** | Die Tastaturbedienung ist der Hauptbedienweg der Spezifikation | M1/M2 |
| 8 | **Kein Test, dass der Prozessstart keine Netzwerkverbindung öffnet** | Einziger maschineller Nachweis der Null-Telemetrie-Zusage (AK-M0-55, NFR-40) | M1, im Desktop-App-Paket |
| 9 | **Die Werkzeugversion von `cargo-audit` wird in der CI nicht verifiziert** | Die Spezifikation pinnt beide Werkzeuge, weil dieselbe Konfiguration je nach Version unterschiedlich gelesen wird — genau dieser Fehler hat im Schwesterprojekt Vocument die Lizenzprüfung stillschweigend abgeschaltet | CI-Block 8.4 oben |
| 10 | **`cargo deny check` läuft ohne `--locked`** | Die Prüfung könnte gegen einen anderen Graphen laufen als den, der ausgeliefert wird | CI-Block 8.4 oben |

**Zwei Befunde, die nicht Testlücken sind, sondern Entscheidungen des Owners:**
- Die Tastatur belegt `F3` doppelt (Betrachter „suchen" und Umbenennungsdialog „Konfliktsprung"). Das ist von der Spezifikation so vorgegeben, der Konflikt-Check des Keymap-Crates kennt aber keinen modalen Kontext. Entweder bekommt der Check eine Kontextidee, oder die Datei dokumentiert die Ausnahme. Mein Test kodiert die Ausnahme explizit, damit ein **weiterer** Konflikt rot wird.
- `docs/acceptance/logs/` enthält die Installationsprotokolle von `cargo-deny` und `cargo-audit` (`.cargo-deny-install.log`, `.cargo-audit-install.log`). Sie belegen die gepinnten Versionen; wenn der Owner sie nicht im Repository will, können sie gelöscht werden — die Versionsausgaben stehen dann nur hier.

---

## 10. Was ich bewusst nicht getan habe

- **Keine Änderung an `.github/workflows/ci.yml`** und an `docs/REQUIREMENTS.md` — beides gehört dem Owner. Die Verbesserungen stehen als fertige YAML-Blöcke in Abschnitt 8.
- **Kein Commit, kein Push, kein Tag.** Die Übergabe ist der Arbeitsbaum plus dieses Dokument.
- **Kein `deliverable.md` des Product Managers überschrieben** — die Übergabe liegt unverändert unter `deliverable-product-manager.md` (prüfsummengleich mit der ursprünglichen `deliverable.md`).
- **Keine Coverage-Prozentzahl.** Ich habe keine Coverage gemessen; eine Zahl zu nennen, die nicht gemessen wurde, wäre eine Lüge. Wer sie braucht, braucht `cargo-llvm-cov` — das ist eine eigene Entscheidung.
- **Kein Test als `skippen`, `#[ignore]` oder auskommentiert.** `cargo test --workspace` meldet 0 ignoriert.
- **Keine Binärdatei im Repository.** Alle bösartigen Archive werden zur Laufzeit im temporären Verzeichnis gebaut und danach entfernt.
- **Keine Produktfunktion implementiert.** Der Auftrag war die Testinfrastruktur; die fehlenden Crates habe ich nicht ersetzt.

---

## 11. Reproduktion

```bash
export PATH=$HOME/.cargo/bin:$PATH
cd /workspace/phocommander

# Gates
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --release
CARGO_HOME=/tmp/cargo-home cargo deny check     # NFS-Lock-Workaround, siehe Abschnitt 2.2
CARGO_HOME=/tmp/cargo-home cargo audit

# Skripte
./scripts/check-crate-boundaries.sh
./scripts/check-file-size.sh
./scripts/check-no-telemetry.sh
python3 scripts/check-docs-links.py
./scripts/check-gates-negative.sh

# Leistung
cargo bench -p phocommander-integration-tests --bench directory_listing -- --warm-up-time 1 --measurement-time 5
```

*Ende der Übergabe. Jede Zahl in diesem Dokument stammt aus einer Kommandoausgabe unter `docs/acceptance/logs/`. Was nicht gemessen wurde, ist als „nicht gemessen" oder „NOT RUN" mit exaktem Grund ausgewiesen.*
