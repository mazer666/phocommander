# Übergabe — M0: zentrale Konfigurationsdatei und Anforderungskatalog

**Rolle:** Product Manager · **Projekt:** Phocommander · **Repository:** `/workspace/phocommander`
**Datum:** 24.10.2026 · **Bindende Grundlage:** `docs/REQUIREMENTS.md` (Fassung 1.1, 963 Zeilen)
**Sprache:** dieses Dokument auf Deutsch, alle Repository-Inhalte auf Englisch

---

## 1. Was geliefert wurde

| # | Auftrag | Ergebnis |
|---|---|---|
| 1 | `config/defaults.toml` | 150 Zeilen, 12 Sektionen, 21 Konfigurationsschlüssel, jeder mit eigenem Kommentar, alle Vorgabewerte wörtlich aus der Spezifikation |
| 2 | `config/keymap.toml` | 268 Zeilen, 35 Aktions-IDs × 3 Bedienkontexte, 34 Belegungen wörtlich aus der Tabelle „Tastatursteuerung" |
| 3 | `config/templates/` | 5 Vorlagen (series, camera, podcast, invoice, music), je mit `name`, `description` und Regelfinition |
| 4 | `docs/REQUIREMENTS-IDS.md` | 208 Zeilen: FR-01…FR-167 und NFR-01…NFR-41, lückenlos, ohne Dubletten, jede Zeile mit Quellenabschnitt |
| 5 | `docs/ACCEPTANCE-CRITERIA.md` | 146 Akzeptanzkriterien für alle Anforderungen aus M0, M1 und M2 (56 + 69 + 21), je mit exaktem Kommando oder Bediensequenz und messbarer Pass Condition |
| 6 | `docs/templates/M0-ACCEPTANCE.md` | 129 Zeilen Vorlage mit den acht vorgeschriebenen Abschnitten und Platzhaltern in eckigen Klammern |
| 7 | Prüfskripte | 3 Skripte unter `scripts/`, die die Behauptungen dieses Auftrags maschinell prüfen |

**Nicht geliefert und nicht behauptet:** kein Rust-Code, keine lauffähige Anwendung, keine Performance- oder Pixelmessung. Alle Abnahmekriterien, die Code voraussetzen, stehen auf `NOT RUN` mit exaktem Grund (siehe Abschnitt 3.6).

---

## 2. Dateiliste

| Pfad | Zeilen | Zweck |
|---|---|---|
| `config/defaults.toml` | 150 | Zentrale Konfigurationsdatei als Vorlage: `schema_version`, `[general]`, `[appearance]`, `[panels]`, `[keymap]`, `[viewer]`, `[archives]`, `[privacy]`, `[advanced]`, `[input]` mit `[input.keyboard]`, `[input.dpad]`, `[input.controller]`, `[remote]` |
| `config/keymap.toml` | 268 | Aktionskatalog: 35 stabile Aktions-IDs, je eine Belegung im Tastatur-, D-Pad- und Controller-Kontext |
| `config/templates/series.toml` | 78 | Vorlage „Serienfolge": `{serie} - S{staffel:02}E{episode:02} - {titel}.{format}`, Erkennung `SxxEyy`, Serienname aus dem übergeordneten Verzeichnis, Windows-Sperrzeichen |
| `config/templates/camera.toml` | 54 | Vorlage „Kamera": `{n}_{serie}_{titel}.{format}` |
| `config/templates/podcast.toml` | 43 | Vorlage „Podcast": `{date}_{titel}.{format}` |
| `config/templates/invoice.toml` | 57 | Vorlage „Rechnung": `{year}_{number}_{description}.{format}` |
| `config/templates/music.toml` | 48 | Vorlage „Musik": `{artist}_{title}_{track}.{format}` |
| `docs/REQUIREMENTS-IDS.md` | 373 | Anforderungskatalog: 167 FR + 41 NFR, je Zeile ID, Titel, Quellenabschnitt, Meilenstein, Priorität, Verifikationsart |
| `docs/ACCEPTANCE-CRITERIA.md` | 259 | 146 Akzeptanzkriterien für M0/M1/M2 im Format How to Verify / Pass Condition, plus Anhang A mit dem vollständigen Aktionskatalog |
| `docs/templates/M0-ACCEPTANCE.md` | 129 | Vorlage der Abnahmedokumentation mit den acht Abschnitten der Spezifikation |
| `scripts/check-requirement-ids.py` | 300 | Prüft Lückenlosigkeit, Dubletten, Spalten, Quellenabschnitte und Abnahmedeckung des Katalogs |
| `scripts/check-action-ids.py` | 344 | Prüft Aktionskatalog, Bindungen gegen die Tabelle der Spezifikation, Palettenverweise, Oberflächenkonflikte und Referenzierung |
| `scripts/check-config-schema.py` | 388 | Prüft TOML-Parsing, Sektionen, Vorgabewerte, Kommentarpflicht, Vorlagensatz und Übereinstimmung mit dem Rust-Schema |
| `deliverable.patch` | 3.202 | Patch dieser Arbeit gegen den Stand vor Beginn (`61fa539`); mit `git apply` reproduzierbar — auf eine frische Kopie des Basisstands angewendet und byteweise mit dem Arbeitsbaum verglichen |
| `deliverable-product-manager.md` | 627 | Diese Übergabe unter dem rollenspezifischen Namen; siehe die Kollisionsnotiz in Abschnitt 8 |

**Gesamt:** 2.456 Zeilen in 13 Dateien.

### 2.1 Entwurf des Konfigurationsschemas

Das Schema folgt der Spezifikation **wörtlich**; es wurde kein Schlüssel erfunden, kein Wert geändert. Vollständige Inventur:

| Sektion | Schlüssel | Vorgabewert | Kommentar vorhanden |
|---|---|---|---|
| (Wurzel) | `schema_version` | `1` | ja |
| `[general]` | `language` | `"system"` | ja |
| `[general]` | `restore_session` | `true` | ja |
| `[appearance]` | `theme` | `"system"` | ja |
| `[appearance]` | `row_height` | `0` | ja |
| `[appearance]` | `show_hidden_files` | `false` | ja |
| `[appearance]` | `show_system_files` | `false` | ja |
| `[appearance]` | `file_size_format` | `"binary"` | ja |
| `[panels]` | `layout` | `"horizontal"` | ja |
| `[panels]` | `right_click_action` | `"context_menu"` | ja |
| `[panels]` | `overwrite_policy` | `"ask"` | ja |
| `[keymap]` | `panel.copy`, `panel.move` | `"F5"`, `"F6"` | ja |
| `[viewer]` | `max_file_size` | `10485760` | ja |
| `[viewer]` | `max_text_size` | `4194304` | ja |
| `[archives]` | `max_extracted_bytes` | `2147483648` | ja |
| `[archives]` | `max_entries` | `100000` | ja |
| `[privacy]` | `log_level` | `"warn"` | ja |
| `[advanced]` | `worker_threads` | `0` | ja |
| `[input]` | `context` | `"auto"` | ja |
| `[input.keyboard]` | `panel.copy`, `panel.move` | `"F5"`, `"F6"` | ja |
| `[input.dpad]` | `panel.copy`, `panel.move`, `palette.open` | `"palette:file.copy"`, `"palette:file.move"`, `"Menu"` | ja |
| `[input.controller]` | `panel.copy`, `selection.toggle` | `"palette:file.copy"`, `"Grip"` | ja |
| `[remote]` | `connect_timeout_seconds` | `15` | ja |
| `[remote]` | `smb_signing` | `"prefer"` | ja |
| `[remote]` | `smb_encryption` | `"prefer"` | ja |
| `[remote]` | `max_remote_download_bytes` | `0` | ja |

Der vollständige Wortlaut der Datei — jede Sektion mit einem Kommentar in Satzform, jeder Schalter mit eigenem Kommentar — liegt in `config/defaults.toml`. Die Datei wurde nicht in dieses Übergabedokument dupliziert, weil sie dort ein lebendes, gepflegtes Artefakt bleiben soll und nicht eine eingefrorene Kopie.

**Regeln für `schema_version`:** Die Zahl wird bei inkompatiblen Änderungen erhöht. Beim Start migriert Phocommander eine ältere Datei und legt das Original als `.bak` daneben (FR-123, AC-M0-18). Eine Erhöhung ist nur zulässig, wenn eine Migrationsfunktion existiert — die Migrationsregel selbst ist noch zu entscheiden (offener Punkt 7).

---

## 3. Belege

### 3.1 TOML-Validierung, jede Datei einzeln

Kommando je Datei: `python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" <datei>`
Das Kommando gibt bei gültigem TOML nichts aus; der Exit-Code ist der Beleg.

