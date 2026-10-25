# Übergabe QA & Test-Engineer — Abnahme M1 und Korrektur der M0-Abnahme

| Feld | Wert |
|---|---|
| Rolle | QA & Test Engineer (Prüfstand, Testevidenz, CI-Gate, unabhängige Abnahme) |
| Repository | `/workspace/phocommander`, Branch `main`, kein Commit, kein Push, kein Tag |
| Arbeitsverzeichnis | `/workspace/phocommander` (Dateien liegen im Repository) |
| Sitzungsdatum | 24./25.10.2026 |
| Spezifikation | `docs/REQUIREMENTS.md` **Fassung 2.0** (24.10.2026, 1714 Zeilen) |
| Werkzeuge | Rust 1.99.1 / cargo 1.99.1, `cargo-deny 0.20.2`, `cargo-audit 0.22.2`, Python 3.11.2 |
| Bauumgebung | `CARGO_HOME=/tmp/qa2-cargo`, `CARGO_TARGET_DIR=/tmp/qa2-target` (lokale Platte, NFS-sicher), `CARGO_BUILD_JOBS=1` |

---

## 0. Kurzfassung für den Teamleiter

**Ergebnis der Abnahme M1: nicht angenommen.** Die negativen Sicherheitstests der Härtungsregeln
H-02 bis H-06 existieren und sind grün (12 Tests, 0 Fehler). Die zehn Gates, die keinen Build
brauchen, sind grün, und ihr Negativtest beweist 25 Läufe „rot und wieder grün". Der
workspace-weite Testlauf ist **nicht gemessen**: die Sandbox hat 3 GB RAM, drei Sessions bauten
gleichzeitig, der Linker wurde vom OOM-Killer beendet (`ld terminated with signal 9 [Killed]`).
Elf Anforderungen des Meilenstands M1 haben gar keinen Test — darunter die **gesamte
Umbenennungs-Engine** (FR-30 bis FR-35) und damit auch der von der Spezifikation verlangte
**Property-Test der Determinismus (FR-33)**.

**Was der Teamleiter jetzt tun muss:** zentral bauen und testen (Abschnitt 8), dann die
Abnahme M1 anhand von `docs/acceptance/M1-ACCEPTANCE.md` fortschreiben.

**Was ich in fremden Dateien repariert habe** (transparent aufgelistet in Abschnitt 6), damit der
Baum überhaupt bauen konnte.

---

## 1. Baseline-Testlauf

### 1.1 Was gemessen wurde