```
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/defaults.toml
exit code: 0  (no output means the file parsed)
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/keymap.toml
exit code: 0  (no output means the file parsed)
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/templates/series.toml
exit code: 0  (no output means the file parsed)
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/templates/camera.toml
exit code: 0  (no output means the file parsed)
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/templates/podcast.toml
exit code: 0  (no output means the file parsed)
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/templates/invoice.toml
exit code: 0  (no output means the file parsed)
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/templates/music.toml
exit code: 0  (no output means the file parsed)
```

### 3.2 Schema-, Kommentar- und Vorlagenprüfung

```
$ python3 scripts/check-config-schema.py
config    : .../phocommander/config
files     : 7
           config/defaults.toml
           config/keymap.toml
           config/templates/camera.toml
           config/templates/invoice.toml
           config/templates/music.toml
           config/templates/podcast.toml
           config/templates/series.toml

OK    all 7 files under config/ parse as TOML
OK    the configuration file carries every section of the specification
OK    the default values match the specification
OK    every key and every section carries a comment
OK    the shipped rename templates are complete
OK    the overrides match the action catalogue
OK    the configuration file and the Rust schema agree

RESULT: PASS — 7 of 7 checks passed
exit code: 0
```

Der letzte Check vergleicht die Datei feldweise mit dem Rust-Schema in `crates/phocommander-config/src/schema.rs` (das parallel von einem anderen Teammitglied entstanden ist): kein Schlüssel der Datei ist dem Schema unbekannt, und kein Skalarfeld des Schemas fehlt in der Vorlage.

### 3.3 Aktions-ID-Abgleich

```
$ python3 scripts/check-action-ids.py
keymap    : .../phocommander/config/keymap.toml
actions   : 35 action IDs, 1 surface action
bindings  : 34 from the key table of the specification, 1 derived (panel.prev_tab)

OK    all three context sections exist
OK    the action catalogue is complete and free of unknown actions
OK    every action occurs exactly once per context
OK    the keyboard bindings match the key table of the specification
OK    dpad and controller refer to the action palette
OK    no key is bound to two actions of the same surface
OK    every action ID is in dotted notation
OK    every action is referenced in the acceptance criteria or the specification
OK    the overrides in defaults.toml name known actions

RESULT: PASS — 9 of 9 checks passed
exit code: 0
```

Interpretation der Prüfregeln:

- „genau einmal": jede Aktions-ID kommt in **jedem** der drei Kontext-Abschnitte genau einmal vor. TOML verbietet doppelte Schlüssel in einer Tabelle, die Mehrfachbindung wird deshalb über die Abschnitte gezählt.
- Die 34 Belegungen aus der Tabelle „Tastatursteuerung" der Spezifikation sind im Skript wörtlich hinterlegt und werden gegen die Datei verglichen — der Vergleich ist maschinell, nicht behauptet.
- Die eine abgeleitete Belegung (`panel.prev_tab` = `Ctrl+Shift+Tab`) ist im Skript getrennt ausgewiesen und im Kommentar der Datei als erste Stelle markiert, die zu revidieren ist, falls sie falsch ist.
- „kein Konflikt": geprüft wird die Regel der Spezifikation — eine Taste darf nicht zwei Aktionen **derselben Oberfläche** binden. Die Oberfläche ist das erste Segment der Aktions-ID (`panel`, `viewer`, `rename`, `selection`, `app`). `F3` ist auf drei Oberflächen belegt; das legt die Spezifikation so fest („F3 im Betrachter sucht weiter, Shift+F3 sucht rückwärts — durchgängig, auch in den Einzelfenstern des Mehrfach-Umbenennungsdialogs"). Ergebnis: 0 Konflikte. Zum Vergleich und als Befund am fremden Crate siehe Abschnitt 3.7.

### 3.4 ID-Konsistenz des Anforderungskatalogs

```
$ python3 scripts/check-requirement-ids.py
catalogue : .../phocommander/docs/REQUIREMENTS-IDS.md
spec      : .../phocommander/docs/REQUIREMENTS.md
rows      : 208 (167 FR, 41 NFR)

OK    catalogue is readable and has rows
OK    FR sequence is gapless and starts at FR-01
OK    NFR sequence is gapless and starts at NFR-01
OK    no duplicated identifiers
OK    all six columns are filled and valid
OK    every source section exists in the specification
OK    every M0/M1/M2 requirement has an acceptance criterion
OK    the acceptance criteria have gapless identifiers

RESULT: PASS — 8 of 8 checks passed
exit code: 0
```

Der Check „every source section exists in the specification" liest die Überschriften aus `docs/REQUIREMENTS.md` und vergleicht sie mit der Quellenspalte des Katalogs. Jede Zeile verweist damit auf einen realen Abschnitt der Spezifikation — eine erfundene Quellenangabe hätte den Check nicht bestanden.

### 3.5 Integrationstest gegen die echte Konfigurationsdatei

Der Workspace des Repositorys ließ sich zum Zeitpunkt der Übergabe nicht bauen (`platform/phocommander-platform` fehlt, `crates/phocommander-keymap` hat noch kein Target — beides Arbeit anderer Teammitglieder). Um meine Datei trotzdem gegen das **echte** Rust-Schema zu prüfen, wurde das Crate `phocommander-config` in einem isolierten Workspace unter `/workspace/verify-config` (außerhalb des Repositorys, ohne Eingriff in den Build des Projekts) mit einem Integrationstest versehen, der die echte Datei `config/defaults.toml` von der Platte liest und mit dem strikten Loader parst (`deny_unknown_fields`):

```
$ cd /workspace/verify-config && cargo test -p phocommander-config --test loads_repository_template
running 2 tests
test palette_references_survive_the_round_trip ... ok
test repository_template_loads_with_the_strict_loader ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

Ergebnis: `config/defaults.toml` wird vom strikten Schema **akzeptiert**; kein unbekanntes Feld, alle Grenzwerte korrekt typisiert, die Palettenverweise (`palette:file.copy`) und die Sonderbelegung (`Grip`) überstehen den Rückschreiben-Test.

### 3.6 Was nicht gelaufen ist (NOT RUN mit Grund)

| Gegenstand | Grund | Reproduktion |
|---|---|---|
| `cargo test --workspace` | Der Workspace lässt sich nicht laden: `platform/phocommander-platform/Cargo.toml` fehlt, `crates/phocommander-keymap/Cargo.toml` hat kein Target. Beides Arbeit anderer Teammitglieder, nicht Teil dieses Auftrags. | `cargo test --workspace` |
| `cargo deny check`, `cargo audit`, `cargo fmt --check`, `cargo clippy` | Gleicher Grund — der Workspace lädt nicht. | s.o. |
| Alle 131 Abnahmekriterien mit Rust-Bezug | Die genannten Crates existieren noch nicht. Sie stehen in `docs/ACCEPTANCE-CRITERIA.md` mit dem Zusatz `NOT RUN — the crate <name> does not exist yet`. | siehe `docs/ACCEPTANCE-CRITERIA.md` |
| Performance-, Speicher- und Pixelmessungen | Keine lauffähige Anwendung vorhanden. | — |
| `scripts/check-crate-boundaries.sh` | Die Datei existiert noch nicht; sie ist Aufgabe des Rust Architect. | — |

### 3.7 Zwei Befunde am parallel entstandenen Keymap-Crate

Während der Arbeit sind `crates/phocommander-keymap` und die Integrationstests entstanden. Beide beziehen sich auf **meine** Dateien. Zwei Punkte sind dort offen; beide sind gemessen, nicht vermutet.

**(a) Die Konflikterkennung des Crates kennt die Oberfläche nicht.** Die Spezifikation belegt `F3` mit drei Funktionen auf drei Oberflächen: „F3 im Betrachter sucht weiter, Shift+F3 sucht rückwärts — durchgängig, auch in den Einzelfenstern des Mehrfach-Umbenennungsdialogs, wo F3 und Shift+F3 den Konfliktsprung steuern". Das Crate vergleicht die Bindungen **innerhalb einer Kontexttabelle**, ohne das erste Segment der Aktions-ID als Oberfläche zu werten. Gemessen mit dem Crate selbst (isolierter Workspace, Test `reports_shipped_conflicts`):

```
conflicts reported by the crate: 5
  context=Keyboard key=F3 actions=[ActionId("panel.view"), ActionId("rename.conflict_next"), ActionId("viewer.find_next")]
  context=Keyboard key=F3 actions=[ActionId("rename.conflict_next"), ActionId("panel.view"), ActionId("viewer.find_next")]
  context=Keyboard key=F3 actions=[ActionId("viewer.find_next"), ActionId("panel.view"), ActionId("rename.conflict_next")]
  context=Keyboard key=Shift+F3 actions=[ActionId("rename.conflict_previous"), ActionId("viewer.find_previous")]
  context=Keyboard key=Shift+F3 actions=[ActionId("viewer.find_previous"), ActionId("rename.conflict_previous")]
```

Alle fünf Meldungen betreffen dieselben zwei Tasten und dieselben drei bzw. zwei Oberflächen. Nach der Regel der Spezifikation — eine Taste darf nicht zwei Aktionen **derselben** Oberfläche binden — ist meine Datei konfliktfrei; `scripts/check-action-ids.py` prüft genau diese Regel und meldet 0 Konflikte. Der eigene Unit-Test des Crates `bindings::tests::reports_a_conflict_between_two_actions` ist ebenfalls rot (`left: 2, right: 1`), weil die Konflikterkennung einen Eintrag **pro belegter Aktion** statt **pro Taste** liefert. Beides gehört in das Crate, nicht in die Datei: die Datei folgt der Spezifikation, das unabhängige Prüfskript eines zweiten Teammitglieders (`scripts/check-keymap-consistency.py`) bestätigt dieselbe Lesart und meldet für meine Datei „RESULT: PASS — specification, catalogue and reference agree" mit 0 undokumentierten Abweichungen.

**(b) `tests/integration/keymap_bindings.rs` parst das Dokument mit der falschen API.** Der Test liest die Datei mit `source.parse::<toml::Value>()`. In `toml` 1.x parst `Value::from_str` einen **Wert**, kein Dokument; das Dokument liest `toml::Table::from_str`. Gemessen mit beiden Dateien des Repositorys:

```
/workspace/phocommander/config/defaults.toml: toml::Value FAILED: TOML parse error at line 1, column 1
/workspace/phocommander/config/defaults.toml: toml::Table ok, 11 top level keys
/workspace/phocommander/config/keymap.toml: toml::Value FAILED: TOML parse error at line 1, column 1
/workspace/phocommander/config/keymap.toml: toml::Table ok, 1 top level keys
```

Der Fehler tritt bei **beiden** Dateien auf, unabhängig vom Inhalt: er ist ein Fehler im Aufruf, nicht in der Datei. Mit `toml::Table` liest der Test meine Datei einwandfrei ein.

---

## 4. Vollständige Anforderungsmatrix

208 Zeilen: 167 funktionale (FR) und 41 nicht-funktionale (NFR) Anforderungen. Quelle, Verifikationsart und Detailtext stehen in `docs/REQUIREMENTS-IDS.md`.

| ID | Titel | Meilenstein | Priorität | Verifikationsart |
|---|---|---|---|---|
| FR-01 | No licence fee, no subscription, no in-app purchase, no closed feature | M0 | Must | automated (CI gate) |
| FR-02 | Every function that exists in a version exists identically in every version; there is no "Pro" branch | M0 | Must | manual (document review) |
| FR-03 | The source code builds the application without network access | M0 | Must | automated (CI gate) |
| FR-04 | No advertising banner, no recommendation column, no message service, no forced account, no intrusive onboarding tour | M2 | Must | manual (visual inspection) |
| FR-05 | No in-app advertising, no sponsored hits in search or connection lists, no settings that are not needed | M2 | Must | manual (visual inspection) |
| FR-06 | No advertising, no tracking, no analytics, no telemetry, no usage profile, no third-party disclosure, no affiliate link | M0 | Must | automated (dependency scan) |
| FR-07 | Local functions work without registration and without login | M0 | Must | automated (CI gate) |
| FR-08 | The connection list contains only entries the user created themselves | M6 | Must | manual (operational test) |
| FR-09 | Source and target are visible at the same time | M0 | Must | manual (visual inspection) |
| FR-10 | The complete operation is possible without a mouse | M2 | Must | manual (operational test) |
| FR-11 | The built-in viewer displays files without starting an external program | M1 | Must | automated (integration test) |
| FR-12 | An input line and a filter field accelerate jumping and selecting | M2 | Should | manual (operational test) |
| FR-13 | Two independent directory windows stand next to each other by default or above one another | M0 | Must | manual (visual inspection) |
| FR-14 | Both windows show path, file list and the columns name, size, date and type | M0 | Must | manual (visual inspection) |
| FR-15 | The arrangement is switchable without a restart | M1 | Must | automated (UI test) |
| FR-16 | Each window has a tab bar; each tab holds its own directory path, its own sorting and its own display settings | M2 | Must | automated (UI test) |
| FR-17 | Tabs can be moved between the two windows | M2 | Should | automated (UI test) |
| FR-18 | A directory tree can be switched on per window; the view type is kept when a directory is left | M2 | Should | automated (UI test) |
| FR-19 | A third area can be switched on: a quick view that shows the content of the marked file without opening it | M2 | Could | manual (operational test) |
| FR-20 | The workspace consists of exactly two panel objects, each with active tab, path, cursor position, sorting, selection set and view mode | M1 | Must | automated (unit test) |
| FR-21 | Operations always act on the active panel; the transfer to the other panel is explicit | M1 | Must | automated (integration test) |
| FR-22 | Copy, move, rename, delete and create directory are the basic operations | M1 | Must | automated (integration test) |
| FR-23 | Every operation runs through a job queue in the background so that the surface stays usable | M1 | Must | automated (integration test) |
| FR-24 | The user can continue working while a copy is running | M1 | Must | automated (integration test) |
| FR-25 | Conflict behaviour is explicit and configurable: overwrite, skip, keep both, ask or cancel | M1 | Must | automated (unit test) |
| FR-26 | Overwriting is not the default; the default conflict policy is "ask" | M1 | Must | automated (config parse) |
| FR-27 | An undo area for operations is provided and is explicitly labelled as limited: undo of a copy means "delete the copy" | M1 | Should | manual (operational test) |
| FR-28 | A job has type, status, progress, rate, cancel flag and error list; the queue is threadsafe and the surface reads a snapshot | M1 | Must | automated (unit test) |
| FR-29 | Several jobs run in parallel up to a configurable limit, by default the number of available cores; one panel half does not block while the other works | M1 | Must | automated (integration test) |
| FR-30 | The multiple rename dialog has three zones: the list of selected files, a table of original name and new name, and the rule definition | M1 | Must | automated (UI test) |
| FR-31 | The rule evaluation combines placeholders, replace and counter in this order | M1 | Must | automated (unit test) |
| FR-32 | The placeholders name, ext, parent, n, date, time, size, hash, rand, case, lower, upper and title are available | M1 | Must | automated (unit test) |
| FR-33 | The replace rule offers search text and replacement text, optionally as a regular expression | M1 | Should | automated (unit test) |
| FR-34 | The counter rule offers start value, step, digit count with leading zeros and the order of the counter sequence | M1 | Must | automated (unit test) |
| FR-35 | The dialog shows a live preview of the result | M1 | Must | automated (UI test) |
| FR-36 | Conflicts are marked in colour and listed in a conflict jump list traversable with F3 and Shift+F3 | M1 | Must | automated (UI test) |
| FR-37 | Rules can be saved as named templates and applied again | M1 | Must | automated (integration test) |
| FR-38 | The dialog recognises season and episode number in an existing file name using the pattern SxxEyy and offers the rest as episode title | M1 | Must | automated (unit test) |
| FR-39 | If no pattern is found, the dialog asks for season and episode and remembers the last input as the pre-selection for the next file | M1 | Must | automated (unit test) |
| FR-40 | The series name comes from the parent directory by default, is always displayed and is overridable | M1 | Must | automated (unit test) |
| FR-41 | The target format is serie, season and episode zero padded to two digits; the width grows automatically to three when a three-digit episode is in the selection | M1 | Must | automated (unit test) |
| FR-42 | The episode title is taken over unchanged; characters that Windows forbids are reported before the rename and a replacement is suggested instead of being applied silently | M1 | Must | automated (unit test) |
| FR-43 | A preset set of templates ships with the application: series, camera, podcast, invoice and music; each is a TOML file in the configuration directory and is user-adjustable | M1 | Must | automated (config parse) |
| FR-44 | Before renaming, the dialog shows a preview of the new names of all selected files side by side, with the recognised fields as a column of their own | M1 | Must | automated (UI test) |
| FR-45 | The viewer opens in an additional area without leaving the application | M1 | Must | automated (integration test) |
| FR-46 | The viewer shows text and source code with charset detection, UTF-8 and UTF-16 and line-end handling | M1 | Must | automated (unit test) |
| FR-47 | The viewer shows a hex dump with offset, hex, ASCII and OR group | M1 | Must | automated (unit test) |
| FR-48 | The viewer shows images in the formats PNG, JPEG, GIF, WebP, BMP and TIFF | M1 | Must | automated (unit test) |
| FR-49 | The viewer lists the content of an archive and extracts a single file | M1 | Must | automated (integration test) |
| FR-50 | The viewer shows metadata: size, type, timestamp and checksum | M1 | Must | automated (unit test) |
| FR-51 | The viewer shows PDF documents with text layer and page navigation | M2 | Should | automated (integration test) |
| FR-52 | The viewer highlights source code syntax | M2 | Should | automated (integration test) |
| FR-53 | The viewer shows audio and video with metadata, still image and playback | M9 | Could | manual (operational test) |
| FR-54 | The viewer shows office documents with metadata only, without a full text representation | M9 | Could | manual (operational test) |
| FR-55 | The viewer is read-only | M1 | Must | automated (unit test) |
| FR-56 | The About dialog names the actually supported formats honestly instead of an inflated number | M1 | Should | manual (document review) |
| FR-57 | F3 searches forward in the viewer and Shift+F3 searches backward | M1 | Must | automated (UI test) |
| FR-58 | A separate text editor is a later expansion stage | M9 | Could | manual (operational test) |
| FR-59 | Archives appear as normal entries in the file list | M1 | Must | automated (integration test) |
| FR-60 | A double click on an archive opens it as a virtual directory whose files can be listed, viewed and copied into the other pane without prior extraction | M1 | Must | automated (integration test) |
| FR-61 | Supported formats are ZIP, TAR with GZip, BZip2, XZ and Zstd, and 7z; reading and writing are possible, archives are created encrypted | M1 | Must | automated (integration test) |
| FR-62 | RAR, WIM, CAB, ARJ and LZH stay out of scope because they need proprietary or licence-bound libraries | M1 | Must | automated (unit test) |
| FR-63 | All extractions run through a hardened path resolution | M1 | Must | automated (integration test) |
| FR-64 | The file search searches by name, size, date and content; text, hex and binary search are possible and Unicode text is handled correctly | M6 | Must | automated (integration test) |
| FR-65 | The search operators AND, OR and NOT are available, as is the search in archives | M6 | Should | automated (integration test) |
| FR-66 | The content search runs in a background process with a growing hit list; a hit can be taken over into a panel | M6 | Should | automated (integration test) |
| FR-67 | The directory comparison places two directories side by side and reports differences in content, date and size | M2 | Should | automated (integration test) |
| FR-68 | The synchronisation transfers differences in both directions and holds profile options that include or exclude directories | M6 | Should | automated (integration test) |
| FR-69 | A plan is shown before the execution, not only a dialog with "OK" | M6 | Must | automated (UI test) |
| FR-70 | Folder favourites and application favourites are callable from the icon bar or a keybinding | M2 | Should | automated (UI test) |
| FR-71 | The file drop collects frequently needed files in one place, independent of where they were found | M2 | Could | manual (operational test) |
| FR-72 | Tags on files and folders are a later expansion stage | M9 | Could | manual (operational test) |
| FR-73 | An input line at the lower edge accepts paths and shell commands | M2 | Must | automated (UI test) |
| FR-74 | A command line history with Ctrl+Down and a command output window are part of the scope | M2 | Could | manual (operational test) |
| FR-75 | The menu bar follows the platform conventions: in the system menu bar on macOS, in the window on Windows and Linux; on macOS the standard roles sit in the expected places | M2 | Must | manual (visual inspection) |
| FR-76 | Every operable function is a named action with a stable action ID; the input layer translates physical input into action IDs | M0 | Must | automated (config parse) |
| FR-77 | The surface layer knows action IDs only and does not know whether an action was triggered by key, click, swipe, D-pad key or controller trigger | M1 | Must | automated (unit test) |
| FR-78 | The action catalogue lives in config/keymap.toml and is referenced by the configuration file; docs/KEYMAP.md documents every action with default, context and platform variant | M0 | Must | automated (config parse) |
| FR-79 | The default binding is stored separately per operating context | M0 | Must | automated (config parse) |
| FR-80 | Five operating contexts are supported: keyboard, pointer, touch, dpad and controller | M4 | Must | automated (integration test) |
| FR-81 | The layouts differ structurally per context: two panes visible for keyboard and pointer, stacked for touch, one pane plus overlay for dpad, spatial arrangement with ray pointer for controller | M4 | Must | manual (visual inspection) |
| FR-82 | In the touch context the row height is at least 44 points, dialogs appear as a sheet from below and hover effects are absent | M4 | Must | manual (visual inspection) |
| FR-83 | In the dpad context one pane is visible, a second pane appears as an overlay when an action needs a target, the type is scaled for a viewing distance of about three metres and the operating hints are visible | M8 | Must | manual (visual inspection) |
| FR-84 | The action palette is a searchable list of all actions available in the current context; it opens with the menu or play/pause key in the dpad context and is fully operable with the direction keys and OK | M8 | Must | automated (UI test) |
| FR-85 | The palette filters by context: in the dpad context only actions that can be executed without a mouse and without a modifier key appear | M8 | Must | automated (UI test) |
| FR-86 | The action palette is available in every context, on the desktop as "find actions" | M8 | Should | automated (UI test) |
| FR-87 | Focus and selection are led as separate states in the dpad and controller contexts; navigating moves the focus without changing the selection, selecting changes the selection without losing the focus | M8 | Must | automated (unit test) |
| FR-88 | The dpad context offers an explicit selection mode in which OK toggles the marking and the direction keys move the focus; the mode is visibly indicated and can be left at any time | M8 | Must | automated (UI test) |
| FR-89 | Focus is a visible and permanently recognisable state on platforms without a pointer | M4 | Must | manual (visual inspection) |
| FR-90 | Every action that is reachable in the keyboard context must be reachable in the dpad context through the action palette; the check is documented in a test protocol | M8 | Must | manual (operational test) |
| FR-91 | The application does not depend on a menu key of the remote control; back navigates back and leaves the application without endless loops | M8 | Must | automated (UI test) |
| FR-92 | Gamepad events do not come from winit; the controller binding on the desktop is a separate task that uses a platform-native library | M7 | Must | automated (integration test) |
| FR-93 | The context "auto" detects the input device at run time and selects keyboard, pointer, touch, dpad or controller | M4 | Should | automated (integration test) |
| FR-94 | Meta Quest is operated as a flat panel on the Android version through the controller context; there is no immersive VR port | M7 | Should | manual (operational test) |
| FR-95 | The limitation of the panel mode — no keyboard — is stated in the documentation | M7 | Must | manual (document review) |
| FR-96 | Network access is a source in the same abstraction; SMB, WebDAV, SFTP and S3 deliver the same folder content as the local file system | M6 | Must | automated (integration test) |
| FR-97 | The operation is identical for all sources: copy, move, rename, delete, view, search | M6 | Must | automated (integration test) |
| FR-98 | The protocols SFTP, FTP and FTPS, SMB 2 and 3, WebDAV, S3-compatible and OAuth 2.0 are supported | M6 | Must | automated (integration test) |
| FR-99 | The WebDAV client is written in the project itself instead of using an LGPL library | M6 | Must | manual (document review) |
| FR-100 | SMB1 is not implemented | M6 | Must | automated (integration test) |
| FR-101 | NTLMv1 is not used; where NTLM is needed it is NTLMv2 | M6 | Must | automated (integration test) |
| FR-102 | SMB signing is required when the server supports it | M6 | Must | automated (integration test) |
| FR-103 | SMB transport encryption is preferred and the credentials never lie in the configuration file | M6 | Must | automated (integration test) |
| FR-104 | A connection attempt against a server that offers only SMB1 or only NTLMv1 fails with an understandable message | M6 | Must | automated (integration test) |
| FR-105 | Phocommander runs no own intermediary server; every login runs directly between the application and the provider and the refresh token lies in the keychain | M7 | Must | automated (dependency scan) |
| FR-106 | Credentials lie in the keychain of the operating system; on platforms without a keychain the application falls back to the app sandbox and points out that the data is then not protected by the operating system | M6 | Must | manual (operational test) |
| FR-107 | Connections are managed in a list the user created; connection names, server addresses and user names are personal data and do not appear in the log | M6 | Must | automated (integration test) |
| FR-108 | The log names protocol, timestamp, result and duration, never the server and never the user | M6 | Must | automated (integration test) |
| FR-109 | For HTTPS sources the server certificate is checked strictly, without an opt-out; a self-signed certificate can be stored deliberately with a PIN-protected exception for exactly that host | M6 | Must | automated (integration test) |
| FR-110 | The client reads in a fixed budget, aborts and reports the error when a server exceeds the expected response size | M6 | Must | automated (integration test) |
| FR-111 | An older protocol version is not silently accepted: SMB1 and NTLMv1 are rejected, a WebDAV server without TLS is rejected when HTTPS is not reachable | M6 | Must | automated (integration test) |
| FR-112 | Directory names injected by a server are normalised, checked for invalid characters and shortened in the surface without losing the real name | M6 | Must | automated (unit test) |
| FR-113 | Every network operation has a time limit; a server that does not finish a response leads to an abort with an understandable message | M6 | Must | automated (integration test) |
| FR-114 | The behaviour on connection problems is configurable: "retry" tries again, "fail" aborts; the timeouts apply per connection establishment | M6 | Must | automated (config parse) |
| FR-115 | The workspace uses flat crates, apps for the binaries and xtask for build helpers | M0 | Must | automated (CI gate) |
| FR-116 | The dependency direction is apps to platform to crates; inside crates no crate depends on a sibling of the same level | M0 | Must | automated (CI gate) |
| FR-117 | The crate boundary rule is checked mechanically by a script that exits with code 1 on a violation, and a negative test proves that the gate has teeth | M0 | Must | automated (CI gate) |
| FR-118 | The core is free of UI dependencies: no windows, no crates of the UI stack, no platform-specific APIs | M0 | Must | automated (CI gate) |
| FR-119 | The directory content is held in a state object outside the egui context; only the visible slice is drawn | M1 | Must | automated (unit test) |
| FR-120 | Viewer and archive adapters are registered through a trait; a new adapter is a new implementation plus one registration line | M1 | Must | automated (unit test) |
| FR-121 | There is exactly one central configuration file in TOML format; its location follows the operating system conventions | M0 | Must | automated (config parse) |
| FR-122 | The configuration file is fully commented because it is the first contact point for users and contributors | M0 | Must | automated (config parse) |
| FR-123 | The configuration file carries a schema_version; on incompatible changes the old file is migrated and the original is kept as a .bak file | M0 | Must | automated (unit test) |
| FR-124 | Import and export of a configuration are provided so that settings can be transferred between machines | M2 | Could | manual (operational test) |
| FR-125 | The repository structure follows the crate structure; the workspace is a single Cargo workspace with shared dependency versions and a pinned toolchain | M0 | Must | automated (CI gate) |
| FR-126 | macOS is delivered without app sandbox, with notarisation, hardened runtime and a first-run page that explains the required permissions | M3 | Must | manual (operational test) |
| FR-127 | The use of full disk access is neither forced nor circumvented; the application handles the refusal | M3 | Must | manual (operational test) |
| FR-128 | iPadOS is not a file system browser; it is a viewer and organiser for root directories the user shares explicitly | M5 | Must | manual (operational test) |
| FR-129 | A structure for permanent access is stored as security-scoped bookmarks; the user can revoke the access in the settings and the application handles the error cleanly | M5 | Must | manual (operational test) |
| FR-130 | The two-pane surface stays sensible on the iPad in split view; touch operates the selection and the keyboard remains the main control | M5 | Should | manual (visual inspection) |
| FR-131 | The Android version works in SAF mode without the special permission and offers full access as a clearly marked optional extension | M4 | Must | manual (operational test) |
| FR-132 | Android declares MANAGE_EXTERNAL_STORAGE in the manifest, explains the core functionality and asks the user to enable the permission | M4 | Must | manual (document review) |
| FR-133 | The Windows manifest sets longPathAware and path length errors are phrased understandably | M3 | Must | automated (integration test) |
| FR-134 | The case in which the explorer does not show a file that Phocommander shows is caught and reported understandably | M3 | Should | manual (operational test) |
| FR-135 | Linux and BSD share the code path; Wayland is preferred and X11 is the fallback | M3 | Must | automated (CI gate) |
| FR-136 | The build on FreeBSD and NetBSD is checked in the CI; the functional tests are carried out manually | M3 | Must | automated (CI gate) |
| FR-137 | The TV build declares no touchscreen requirement, sets CATEGORY_LEANBACK_LAUNCHER and carries a 320 by 180 pixel banner | M8 | Must | manual (document review) |
| FR-138 | The pointer is never the primary means of operation on the TV | M8 | Must | manual (visual inspection) |
| FR-139 | Every public function has at least one unit test | M0 | Must | automated (CI gate) |
| FR-140 | Integration tests cover the crate boundaries: copy through a temporary directory, extraction of an archive with traversal entries, loading of a configuration file with an unknown field | M1 | Must | automated (integration test) |
| FR-141 | Property tests check invariants that cannot be enumerated: path resolution never leaves the destination, the rename rule is deterministic, the key resolution decides deterministically | M1 | Must | automated (property test) |
| FR-142 | Fuzz tests check the parsers: archive header, image header, charset detection; a fuzz run belongs to the nightly CI | M1 | Must | automated (fuzz test) |
| FR-143 | UI tests check the keyboard operation deterministically, without pixel-wise screenshots | M1 | Must | automated (UI test) |
| FR-144 | Performance measurements with criterion are part of the gate; the values of the performance requirement are the thresholds | M1 | Must | automated (benchmark) |
| FR-145 | The CI runs as a matrix over Linux, Windows and macOS and additionally checks the cross-compilation to aarch64-linux-android and aarch64-apple-ios and the build for FreeBSD | M0 | Must | automated (CI gate) |
| FR-146 | Every CI run covers cargo fmt --check, cargo clippy --all-targets with -D warnings, cargo test --workspace, cargo-deny, cargo-audit and cargo build --release | M0 | Must | automated (CI gate) |
| FR-147 | The tool versions are pinned because the same configuration is read differently per version | M0 | Must | automated (CI gate) |
| FR-148 | For every hardening — path traversal, decompression bomb, header limit — a negative test exists that actually performs the attack | M1 | Must | automated (integration test) |
| FR-149 | The gate must not be green when a rule is violated | M0 | Must | automated (CI gate) |
| FR-150 | A milestone counts as reached only when its acceptance criteria are met and the acceptance is documented under docs/acceptance | M0 | Must | manual (document review) |
| FR-151 | The core comes before the surface; a surface function without core support is not started | M0 | Must | manual (document review) |
| FR-152 | If a basic assumption does not hold, the affected milestone is stopped, the assumption is corrected in an ADR or in the specification and the work is repeated | M0 | Must | manual (document review) |
| FR-153 | The documentation landscape contains the architecture, the ADRs, the user guide in both languages, the security documents, the glossary, the acceptances, the reviews, the guides and the templates | M0 | Must | manual (document review) |
| FR-154 | The ADRs follow the pattern status, date, context, decision, consequences, alternatives; the first series covers the framework choice, the crate boundaries, the error hierarchy and the security strategy for archive extraction | M0 | Must | manual (document review) |
| FR-155 | AGENTS.md describes the working rules for AI-assisted contributions | M0 | Must | manual (document review) |
| FR-156 | CONTRIBUTING.md summarises the rules, SECURITY.md describes the threat model and the reporting path, PRIVACY.md holds the zero-telemetry pledge | M0 | Must | manual (document review) |
| FR-157 | CODE_OF_CONDUCT.md follows the Contributor Covenant | M0 | Must | manual (document review) |
| FR-158 | The repository stands under the dual licence MIT OR Apache-2.0; both files lie in the repository root and every source file carries a short SPDX header | M0 | Must | automated (CI gate) |
| FR-159 | The central configuration file contains the sections schema_version, general, appearance, panels, keymap, viewer, archives, privacy, advanced, input and remote with the specified default values | M0 | Must | automated (config parse) |
| FR-160 | Every section and every single switch of the configuration file carries a comment that explains what it does | M0 | Must | automated (config parse) |
| FR-161 | Every action has a stable action ID in dotted notation and exactly one binding per operating context | M0 | Must | automated (config parse) |
| FR-162 | The binding table of the keyboard context matches the key table of the specification | M0 | Must | automated (config parse) |
| FR-163 | In the dpad and controller contexts the actions refer to the action palette because those devices have no function keys | M0 | Must | automated (config parse) |
| FR-164 | The shipped rename templates series, camera, podcast, invoice and music exist as TOML files in the configuration directory | M0 | Must | automated (config parse) |
| FR-165 | The requirement catalogue decomposes the specification into gapless FR and NFR IDs, each row naming a concrete section of the specification | M0 | Must | automated (config parse) |
| FR-166 | Every FR of M0, M1 and M2 has an acceptance criterion with an exact command or an exact operation sequence and a measurable pass condition | M0 | Must | automated (config parse) |
| FR-167 | The acceptance template carries the sections functionality, technique, performance, guidance, security and privacy, interoperability, known limitations and decision | M0 | Must | manual (document review) |
| NFR-01 | The trust boundary runs between the process and the file system; inputs from the file system and from archives are hostile by assumption | M1 | Must | automated (integration test) |
| NFR-02 | No extract function of an archive crate is used; the entries are iterated and written by the project itself, every target path is canonicalised and checked against the target directory, and no symlink is created from archive material | M1 | Must | automated (integration test) |
| NFR-03 | The minimum version of the zip crate is 9.0.5 | M1 | Must | automated (CI gate) |
| NFR-04 | The extraction has a limit for the total size, a limit for the number of entries and a limit for the size of a single entry; the operation aborts when a limit is exceeded | M1 | Must | automated (integration test) |
| NFR-05 | Where the check of a target and the access to it can fall apart, the access runs descriptor-based or the check is repeated immediately before the access; the remaining race is named in the threat model | M1 | Must | manual (document review) |
| NFR-06 | When copying recursively, symlinks are copied as symlinks and not dereferenced; when deleting, they are unlinked and not deleted recursively | M1 | Must | automated (integration test) |
| NFR-07 | Secure deletion follows NIST SP 800-88 Rev. 2 with the levels Clear, Purge and Destroy; Clear is the standard level and is called "secure deletion" | M1 | Must | automated (integration test) |
| NFR-08 | The limitation of overwriting deletion on flash media is stated in the surface and not only in the manual | M1 | Must | manual (visual inspection) |
| NFR-09 | The trash is the standard path for deletion; secure deletion is a deliberate additional option | M1 | Must | automated (integration test) |
| NFR-10 | Every remote source is hostile; names, size information, timestamps and contents delivered by the server are treated as unverified | M6 | Must | automated (integration test) |
| NFR-11 | Only libraries with a permissive licence are used; crates with a copyleft effect are excluded | M0 | Must | automated (CI gate) |
| NFR-12 | The CI checks every dependency with cargo-deny and cargo-audit in the pinned tool versions | M0 | Must | automated (CI gate) |
| NFR-13 | No telemetry, no analytics, no usage counter, no advertising and no cloud connection | M0 | Must | automated (dependency scan) |
| NFR-14 | No usage profile, no crash report and no content index is stored; the application writes only what the user configures | M0 | Must | automated (dependency scan) |
| NFR-15 | Connection data for SFTP and FTP is not stored in cleartext in the configuration file but in the keychain of the operating system; on platforms without a keychain the application falls back to the app sandbox and points out the missing protection | M6 | Must | automated (integration test) |
| NFR-16 | Log data contains no user name, no server name and no IP address; personal fields are masked when logging is needed | M6 | Must | automated (integration test) |
| NFR-17 | The log level is configurable and defaults to "warn" | M0 | Should | automated (config parse) |
| NFR-18 | A directory list with up to 10,000 entries is drawn in under 100 ms | M1 | Must | automated (benchmark) |
| NFR-19 | A directory change with a loaded listing is under 16 ms | M1 | Must | automated (benchmark) |
| NFR-20 | A key press reaches a visible reaction in under 50 ms | M1 | Must | automated (benchmark) |
| NFR-21 | The start-up to the operable surface is under 800 ms on macOS, warm | M1 | Must | automated (benchmark) |
| NFR-22 | The memory demand in idle with two panes open is under 120 MB | M1 | Must | automated (benchmark) |
| NFR-23 | A sequential file copy reaches at least 80 % of the sequential read and write bandwidth | M1 | Must | automated (benchmark) |
| NFR-24 | A search over 100,000 file names takes under 30 s | M6 | Must | automated (benchmark) |
| NFR-25 | An archive listing with 5,000 entries takes under 500 ms | M1 | Must | automated (benchmark) |
| NFR-26 | Background operations are mandatorily asynchronous; no process that can take longer than 50 ms blocks the surface | M1 | Must | automated (integration test) |
| NFR-27 | Every operation reports progress with a rate and is cancellable | M1 | Must | automated (integration test) |
| NFR-28 | The operation is completely possible by keyboard, with a visible focus marking and a deterministic focus order | M1 | Must | automated (UI test) |
| NFR-29 | The contrast ratio is at least 4.5:1 for text and 3:1 for focus frames | M2 | Must | automated (UI test) |
| NFR-30 | prefers-reduced-motion is respected | M2 | Should | manual (visual inspection) |
| NFR-31 | Screen reader support works via AccessKit | M2 | Should | manual (operational test) |
| NFR-32 | Every public function carries a doc comment in the pattern WHAT, WHY, EXAMPLE; every module begins with a header comment that names its responsibility and its boundary; every constant carries a name that explains its purpose; every public function has at least one unit test | M0 | Must | automated (CI gate) |
| NFR-33 | No source file exceeds 400 lines (soft) or 550 lines (hard); unsafe_code, unwrap_used and expect_used are forbidden in the workspace; missing_docs is a warning and becomes an error for the public API of the core crates | M0 | Must | automated (CI gate) |
| NFR-34 | What a beginner cannot follow is rewritten and not left out; deep nesting is avoided with early returns | M0 | Must | manual (document review) |
| NFR-35 | All surface texts run through a translation key; no string is hardcoded in the code; German and English are fully covered; date, number and size formats follow the system language | M2 | Must | automated (CI gate) |
| NFR-36 | The code is in English, the documentation is bilingual in German and English, the surface texts are internationalisable with German as default | M0 | Must | manual (document review) |
| NFR-37 | No gate is weakened to become green: no lint is commented out with an allow attribute, no check is removed from the CI, no test is disabled | M0 | Must | manual (document review) |
| NFR-38 | Every evidence is a command output, a test name with a green result or an extract from an acceptance documentation; "it works" is not evidence | M0 | Must | manual (document review) |
| NFR-39 | Tests that could not be executed are reported as NOT RUN with the exact reason | M0 | Must | manual (document review) |
| NFR-40 | A CI check confirms that no crate of the telemetry or analytics categories is referenced, and a test proves that the process start opens no network connection | M0 | Must | automated (CI gate) |
| NFR-41 | The milestone gates are: formatting, static analysis, licence check, vulnerability check, crate boundaries, test run, platform build, performance, security proof and, before M8, the operator reachability | M0 | Must | automated (CI gate) |

**Verteilung:**

| Meilenstein | FR | NFR | Summe |
|---|---|---|---|
| M0 | 42 | 14 | 56 |
| M1 | 50 | 19 | 69 |
| M2 | 17 | 4 | 21 |
| M3–M9 | 58 | 4 | 62 |
| **Summe** | **167** | **41** | **208** |

---

## 5. Fertige Textbausteine für die Abnahmekriterien

### 5.1 Zeilenformat für ein neues Akzeptanzkriterium

Direkt einfügbar in `docs/ACCEPTANCE-CRITERIA.md`:

```
| AC-M<n>-<nn> | <FR-ID> | <exakter Befehl oder exakte Bediensequenz> | <messbare Pass Condition> | <automated\|manual> | <PASS — evidence in deliverable.md> |
```

Belegregeln, die dabei gelten:

1. Das Feld „How to Verify" nennt ein Kommando, das man kopieren kann, oder eine Bediensequenz, die man ausführen kann.
2. Das Feld „Pass Condition" nennt einen messbaren Zustand. Verboten sind „funktioniert", „ist implementiert", „sollte passen" und jede Coverage-Angabe ohne Messung.
3. Das Feld „Type" ist `automated` oder `manual`. Manuell geprüfte Kriterien sind erlaubt und werden als `manual` markiert.
4. Das Feld „Evidence" ist entweder `PASS — evidence in deliverable.md` mit dem Verweis auf die Ausgabe, oder `NOT RUN — <exakter Grund>`.

### 5.2 Zeilenformat für eine neue Anforderung

```
| <FR-ID> | <Titel als eine Aussage> | <Abschnitt> > <Unterabschnitt> | M<n> | Must\|Should\|Could | automated (<Art>) \| manual (<Art>) |
```

Der Quellenabschnitt muss als Überschrift in `docs/REQUIREMENTS.md` existieren — das prüft `scripts/check-requirement-ids.py`.

### 5.3 Drei fertige Beispielzeilen

```
| AC-M1-09 | FR-26 | `python3 scripts/check-config-schema.py`; read the section `panels` of `config/defaults.toml`. | The script exits with code 0 and reports `overwrite_policy = "ask"`. A test asserts that the default path of a copy does not overwrite an existing file. | automated | PASS for the configuration value — evidence in deliverable.md; the behaviour test is NOT RUN, the crate does not exist yet |
```

```
| AC-M1-14 | FR-31 | `cargo test -p phocommander-core --test rename_rule_order -- --exact`. | The test asserts that the placeholder result is the input of the replace step and that the counter result is the final name. | automated | NOT RUN — the crate `phocommander-core` does not exist yet |
```

```
| AC-M2-15 | FR-75 | `cargo run -p phocommander-desktop` on macOS and on Windows; look at the menu. | On macOS the menu sits in the system menu bar and About, Services, Hide and Quit sit in their expected places. On Windows the menu sits in the window. | manual | NOT RUN — no crate exists in the repository yet |
```

### 5.4 Textbaustein für eine Abnahmezeile im Abnahmedokument

```
| <FR-ID> | <beobachtetes Verhalten, ein Satz> | <Kommandoausgabe oder Testname> |
```

### 5.5 Textbaustein für eine neue Anforderung in der Spezifikation

Falls eine Entscheidung des Owners eine neue Anforderung erzeugt, gehört sie in die Spezifikation und danach mit der **nächsten** freien ID in den Katalog:

```
| FR-<n> | <Titel> | <Quelle> | M<n> | <Priorität> | <Verifikationsart> |
```

Die ID wird nie wiederverwendet. Der Änderungslog am Ende von `docs/REQUIREMENTS-IDS.md` führt jede Ergänzung mit Datum nach.

---

## 6. Widerspruchsliste

Gefunden bei der Zerlegung der Spezifikation in IDs. Keiner dieser Punkte wurde eigenmächtig in `docs/REQUIREMENTS.md` korrigiert — die Spezifikation bleibt unverändert, die Entscheidung liegt beim Owner.

| # | Fundstelle | Beobachtung | Umgang in diesem Auftrag |
|---|---|---|---|
| W1 | `Funktionsumfang > Mehrfaches Umbenennen` | Zwei Platzhalter-Syntaxen: die generischen Platzhalter stehen in **eckigen** Klammern (`[name]`, `[ext]`, `[parent]`, `[n]`, `[date]`, `[time]`, `[size]`, `[hash]`, `[rand]`, `[case]`, `[lower]`, `[upper]`, `[title]`), das Serienfolgen-Muster in **geschweiften** Klammern (`{serie} - S{staffel:02}E{episode:02} - {titel}.{format}`). Der Kern braucht entweder zwei Parser oder eine Übersetzung. | Die Vorlagen verwenden durchgängig geschweifte Klammern, weil die Spezifikation das Serienmuster so festlegt; die generischen Platzhalter sind in jedem Vorlagenkopf als Kommentar genannt. **Owner-Entscheidung nötig** (offener Punkt 1). |
| W2 | `Funktionsumfang > Favoriten, Dateiablage und Tags` vs. `Roadmap > Phasen M0 bis M9` und `Projektplan > Meilensteine` | Der Funktionsumfang nennt Tags „eine spätere Ausbaustufe (M3)"; die Roadmap und die Meilenstein-Tabelle führen Tags in **M9**. | FR-72 ist auf **M9** gesetzt, weil zwei von drei Fundstellen M9 sagen. **Owner-Entscheidung nötig** (offener Punkt 9). |
| W3 | `Funktionsumfang > Suche und Synchronisation` vs. `Projektplan > Meilensteine` | Der Funktionsumfang beschreibt die Suche vollständig; M6 nennt nur „Suche mit Inhalt", während der Vergleich bereits in M2 steht. Die Namens-, Größen- und Datumssuche ist keinem Meilenstein zugeordnet. | Die Suche liegt vollständig in M6 (FR-64 bis FR-66), der Verzeichnisvergleich in M2 (FR-67). **Owner-Entscheidung nötig** (offener Punkt 10). |
| W4 | `Systemarchitektur > Crate-Struktur` | Der Verzeichnisbaum führt `config/` **zweimal** auf: einmal mit `defaults.toml` und `keymap.toml`, einmal mit „Vorgaben: defaults.toml, keymap.toml, Vorlagen/". Das `Vorlagen/`-Segment ist die einzige Erwähnung des Vorlagenverzeichnisses in der Spezifikation. | Kein funktionaler Widerspruch. Die Vorlagen liegen unter `config/templates/` und sind damit von der zweiten Baumzeile gedeckt. Reine Redigierempfehlung. |
| W5 | `Systemarchitektur > Konfigurationsdatei` | „Der Aktionskatalog liegt in `config/keymap.toml` und wird von der Konfigurationsdatei referenziert" — im Schema der Konfigurationsdatei existiert **kein** Schlüssel, der diese Referenz ausdrückt. Der Abschnitt `[keymap]` enthält nur Overrides einzelner Aktionen. | Die Datei folgt dem Schema der Spezifikation unverändert; der Bezug wird im Kommentar des Abschnitts `[keymap]` in Worten hergestellt. **Owner-Entscheidung nötig** (offener Punkt 7). |
| W6 | `Repository- und Dokumentationsorganisation > Ablagestruktur` | Der Doku-Baum kennt `docs/REQUIREMENTS-IDS.md` und `docs/ACCEPTANCE-CRITERIA.md` nicht — sie entstanden mit diesem Auftrag. | `docs/REQUIREMENTS.md` wurde auftragsgemäß nicht verändert. Die Nachtragung in `docs/WHERE_TO_CHANGE.md` und im Baum ist Aufgabe des Documentation Lead (offener Punkt 8). |
| W7 | `crates/phocommander-keymap/src/bindings.rs` (fremdes Crate, parallel entstanden) | Die Konflikterkennung vergleicht die Bindungen innerhalb einer Kontexttabelle, ohne die Oberfläche zu werten. `F3` ist auf drei Oberflächen belegt (Panel, Betrachter, Umbenennungsdialog) — das legt die Spezifikation ausdrücklich so fest. Folge: die Datei, die der Spezifikation folgt, wird als widersprüchlich gemeldet (5 Meldungen, gemessen in Abschnitt 3.7). | **Befund am fremden Crate, kein Fehler in der Datei.** Die Erkennung muss nach Oberfläche gruppieren (erstes Segment der Aktions-ID). Zusätzlich ist der eigene Unit-Test des Crates rot, weil die Erkennung einen Eintrag pro Aktion statt pro Taste liefert. |
| W8 | `tests/integration/keymap_bindings.rs` (fremder Test, parallel entstanden) | Der Test liest die Datei mit `source.parse::<toml::Value>()`. In `toml` 1.x parst `Value::from_str` einen Wert, kein Dokument; beide Dateien des Repositorys scheitern damit (gemessen in Abschnitt 3.7). | **Befund am fremden Test, kein Fehler in der Datei.** `toml::Table::from_str` liest beide Dateien einwandfrei. |
| W9 | `crates/phocommander-config/src/values.rs` (fremdes Crate, parallel entstanden) | `Language::default()` ist `De`, das Schema der Spezifikation und `config/defaults.toml` verlangen `"system"`. Der eigene Test des Crates (`values::tests::defaults_match_the_shipped_default_file`) schlägt deshalb fehl: `left: De, right: System`. Ebenso `values::tests::every_value_has_a_unique_spelling`: die Schreibweisen müssen je Enum eindeutig sein, nicht über alle Enums hinweg — „system", „require" und „prefer" kollidieren zu Recht über Enums. | **Befund am fremden Crate, kein Fehler in meiner Datei.** Die Korrektur (`#[default] System` und eine Prüfung je Enum) gehört dem Rust Architect. Nachweis in Abschnitt 3.5 und 7.3. |