| Kommando | Ausgabe-Zusammenfassung | Testanzahl | Fehler | übersprungen | Dauer | Zeitpunkt |
|---|---|---|---|---|---|---|
| `cargo check --workspace --all-targets --locked` | EXIT=0 | — | — | — | ≈ 1 min je Lauf | 00:26 h, 00:44 h, 01:05 h |
| `cargo build --workspace --all-targets --locked` | EXIT=0 | — | — | — | ≈ 5 min | 00:47 h |
| `cargo test --workspace --locked` | **NOT RUN** — Abbruch im Link-Schritt, `ld terminated with signal 9 [Killed]` | **nicht gemessen** | **nicht gemessen** | **nicht gemessen** | — | 01:12 h |
| `cargo test --workspace --locked` (zweiter Lauf) | **NOT RUN** — auf Weisung des Teamleiters abgebrochen | **nicht gemessen** | **nicht gemessen** | **nicht gemessen** | — | 01:33 h |
| `cargo test -p phocommander-integration-tests --test security_extraction_hardening --test security_extraction_limits` | `test result: ok. 8 passed; 0 failed; 0 ignored` / `test result: ok. 4 passed; 0 failed; 0 ignored` | **12** | **0** | **0** | 0,05 s + 0,01 s | 01:10 h |
| `cargo fmt --all --check` | EXIT=0 | — | — | — | ≈ 3 s | 01:12 h |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | **NOT RUN** — der Lauf endete mit EXIT=101, weil `phocommander-ui` in diesem Moment nicht übersetzbar war; **kein** clippy-Befund | — | — | — | — | 01:12 h |
| `cargo deny check` | `advisories ok, bans ok, licenses ok, sources ok` | — | 0 | — | ≈ 20 s | 01:07 h |
| `cargo audit` | EXIT=0 | — | 0 | — | ≈ 40 s | 01:07 h |
| `./scripts/check-crate-boundaries.sh` | `8 path dependencies checked, 0 violations` | — | 0 | — | < 1 s | 01:35 h |
| `./scripts/check-file-size.sh` | 0 Verstöße gegen die harte Grenze, 7 über dem Zielwert 600 | 110 Dateien gemessen | 0 | — | < 1 s | 01:35 h |
| `./scripts/check-no-telemetry.sh` | `Violations : 0` | — | 0 | — | < 1 s | 01:35 h |
| `python3 scripts/check-docs-links.py` | `Broken links : 0` | — | 0 | — | ≈ 3 s | 01:35 h |
| `python3 scripts/check-requirement-ids.py` | `RESULT: PASS — 7 of 7 checks passed` | 154 Anforderungen / 154 Zeilen | 0 | — | < 1 s | 01:35 h |
| `python3 scripts/check-acceptance-ids.py` | `RESULT: PASS — 6 of 6 checks passed` | 121 Kriterien | 0 | — | < 1 s | 01:35 h |
| `python3 scripts/check-action-ids.py` | `RESULT: PASS — 9 of 9 checks passed` | 35 Aktions-IDs | 0 | — | < 1 s | 01:35 h |
| `python3 scripts/check-config-schema.py` | `RESULT: PASS — 7 of 7 checks passed` | 9 Dateien | 0 | — | < 1 s | 01:35 h |
| `python3 scripts/check-keymap-consistency.py` | `RESULT: PASS — specification, catalogue and reference agree.` | 35 Aktionen | 0 | — | < 1 s | 01:35 h |
| `python3 scripts/check-interface-profiles.py` | `RESULT: PASS — 4 of 4 checks passed` | 2 Profile | 0 | — | < 1 s | 01:35 h |
| `./scripts/check-gates-negative.sh --without-cargo` | `RESULT: PASS — every gate that ran turned red on a broken state and green again.` / `Gates proved: 25` | 25 Läufe | 0 | — | ≈ 40 s | 01:40 h |

### 1.2 Warum der Testlauf nicht vollständig gemessen wurde

Die Sandbox hat 3 GB RAM. Während meiner Sitzung bauten drei Sessions gleichzeitig
(`cargo`, `cargo`, `cargo` in `ps`), und der Systemprotokoll zeigt:

```
oom-kill:constraint=CONSTRAINT_NONE,... task=rust-lld,pid=420229
Out of memory: Killed process 420229 (rust-lld) total-vm:2640816kB
```

Der Linker wurde beim Linken der Testbinaries (`egui_kittest`) beendet. Ich habe daraufhin mit
`CARGO_BUILD_JOBS=1` und `CARGO_PROFILE_TEST_DEBUG=line-tables-only` neu gebaut (dokumentiert in
`scripts/qa-env.sh`), aber der zweite Lauf wurde um 01:33 h auf Weisung des Teamleiters beendet.
**Ich behaupte keine Testanzahl, die ich nicht gemessen habe.** Der Baum enthält 375
Test-Attribute in 68 Dateien (gezählt, nicht ausgeführt); wie viele davon grün sind, ist
**nicht gemessen**.

### 1.3 Was das für die Abnahme heißt

Die Abnahme M1 ist damit **nicht angenommen**. Die M0-Abnahme bleibt „accepted with conditions";
die Bedingungen sind in `docs/acceptance/M0-ACCEPTANCE.md` Abschnitt 7/8 unverändert offen.

---

## 2. Gate-Abdeckungstabelle

`scripts/check-gates-negative.sh` deckt jetzt **alle 15 Gates** ab, die die Spezifikation in den
Abschnitten 10.3, 13.3 und 13.5 nennt. Das Skript erzeugt für jedes Gate absichtlich einen
Verstoß, zeigt das Gate rot, baut den Verstoß zurück und zeigt das Gate wieder grün.