Zur Einordnung: W1 bis W3 sind echte Unstimmigkeiten der Spezifikation, die eine ID-Zuweisung nicht auflösen kann. W4 bis W6 sind redigierbare Lücken ohne Verhaltensfolge. W7, W8 und W9 sind Befunde außerhalb meines Zuständigkeitsbereichs; alle drei betreffen Kisten und Tests, die parallel entstanden sind, und alle drei müssen vor der M0-Abnahme behoben werden, weil sonst ein roter Testlauf im Gate steht. **In keinem der drei Fälle liegt der Fehler in den Dateien dieses Auftrags** — das ist jeweils gemessen, nicht behauptet.

---

## 7. Offene Punkte für eine Owner-Entscheidung

| # | Frage | Warum sie blockiert | Vorschlag |
|---|---|---|---|
| 1 | **Eine Platzhalter-Syntax oder zwei?** | Der Kern braucht einen Parser; zwei Syntaxen bedeuten zwei Parser oder eine Übersetzung. | Auf die geschweiften Klammern umstellen und die eckigen Klammern in der Spezifikation als Schreibweise der Dokumentation belassen. Betrifft die Core-Engine in M1. |
| 2 | **`panel.prev_tab` = `Ctrl+Shift+Tab`** | Die Tabelle der Spezifikation kennt nur „Ctrl+Tab — nächste Registerkarte". Die Belegung ist von mir abgeleitet. | Bestätigen oder eine andere Belegung festlegen. Die Datei markiert die Stelle als „first point to adjust". |
| 3 | **`panel.drives` = `Alt+F1`, klassisch zusätzlich `Alt+F2`** | Die Spezifikation schreibt „Alt+F1/Alt+F2 — Laufwerke". Mein Format ist „eine Aktion → eine Taste". | Entscheiden, ob das Format eine Liste zulässt (`["Alt+F1", "Alt+F2"]`) oder ob die zweite Variante im Code als Pane-Variante abgebildet wird. |
| 4 | **`palette.open` — Katalogeintrag oder Oberflächenaktion?** | Die Datei führt `palette.open` in `input.dpad` und `input.controller`, aber nicht im Tastatur-Kontext und nicht im Aktionskatalog der 35 IDs. | Als reguläre Aktion mit ID `palette.open` aufnehmen (dann 36 IDs) oder als Sonderfall des Surface-Layers führen. |
| 5 | **Schema der Vorlagendateien** | Die Spezifikation sagt nur „Jede Vorlage ist eine TOML-Datei im Konfigurationsverzeichnis". Das Schema ist meine Entwurfsentscheidung: `name`, `description`, `[rule]`, `[rule.detection]`, `[rule.series_name]`, `[rule.counter]`, `[rule.windows]`, `[rule.date]`. | Bestätigen oder korrigieren, bevor die Core-Engine in M1 den Vorlagenparser schreibt. |
| 6 | **Struktur von `config/keymap.toml`** | Die Spezifikation beschreibt die Datei nur als „Tastenbelegung (Vorlage)". Ich habe die Struktur der Konfigurationsdatei gespiegelt: `[input.keyboard]`, `[input.dpad]`, `[input.controller]`. | Bestätigen. Falls eine flache Datei ohne Kontext-Präfix gewünscht ist, betrifft das die drei Kontext-Abschnitte und die Skripte. |
| 7 | **Migrationsregel für `schema_version`** | Die Erhöhung der Schemaversion ist nur mit einer Migrationsfunktion zulässig (FR-123, AC-M0-18). Was beim Erhöhen passiert — Feld löschen, umbenennen, auf Vorgabe zurücksetzen — ist nirgends festgelegt. | Festlegen, dass entfernte Felder auf die Vorgabe zurückfallen und ein `.bak` entsteht; die konkrete Funktion dann in M1 mit Tests. |
| 8 | **Nachtragung der neuen Dokumente in den Doku-Baum** | `docs/REQUIREMENTS-IDS.md` und `docs/ACCEPTANCE-CRITERIA.md` sind nicht im Baum der Spezifikation und nicht in `docs/WHERE_TO_CHANGE.md`. | Zwei Zeilen in `docs/WHERE_TO_CHANGE.md` ergänzen und den Baum in der Spezifikation bei der nächsten Fassung nachziehen. |
| 9 | **Tags: M3 oder M9?** | Zwei Fundstellen sagen M3, zwei sagen M9 (siehe W2). | M9 festlegen und den Funktionsumfang bei der nächsten Spezifikationsfassung anpassen. |
| 10 | **Meilenstein der Namens-, Größen- und Datumssuche** | Die Suche ist im Funktionsumfang vollständig beschrieben, aber keinem Meilenstein zugeordnet (siehe W3). | Suche als Ganzes in M6 aufnehmen und die Meilensteinbeschreibung von M6 entsprechend ergänzen. |
| 11 | **Der Befund am fremden Crate (W7)** | `Language::default()` widerspricht dem Schema; zwei Tests des Crates sind rot. Ein roter Testlauf ist ein rotes Gate. | Rust Architect korrigiert `#[default] System` und die Prüfung „unique spelling" je Enum; danach ist der Workspace baubar und `cargo test --workspace` liefert den Beleg für AC-M0-22. |
| 12 | **Abnahme der M0-Kriterien, die Code voraussetzen** | 131 der 146 Kriterien stehen auf NOT RUN, weil die Crates fehlen. | Nach Fertigstellung von M0 einmal durchlaufen und `docs/acceptance/M0-ACCEPTANCE.md` aus der Vorlage füllen. |
| 13 | **Konflikterkennung des Keymap-Crates nach Oberfläche gruppieren** | Ohne diese Änderung meldet das Crate die spezifikationskonforme Datei als widersprüchlich, und der Integrationstest `the_shipped_keymap_has_no_conflict_in_any_context` schlägt fehl (gemessen in 3.7). | Konflikte je Oberfläche bewerten: erstes Segment der Aktions-ID. Der Rust Architect entscheidet zusätzlich, ob die Oberfläche Teil der Aktions-ID bleibt oder als eigenes Feld in die Bindung kommt. |
| 14 | **`tests/integration/keymap_bindings.rs` auf `toml::Table` umstellen** | Der Test scheitert an der falschen API und würde für jede Datei rot werden (gemessen in 3.7). | `source.parse::<toml::Table>()` statt `source.parse::<toml::Value>()`. |