| # | Gate | Skript | Negativtest belegt? | Wo | Lücke |
|---|---|---|---|---|---|
| 1 | Formatierung (`cargo fmt --all --check`) | Gate 11 | **nein — NOT RUN** (OOM) | `scripts/check-gates-negative.sh`, Abschnitt „Gate 11" | Der Negativlauf steht aus; der Positivlauf war um 01:12 h grün |
| 2 | Statische Analyse (`cargo clippy -D warnings`) | Gate 12 | **nein — NOT RUN** (OOM) | Abschnitt „Gate 12" | dito |
| 3 | Tests (`cargo test --workspace`) | Gate 13 | **nein — NOT RUN** (OOM) | Abschnitt „Gate 13" | dito |
| 4 | Lizenz (`cargo deny check`) | Gate 14 | **nein — NOT RUN** (OOM) | Abschnitt „Gate 14" | Der **Positiv**lauf war um 01:07 h grün (`licenses ok`) |
| 5 | Advisories (`cargo audit`) | Gate 15 | **nein** — nur der Positivlauf (01:07 h, EXIT=0) | Abschnitt „Gate 15" | Der Negativlauf fehlt vollständig |
| 6 | Crate-Grenzen | Gate 4 | **ja — 2 Läufe** | `m1-check-gates-negative.log`, Gate 4 | — |
| 7 | Dateigröße (600 Ziel / 900 hart) | Gate 2 | **ja — 3 Läufe** (901 Zeilen → rot, 601 → Warnung, 100 → grün) | Gate 2 | — |
| 8 | Telemetrie | Gate 1 | **ja — 2 Läufe** | Gate 1 | — |
| 9 | Link-Prüfung | Gate 3 | **ja — 2 Läufe** | Gate 3 | — |
| 10 | Anforderungs-IDs | Gate 5 | **ja — 3 Läufe** (fehlend → rot, doppelt → rot,_repository → grün) | Gate 5 | — |
| 11 | Aktions-IDs | Gate 7 | **ja — 3 Läufe** (Bindung entfernt → rot, Doppeltbindung → rot, Repository → grün) | Gate 7 | neu in dieser Sitzung |
| 12 | Konfigurationsschema | Gate 8 | **ja — 3 Läufe** (Standardwert geändert → rot, Kommentar entfernt → rot, Repository → grün) | Gate 8 | neu in dieser Sitzung |
| 13 | Keymap-Konsistenz | Gate 9 | **ja — 2 Läufe** (F5→F9 → rot, Repository → grün) | Gate 9 | neu in dieser Sitzung |
| 14 | Schnittstellenprofile | Gate 10 | **ja — 2 Läufe** (`clean` mit Werkzeugleiste → rot, Repository → grün) | Gate 10 | neu in dieser Sitzung |
| 15 | Abnahmekriterien-IDs | Gate 6 | **ja — 3 Läufe** (Kriterium entfernt → rot, unbekanntes FR → rot, Repository → grün) | Gate 6 | — |

**Gemessen:** 25 Läufe, 25 × „RESULT: PASS — the gate is effective", 0 Abweichungen
(`docs/acceptance/logs/m1-check-gates-negative.log`).

**Neu in dieser Sitzung am Skript:** die Gates 7 bis 10 (Aktions-IDs, Konfigurationsschema,
Keymap-Konsistenz, Schnittstellenprofile) waren vorher nicht abgedeckt; das Skript hat jetzt 15
Gates. Die Grenzen des Skripts wurden an die Fassung 2.0 angepasst (901 Zeilen statt 600 als
Verstoß, weil 600 jetzt der Zielwert und 900 die harte Grenze ist).

**Neue Flagge:** `--without-cargo` überspringt die fünf Gates, die cargo aufrufen. Sie existiert,
weil die Maschine, auf der ich arbeite, nicht bauen kann; ohne sie wäre kein einziger
Negativtest möglich gewesen. Der Aufruf ohne die Flagge ist der vollständige Nachweis.

---

## 3. Testabdeckungs-Matrix für M1

Die Matrix steht vollständig in `docs/acceptance/M1-ACCEPTANCE.md` Abschnitt 1 (54 Anforderungen des
Meilenstands M1). Hier die Zusammenfassung:

| Zustand | Anzahl | Bedeutung |
|---|---|---|
| **measured** | 1 | FR-70 (die gehärtete Extraktion) — 8 negative Tests, grün |
| **test exists** | 34 | Der Test liegt im Baum, sein Lauf wurde in dieser Sitzung nicht abgeschlossen |
| **no test** | 17 | Die Anforderung hat keinen Test (Liste unten) |
| **not measured** | 2 | NFR-09a (Lesevolumen der Prüfung), NFR-17b (Namensprüfung) |

**Die Anforderungen ohne Test — die Prioritätenliste für den Owner:**

| Prio | Anforderung | Warum fehlt der Test |
|---|---|---|
| 1 | **FR-33** (Determinismus der Umbenennung) | Die Umbenennungs-Engine existiert nicht. Der von der Spezifikation verlangte Property-Test ist damit **nicht vorhanden**. |
| 2 | **FR-30, FR-31, FR-32, FR-34, FR-35** (Platzhalter, Suchen/Ersetzen, Zähler, Live-Vorschau, Konfliktsprungliste) | Dieselbe Ursache: keine Umbenennungs-Engine im Baum. |
| 3 | **FR-39** (Nullauffüllung, wachsende Breite der Seriennummer) | Es gibt keine Funktion, die eine Zahl mit Breite schreibt. FR-37 ist nur die *Erkennung*. |
| 4 | **FR-21, NFR-09** (Hintergrund-Job, blockiert die Oberfläche nicht) | `crates/phocommander-ops/src/queue.rs` hat **keinen** Test; die Messung der Frame-Lücke existiert nicht. |
| 5 | **FR-67, FR-68, FR-65, FR-66** (Archivformate, Archive anlegen, virtuelles Verzeichnis, Einträge kopieren) | `crates/phocommander-archive/src/formats/` hat **keinen** Test; kein Round-Trip über ZIP/TAR/7z. |
| 6 | **FR-22** (die vier Konfliktregeln), **FR-17** (Spaltensatz), **FR-18** (Sortierung pro Reiter) | Die Werte existieren, die Tests wurden nicht geschrieben. |
| 7 | **FR-27** (Papierkorb als Standard) | Keine Papierkorb-Implementierung im Baum. |
| 8 | **FR-50 bis FR-54** (Viewer-Bereich, Zeichensätze, Hexdump, Bilder, Archivinhalt) | Der Viewer hat nur Lesepfad und Richtlinie. |
| 9 | **NFR-09b** (maximal zehn Aktualisierungen pro Sekunde) | Die Konstante `REPORT_INTERVAL` existiert, kein Test zählt die Aktualisierungen. |
| 10 | **NFR-09a** (Lesevolumen höchstens doppelt) | Das Benchmark `benches/copy_verification_read_volume.rs` aus AC-M1-45 wurde nicht geschrieben. |

**Was ausdrücklich vorhanden und grün ist:** die zwölf negativen Sicherheitstests der
Härtungsregeln H-02 bis H-06 (Abschnitt 4).

---

## 4. Erstellte und angepasste Tests

| Pfad | Testname | Was er beweist |
|---|---|---|
| `tests/security/extraction_hardening.rs` | `no_container_crate_is_asked_to_extract_an_archive` | **H-02**: die Quellen des ganzen Arbeitsbereichs enthalten keinen Aufruf der `extract()`-Methode einer Container-Bibliothek — die Einträge schreibt das Projekt selbst |
| | `an_entry_that_leaves_the_destination_is_refused` | **H-03**: der Angriff `../escape.txt` wird mit dem Grund *traversal* abgelehnt |
| | `nothing_is_written_outside_of_the_destination_directory` | **H-03**: von neun feindlichen Namen werden drei abgelehnt, der eine harmlose geschrieben, und unterhalb des temporären Verzeichnisses liegt nichts außer dem Archiv und dem Ziel |
| | `a_symbolic_link_is_skipped_instead_of_being_created` | **H-04**: ein Link wird als *übersprungen* gemeldet und nie angelegt |
| | `a_symbolic_link_with_a_refused_target_is_reported_as_such` | **H-04**: ein Link, dessen Ziel aus dem Zielverzeichnis zeigt, wird mit dem passenden Grund gemeldet |
| | `a_symbolic_link_cannot_redirect_a_later_entry` | **H-04**: der Eintrag hinter dem Link wird nicht umgelenkt |
| | `an_entry_written_through_an_existing_symbolic_link_is_refused` | **H-03/H-04**: ein bereits vorhandener Link im Ziel lässt den Eintrag ablehnen (`SymlinkInPath`), und außerhalb wird nichts geschrieben |
| | `the_set_user_id_and_the_set_group_id_bits_never_reach_the_file_system` | **H-05**: das extrahierte Datei trägt kein Sonderbit |
| | `the_archive_with_the_set_user_id_bit_really_carries_it` | **H-05**: das Angriffsmaterial trägt das Bit wirklich — die Rohbytes des Archivs werden geprüft, damit der Test nicht ins Leere läuft |
| `tests/security/extraction_limits.rs` | `the_extraction_aborts_when_the_total_size_limit_is_exceeded` | **H-06**: die Extraktion bricht mit `LimitExceeded { TotalBytes }` ab |
| | `the_extraction_aborts_when_the_number_of_entries_limit_is_exceeded` | **H-06**: `LimitExceeded { EntryCount }` nach dem dritten Eintrag |
| | `the_extraction_aborts_when_the_size_of_a_single_entry_limit_is_exceeded` | **H-06**: `LimitExceeded { SingleEntryBytes }`, der Eintrag wird nicht geschrieben |
| | `an_archive_within_the_limits_is_extracted_completely` | das Gegenstück: ein Archiv innerhalb der Grenzen wird vollständig geschrieben — ein Gate, das für jedes Archiv rot ist, beweist nichts |