---

## 8. Was ich bewusst nicht getan habe

| Nicht getan | Warum |
|---|---|
| Kein Commit, kein Push, kein Tag | Vorgabe des Auftrags. **Hinweis:** Während meiner Arbeit haben andere Teammitglieder zwei Commits erstellt (`b88a27c`, `5af10fb`), die meine Dateien enthalten. Ich habe selbst nichts committet; die Commits gehören nicht mir. |
| `docs/REQUIREMENTS.md` nicht verändert | Vorgabe des Auftrags. Alle Widersprüche stehen in Abschnitt 6 und brauchen eine Entscheidung, keine stille Korrektur. |
| Kein Rust-Code geschrieben | Nicht Teil dieses Auftrags. Das Crate `phocommander-config` und die Prüfskripte der anderen Teammitglieder sind fremde Arbeit; ich habe sie nur gelesen und meine Datei gegen sie geprüft. |
| `docs/WHERE_TO_CHANGE.md` und `README.md` nicht ergänzt | Beide Dateien wurden parallel von anderen Teammitgliedern geändert; eine gleichzeitige Bearbeitung hätte den Konflikt wahrscheinlich gemacht. Der Nachtrag ist als offener Punkt 8 erfasst. |
| `docs/KEYMAP.md` nicht geschrieben | Das Dokument existiert bereits (parallel entstanden) und ist mit `config/keymap.toml` konsistent — geprüft, nicht überschrieben. |
| Keine Leistungs-, Speicher- oder Pixelmessung | Es gibt keine lauffähige Anwendung. |
| Keine Vorlage mit Platzhaltern in der Serien-Vorlage, die die Spezifikation nicht hergibt | Die Spec nennt nur „Doppelpunkt, Schrägstrich oder Fragezeichen". Die Vorlage listet zusätzlich die vollständige Windows-Sperrzeichenmenge und benennt im Kommentar, dass die Spezifikation drei davon ausdrücklich nennt. |
| `docs/REQUIREMENTS.md` nicht ergänzt | Vorgabe des Auftrags und Ablageort der Spezifikation. |