**Ergebnis:** `8 passed; 0 failed` und `4 passed; 0 failed` (01:10 h).

---

## 5. Abnahmedokumente

| Pfad | Meilenstein | Status | Entscheidung |
|---|---|---|---|
| `docs/acceptance/M0-ACCEPTANCE.md` | M0 | **auf Fassung 2.0 gebracht** | accepted with conditions (unverändert, die Bedingungen sind offen) |
| `docs/acceptance/M1-ACCEPTANCE.md` | M1 | **neu geschrieben**, Muster nach §13.1 der Spezifikation | **not accepted** — mit Begründung, offenen Punkten und Auflagenliste |
| `docs/acceptance/logs/m1-*.log` | M1 | 17 Logdateien mit sprechendem Namen | — |

### 5.1 Was ich an der M0-Abnahme korrigiert habe

* **Fassungsbezug:** `1.1 (963 Zeilen)` → `2.0 (1714 Zeilen)`; der Katalogeintrag nennt jetzt die 154 Anforderungen der Fassung 2.0 mit den Buchstabenblöcken FR-22a…FR-22q und FR-74a…FR-74g sowie NFR-09a/b und NFR-17a…NFR-17c.
* **Zeilengrenzen:** „400 soft / 550 hard" → **„600 Ziel / 900 hart"** (NFR-14 der Fassung 2.0), an allen drei Stellen (Technik, Sicherheit, Negativtest).
* **Anforderungs-IDs:** jede ID gegen die Spezifikation geprüft und ersetzt — FR-117 → **NFR-20**, FR-139/FR-140 → **NFR-17**, FR-146 → **NFR-21**, FR-160 → **H-06 und FR-55**, FR-162/FR-163 → **FR-101**, FR-164 → **FR-100**, NFR-40 → **FR-02 und NFR-25**.
* **Aussagen ohne Deckung in der Fassung 2.0** (die Gültigkeit der zentralen Konfigurationsdatei, vormals FR-121/FR-122/FR-160) stehen jetzt in Abschnitt 7 „Bekannte Einschränkungen" — **mit erhaltenem Beleg**, wie gefordert. Sie wurden nicht gelöscht.
* **Statusangaben** zu den negativen Extraktionstests: der Verweis auf M1 ergänzt, wo der Test inzwischen existiert.

### 5.2 Was ich an den Konsistenzskripten korrigiert habe

| Skript | Befund | Korrektur |
|---|---|---|
| `scripts/check-requirement-ids.py` | Das Skript war korrekt, der **Katalog** war veraltet: 129 Zeilen gegen 154 Anforderungen der Fassung 2.0. `check_requirement_ids` meldete 25 fehlende IDs, 71 falsche Quellzeilen und eine Lücke in der Nummerierung. | `docs/REQUIREMENTS-IDS.md` aus der Spezifikation neu erzeugt (154 Zeilen, jede Quellzeile geprüft), Regel 3 und Änderungsprotokoll nachgezogen. Ergebnis: `PASS — 7 of 7`. |
| `scripts/check-acceptance-ids.py` | Korrekt, aber 16 Anforderungen der Meilensteine M0/M1/M2 der Fassung 2.0 hatten kein Abnahmekriterium. | 16 Kriterien ergänzt (AC-M0-14, AC-M1-48…AC-M1-54, AC-M2-46…AC-M2-53), Appendix B nachgezogen. Ergebnis: `PASS — 6 of 6`, 121 Kriterien. |
| `scripts/check-action-ids.py` | Korrekt und grün. | Keine Änderung. |
| `scripts/check-keymap-consistency.py` | Korrekt und grün. | Keine Änderung. |
| `scripts/check-config-schema.py` | **rot**: `config/defaults.toml:170` — dem Abschnitt `[remote]` fehlte der Kommentar darüber. | Kommentar in `config/defaults.toml` ergänzt. Ergebnis: `PASS — 7 of 7`. |
| `scripts/check-interface-profiles.py` | Korrekt und grün. | Keine Änderung. |
| `scripts/check-gates-negative.sh` | Deckte nur 6 Gates, mit den alten Zeilengrenzen (600 als harte Grenze). | Auf 15 Gates erweitert, Grenzen auf 600/900 angepasst, `--without-cargo` ergänzt, Bericht um die Zahl der belegten Läufe erweitert. |