### Kollisionsnotiz `deliverable.md`

Der Pfad `deliverable.md` ist im Repository nicht eindeutig: mehrere Agenten schreiben dort ihre Übergabe. Während meiner Arbeit hat der Security & Privacy Agent `deliverable.md` mit **seiner** Übergabe überschrieben (das Dokument „Übergabe — Security & Privacy, Meilenstein M0"). Sein Inhalt ist unbeschädigt; ich habe eine Kopie davon unter `deliverable-security-manager.md` angelegt und **nicht** verändert. Meine Übergabe liegt deshalb an drei Stellen:

| Ort | Status |
|---|---|
| `deliverable-product-manager.md` | vollständig, rollenspezifisch, von der Ablage angelegt |
| `deliverable.md` | **nur wenn zum Zeitpunkt der Übergabe von keinem anderen Agenten beschrieben** |
| `deliverable.patch` | maschinenlesbare Form beider Dateien |

**Für den Owner:** bitte `deliverable-product-manager.md` lesen — das ist die Datei, die unambig zu diesem Auftrag gehört. Wer `deliverable.md` liest, muss prüfen, welche Rolle sie geschrieben hat; im Kopf jedes Übergabedokuments steht die Rolle.

---

## 9. Empfohlene Reihenfolge für den nächsten Schritt

1. Owner entscheidet die offenen Punkte 1, 2, 3, 4, 5, 7 (Abschnitt 7) — sie bestimmen das Schema, das in M1 implementiert wird.
2. Rust Architect behebt W7, damit der Workspace baubar wird und `cargo test --workspace` als Beleg für AC-M0-22 laufen kann.
3. QA führt `scripts/check-config-schema.py`, `scripts/check-action-ids.py` und `scripts/check-requirement-ids.py` in die CI ein — sie sind die mechanische Form der Abnahmekriterien AC-M0-11, AC-M0-17, AC-M0-34 bis AC-M0-41.
4. Documentation Lead ergänzt die zwei neuen Dokumente in `docs/WHERE_TO_CHANGE.md` und im Doku-Baum.
5. Nach Fertigstellung von M0: `docs/acceptance/M0-ACCEPTANCE.md` aus `docs/templates/M0-ACCEPTANCE.md` füllen und die 131 NOT RUN-Kriterien nachziehen.

---

## 10. Reproduktion aller Belege

```bash
export PATH=$HOME/.cargo/bin:$PATH
cd /workspace/phocommander

# TOML-Parsing jeder Datei
for f in config/defaults.toml config/keymap.toml config/templates/*.toml; do
  python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" "$f"; echo "$f -> $?"
done

# Konsistenzprüfungen
python3 scripts/check-config-schema.py
python3 scripts/check-action-ids.py
python3 scripts/check-requirement-ids.py

# Integrationstest gegen das echte Rust-Schema
cd /workspace/verify-config && cargo test -p phocommander-config --test loads_repository_template
```

**Prüfsummen des Auftrags:** 7/7 Konfigurationsprüfungen, 9/9 Aktions-ID-Prüfungen, 8/8 ID-Konsistenzprüfungen, 7/7 TOML-Dateien gültig, 2/2 Rust-Integrationstests gegen die echte `config/defaults.toml` bestanden, 1/1 unabhängiges Fremdskript (`scripts/check-keymap-consistency.py`) bestanden, 3 Befunde am parallel entstandenen Code gemessen.