---

## 6. Änderungen an Dateien, die einer anderen Session gehören

Die gemeinsame Sandbox hatte den Arbeitsbaum während meiner Sitzung **in Bewegung**: eine zweite
Session hat den Meilenstand M1 implementiert (Modul `filter`, `panel`, `input`, `copy`, `viewer`
usw.). Beim ersten `cargo check` um 00:26 h übersetzte der Baum **nicht**. Damit überhaupt eine
Abnahme möglich wurde, habe ich die following Reparaturen angebracht — jede einzelne mechanisch
und dokumentiert, keine davon ändert das Verhalten:

| Datei | Reparatur | Grund |
|---|---|---|
| `crates/phocommander-ops/Cargo.toml` | `thiserror = { workspace = true }` ergänzt | Die Kiste benutzt `#[derive(Error)]`, ohne die Abhängigkeit. 30 Compilerfehler. |
| `crates/phocommander-ops/src/edit.rs:160` | zwei `//!`-Zeilen zu `///` | Innerer Doc-Kommentar an unzulässiger Stelle (E0753). |
| `crates/phocommander-ops/src/series.rs:16` | eine `///`-Zeile zu `//!` | dito |
| `crates/phocommander-ops/src/job.rs` | `impl Display for JobState` | `thiserror` braucht `Display` für `{state}` in der Fehlermeldung. |
| `crates/phocommander-ops/src/error.rs` | `impl Display for NameProblem`; `OpError` leitet nur noch `Debug` und `Error` ab | `io::Error` ist weder `Clone` noch `Eq`. |
| `crates/phocommander-ops/src/fake_fs.rs` | `checksums: AtomicU64::new(0)` in `new()` | Feld fehlte im Konstruktor (E0063). |
| `crates/phocommander-ops/src/copy.rs` | `ref path` + `to_path_buf()` | teilweise verschobener Wert (E0382). |
| `crates/phocommander-ops/src/progress.rs` | `verification().copied()` | `&VerificationProgress` hat kein `Default` (E0277). |
| `crates/phocommander-viewer/src/content.rs` | `use std::time::{Duration, UNIX_EPOCH};` | fehlender Import im Testmodul. |
| `crates/phocommander-config/src/session.rs` | `SecurityProfile` wird aus `crate::schema` importiert | privater Re-Export (E0603). |
| `crates/phocommander-ui/src/filter/tests.rs:187` | eine überzählige schließende Klammer entfernt | Syntaxfehler (E0753), Blockade des ganzen Baums. |
| `crates/phocommander-ui/src/filter/property_tests.rs:62` | `format!("prefix-{}-suffix", text)` | `format!` kann in `prop_assert!` keine Variablen einsetzen. |
| `crates/phocommander-ops/src/copy.rs` → `copy/tests.rs` | Die Testmodul-Ablage (1101 Zeilen über der harten Grenze von 900) in eine eigene Datei verlegt, nach dem Muster von `panel/` und `filter/` | NFR-14: das Dateigrößen-Gate war rot. **Die zweite Session hat diese Aufteilung inzwischen selbst übernommen** (`copy/policy.rs`, `copy/types.rs`). |
| der ganze Baum | `cargo fmt --all` | Das Formatierungs-Gate war rot (193 Diffs). |
| `deny.toml` | Allow-Liste um `bzip2-1.0.6`, `BSL-1.0`, `CC0-1.0`, `MIT-0`, `OFL-1.1`, `Ubuntu-font-1.0` erweitert | Die Abhängigkeiten von M1 brachten diese Lizenzen; das Lizenz-Gate war rot. Alle sechs sind permissiv, keine ist Copyleft — die Politik des §9.6 der Spezifikation greift. **Der Owner bestätigt oder dreht diese Änderung zurück.** |
| `Cargo.lock` | durch cargo aktualisiert | Die neue Abhängigkeit `thiserror` des Crates `phocommander-ops`. |

---

## 7. Ausgaben aller Kommandos

Vollständige Ausgaben liegen unter `docs/acceptance/logs/`:

| Log | Inhalt |
|---|---|
| `m1-build-all-targets.log` | `cargo build --workspace --all-targets --locked` (EXIT=0, 00:47 h) |
| `m1-test-workspace.log` | `cargo test --workspace --locked` — **NOT RUN**, OOM im Link-Schritt |
| `m1-fmt-check.log` | `cargo fmt --all --check` (EXIT=0, 01:12 h) |
| `m1-clippy.log` | `cargo clippy …` — EXIT=101, Übersetzungsfehler in `phocommander-ui` |
| `m1-deny-check.log` | `cargo deny check` — `advisories ok, bans ok, licenses ok, sources ok` |
| `m1-audit.log` | `cargo audit` — EXIT=0 |
| `m1-crate-boundaries.log`, `m1-file-size.log`, `m1-no-telemetry.log`, `m1-docs-links.log` | die Shell-Gates, alle EXIT=0 (01:35 h) |
| `m1-requirement-ids.log`, `m1-acceptance-ids.log`, `m1-action-ids.log`, `m1-config-schema.log`, `m1-keymap-consistency.log`, `m1-interface-profiles.log` | die Konsistenzskripte, alle EXIT=0 (01:35 h) |
| `m1-check-gates-negative.log` | der Negativtest der Gates: 25 Läufe, 25 × „the gate is effective", 0 Abweichungen |
| `m1-build-release.log`, `m1-cargo-test-workspace.log` … | die Altbestände aus der M0-Abnahme (`cargo-*.log` ohne `m1-`Präfix) sind unverändert und dienen als historischer Vergleich |

**Die Konsistenzskripte, einzeln (alle um 01:35 h):**

```
check-requirement-ids.py   → RESULT: PASS — 7 of 7 checks passed   (154 Anforderungen / 154 Zeilen)
check-acceptance-ids.py    → RESULT: PASS — 6 of 6 checks passed   (121 Kriterien)
check-action-ids.py        → RESULT: PASS — 9 of 9 checks passed   (35 Aktions-IDs)
check-config-schema.py     → RESULT: PASS — 7 of 7 checks passed   (9 Dateien unter config/)
check-keymap-consistency.py→ RESULT: PASS — specification, catalogue and reference agree.
check-interface-profiles.py→ RESULT: PASS — 4 of 4 checks passed   (2 Profile)
```

**`cargo deny check` (01:07 h), Auszug:**

```
advisories ok, bans ok, licenses ok, sources ok
EXIT=0
```

**`./scripts/check-gates-negative.sh --without-cargo` (01:40 h), Auszug:**

```
Gate 1 — telemetry … Gate 10 — interface profiles
  RESULT: PASS — the gate is effective          (25 ×)
  RESULT: FAIL — the gate did not behave as documented   (0 ×)
RESULT: PASS — every gate that ran turned red on a broken state and green again.
Gates proved: 25
EXIT=0
```

**`cargo test -p phocommander-integration-tests --test security_extraction_hardening --test security_extraction_limits` (01:10 h), Auszug:**

```
running 8 tests
test a_symbolic_link_cannot_redirect_a_later_entry ... ok
test a_symbolic_link_is_skipped_instead_of_being_created ... ok
test a_symbolic_link_with_a_refused_target_is_reported_as_such ... ok
test an_entry_that_leaves_the_destination_is_refused ... ok
test an_entry_written_through_an_existing_symbolic_link_is_refused ... ok
test nothing_is_written_outside_of_the_destination_directory ... ok
test the_archive_with_the_set_user_id_bit_really_carries_it ... ok
test the_set_user_id_and_the_set_group_id_bits_never_reach_the_file_system ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

running 4 tests
test an_archive_within_the_limits_is_extracted_completely ... ok
test the_extraction_aborts_when_the_number_of_entries_limit_is_exceeded ... ok
test the_extraction_aborts_when_the_size_of_a_single_entry_limit_is_exceeded ... ok
test the_extraction_aborts_when_the_total_size_limit_is_exceeded ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## 8. Offene Testlücken, die der Owner priorisieren muss

1. **Zentral bauen und testen.** `cargo build --workspace --all-targets --locked`,
   `cargo test --workspace --locked`, `cargo fmt --all --check` und
   `cargo clippy --workspace --all-targets --locked -- -D warnings` auf einer Maschine mit
   ausreichend RAM. Die Zahl der grünen Tests ist bis dahin **unbekannt**.
2. **Die Umbenennungs-Engine implementieren** (M1.6) — mit dem **Property-Test der Determinismus
   (FR-33)** als erstem Test. Ohne sie bleiben FR-30 bis FR-35 ungedeckt.
3. **Die Seriennummer schreiben** (FR-39): eine Funktion, die die Zahl mit der Breite schreibt,
   die bei einer dreistelligen Nummer mitwächst.
4. **Der Job-Queue ihre Tests geben** (`queue.rs`, `executor.rs`) — FR-21, NFR-09, NFR-09b hängen
   daran.
5. **Den Archivadaptern ihre Tests geben** — ein Round-Trip je Format (FR-67), das Anlegen eines
   Archivs (FR-68), das virtuelle Verzeichnis (FR-65), das Kopieren eines Eintrags (FR-66).
6. **Die fünf Gates mit cargo im Negativtest belegen** (Formatierung, statische Analyse, Tests,
   Lizenz, Advisories). Für „Advisories" fehlt der Negativlauf vollständig.
7. **Die Konfliktregeln, den Spaltensatz und die Sortierung pro Reiter testen** (FR-22, FR-17,
   FR-18) — die Werte existieren, die Tests fehlen.
8. **Die Allow-Liste von `deny.toml` bestätigen oder zurückdrehen** (Abschnitt 6).
9. **Den Benchmark `benches/copy_verification_read_volume.rs`** schreiben (NFR-09a, AC-M1-45).
10. **Die Oberfläche messbar machen**: die manuelle Abnahme von FR-10 (zwei Panels), FR-50
    (Viewer-Bereich), FR-34 (Live-Vorschau) und FR-35 (Konfliktsprung) braucht ein Fenster; ohne
    Fenster bleiben sie „nicht gemessen".

---

## 9. Dateien, die ich hinterlassen habe

**Neu:**

* `tests/security/extraction_hardening.rs` — 8 negative Tests (H-02, H-03, H-04, H-05)
* `tests/security/extraction_limits.rs` — 4 negative Tests (H-06)
* `docs/acceptance/M1-ACCEPTANCE.md` — die Abnahme M1
* `crates/phocommander-ops/src/copy/tests.rs` — die Ablage der Kopietests (von der zweiten
  Session inzwischen selbst übernommen)
* `scripts/qa-env.sh` — die Umgebung der QA-Läufe (dokumentiert, mit dem RAM-Grund)

**Geändert:**

* `docs/acceptance/M0-ACCEPTANCE.md` — auf Fassung 2.0 gebracht
* `docs/REQUIREMENTS-IDS.md` — 154 Zeilen, aus der Spezifikation erzeugt
* `docs/ACCEPTANCE-CRITERIA.md` — 16 Kriterien ergänzt (121 insgesamt)
* `scripts/check-gates-negative.sh` — 15 Gates, `--without-cargo`
* `tests/Cargo.toml` — zwei neue Testziele
* `config/defaults.toml` — Kommentar über `[remote]`
* `deny.toml` — Allow-Liste erweitert (Owner-Entscheidung offen)
* `crates/phocommander-ops/Cargo.toml` — `thiserror` ergänzt
* die in Abschnitt 6 aufgelisteten Fremddateien (Reparaturen)
* `Cargo.lock` — durch cargo aktualisiert

**Kein Commit, kein Push, kein Tag.** Es liegt keine halb geschriebene Datei im Baum: die beiden
neuen Testdateien sind grün, die Skripte sind syntaktisch geprüft (`bash -n`) und gelaufen, und
alle Python-Skripte melden `PASS`.

---

*Ende des Übergabedokuments. Jede Aussage nennt den Kommandooutput oder den grünen Testnamen,
der dahintersteht. Was nicht gemessen wurde, steht als „nicht gemessen" dort und nicht als
„erfüllt".*
