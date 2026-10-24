# Übergabe — Product Manager

**Projekt:** Phocommander · **Repository:** `/workspace/phocommander` · **Branch:** `main`
**Spezifikation:** `docs/REQUIREMENTS.md`, Version 2.0 (24.10.2026) · **Datum:** 24.10.2026
**Auftrag:** Der Anforderungskatalog und die Abnahmekriterien wurden von der deutschen
Spezifikation 1.1 auf die englische Spezifikation 2.0 umgestellt.

Dieses Dokument ist auf Deutsch, die Inhalte des Repositorys bleiben englisch. Es wurde kein
Commit erzeugt — die Übergabe erfolgt über die Dateien im Arbeitsbaum und dieses Dokument.

---

## 1. Was in dieser Sitzung gemacht wurde

| # | Aufgabe | Ergebnis |
|---|---|---|
| 1 | `docs/REQUIREMENTS-IDS.md` neu aufgebaut | 129 Zeilen, je eine Zeile je Anforderung aus Abschnitt 4 der Spezifikation, mit Quelle (Abschnitt **und** Zeile), Meilenstein, Priorität, Verifikationsart und AK-ID |
| 2 | `docs/ACCEPTANCE-CRITERIA.md` neu geschrieben | 105 Abnahmekriterien, je eines je Anforderung aus M0 (13), M1 (47) und M2 (45) |
| 3 | `docs/templates/M0-ACCEPTANCE.md` neu | Vorlage mit den acht Abschnitten aus Abschnitt 13.1 der Spezifikation |
| 4 | `config/profiles/clean.toml`, `config/profiles/full.toml` neu | Die beiden ausgelieferten Interface-Profile nach dem Muster aus Abschnitt 7.2 |
| 5 | `scripts/check-requirement-ids.py` angepasst | Prüft jetzt Spezifikation gegen Katalog: Mengengleichheit, Dubletten, Lücken innerhalb der Blöcke, Buchstaben-Suffixe, Spalten, Quellzeilen, AK-IDs |
| 6 | `scripts/check-acceptance-ids.py` neu | Prüft die AK-Konsistenz: AK-IDs eindeutig und lückenlos, jede AK zeigt auf eine existierende FR-ID, jede FR-ID aus M0–M2 hat mindestens eine AK |
| 7 | `scripts/check-interface-profiles.py` neu | Prüft die Profile: TOML-Syntax, Schema, Kommentarabdeckung, Werte der beiden ausgelieferten Profile |
| 8 | `scripts/check-keymap-consistency.py` repariert | Das Skript stürzte seit der Neufassung der Spezifikation ab (es suchte die deutsche Sektion „Tastatursteuerung", die es in Version 2.0 nicht mehr gibt) |
| 9 | Weitere Artefakte auf die neue Struktur umgestellt | `docs/KEYMAP.md`, `SECURITY.md`, `docs/security/THREAT_MODEL.md`, `docs/architecture/README.md`, `tests/security/README.md`, `scripts/check-action-ids.py`, `scripts/check-no-telemetry.sh` — nur die Verweise auf die Sektionsnamen, kein inhaltlicher Eingriff |
| 10 | `scripts/check-gates-negative.sh` erweitert | Gate 5 (Anforderungs-IDs) und Gate 6 (Abnahmekriterien) mit Negativtest: beide Gates werden rot bei einem kaputten Zustand und wieder grün |

### Beweis, dass die neuen Gates Zähne haben

Die Gates wurden nicht nur grün, sondern auch rot geprüft. `scripts/check-gates-negative.sh`
bricht den Katalog und das Akzeptanzdokument absichtlich und zeigt, dass beide Skripte dann
mit Exit-Code 1 abbrechen.

```console
$ ./scripts/check-gates-negative.sh
==============================================================
Gate 1 — scripts/check-no-telemetry.sh
==============================================================

1.1 positive run (the repository as it is):
Dependency names read : 61
Forbidden crate names : 48
Violations            : 0
RESULT: PASS — no dependency of the telemetry or analytics category is referenced.
  expected exit code 0, got 0 — the gate is green on the repository
  RESULT: PASS — the gate is effective

1.2 negative run (a dependency of the forbidden category is simulated
    by pointing the list of the script at a crate that IS in the tree):
VIOLATION  ./Cargo.toml  references the forbidden crate 'egui'
VIOLATION  ./Cargo.toml  references the forbidden crate 'zip'
VIOLATION  ./crates/phocommander-archive/Cargo.toml  references the forbidden crate 'zip'
VIOLATION  ./crates/phocommander-ui/Cargo.toml  references the forbidden crate 'egui'
Violations            : 4
RESULT: FAIL — a dependency of the telemetry category is referenced.
  expected exit code 1, got 1 — the gate is red on a violating dependency
  RESULT: PASS — the gate is effective

==============================================================
Gate 2 — scripts/check-file-size.sh
==============================================================
2.1 negative run (a source file with 600 lines, above the hard limit of 550):
  expected exit code 1, got 1 — the gate is red above the hard limit
  RESULT: PASS — the gate is effective

2.2 positive run (the same file with 100 lines, below the soft limit of 400):
  expected exit code 0, got 0 — the gate is green below the soft limit
  RESULT: PASS — the gate is effective

==============================================================
Gate 3 — scripts/check-docs-links.py
==============================================================
3.1 negative run (a document with a link to a file that does not exist):
docs/broken.md:3  does-not-exist.md  BROKEN  (create the file or fix the link)
Broken links           : 1
RESULT: FAIL — a dead link is a documentation defect.
  expected exit code 1, got 1 — the gate is red on a dead link
  RESULT: PASS — the gate is effective

3.2 positive run (the same document with a link to a file that exists):
Broken links           : 0
RESULT: PASS — every repository link resolves.
  expected exit code 0, got 0 — the gate is green on a live link
  RESULT: PASS — the gate is effective

==============================================================
Gate 4 — scripts/check-crate-boundaries.sh
==============================================================
4.1 negative run (a crate below crates/ gets a dependency on a sibling):
8 path dependencies checked, 1 violations
  expected exit code 1, got 1 — the gate is red on a dependency between two crates of the same level
  RESULT: PASS — the gate is effective

4.2 positive run (the dependency is removed again):
8 path dependencies checked, 0 violations
  expected exit code 0, got 0 — the gate is green again
  RESULT: PASS — the gate is effective

==============================================================
Gate 5 — scripts/check-requirement-ids.py
==============================================================
5.1 negative run (a requirement of the catalogue is removed):
        FR-22c is in the specification but not in the catalogue
        FR-22c is in the specification but not in the catalogue
RESULT: FAIL — 2 of 7 checks failed
  expected exit code 1, got 1 — the gate is red when a requirement is missing
  RESULT: PASS — the gate is effective

5.2 negative run (a requirement of the catalogue is written twice):
        duplicate id FR-22c on lines 63 and 227
RESULT: FAIL — 1 of 7 checks failed
  expected exit code 1, got 1 — the gate is red when a requirement is written twice
  RESULT: PASS — the gate is effective

5.3 positive run (the catalogue of the repository):
RESULT: PASS — 7 of 7 checks passed
  expected exit code 0, got 0 — the gate is green on the catalogue of the repository
  RESULT: PASS — the gate is effective

==============================================================
Gate 6 — scripts/check-acceptance-ids.py
==============================================================
6.1 negative run (the acceptance criterion of a requirement is removed):
        FR-22c (M1) has no acceptance criterion
        FR-22c names the criterion AC-M1-10 in the catalogue, but the acceptance document holds no criterion
RESULT: FAIL — 3 of 6 checks failed
  expected exit code 1, got 1 — the gate is red when a criterion is missing
  RESULT: PASS — the gate is effective

6.2 negative run (a criterion references a requirement that does not exist):
        line 33: AC-M0-02 references FR-99, which is not in the catalogue
RESULT: FAIL — 3 of 6 checks failed
  expected exit code 1, got 1 — the gate is red on a criterion that points at nothing
  RESULT: PASS — the gate is effective

6.3 positive run (the acceptance document of the repository):
RESULT: PASS — 6 of 6 checks passed
  expected exit code 0, got 0 — the gate is green on the acceptance document of the repository
  RESULT: PASS — the gate is effective

==============================================================
RESULT: PASS — every gate turned red on a broken state and green again.
(exit code 0)
```
---

## 2. Dateiliste

| Pfad | Zeilen | Zweck | geändert oder neu |
|---|---|---|---|
| `docs/REQUIREMENTS-IDS.md` | 226 | Anforderungskatalog: 129 Anforderungen, sieben Spalten, Quelle mit Abschnitt und Zeile | geändert |
| `docs/ACCEPTANCE-CRITERIA.md` | 200 | 105 Abnahmekriterien für M0, M1 und M2, mit How to Verify, Pass Condition und Belegstatus | geändert |
| `docs/templates/M0-ACCEPTANCE.md` | 146 | Ausfüllbare Abnahmedokumentation mit den acht Abschnitten aus Abschnitt 13.1 | geändert |
| `config/profiles/clean.toml` | 82 | Ausgeliefertes Interface-Profil `clean`: zwei Panels, Pfad, Dateiliste, Eingabezeile | neu |
| `config/profiles/full.toml` | 79 | Ausgeliefertes Interface-Profil `full`: jede Aktion und jeder Panel-Bereich sichtbar | neu |
| `scripts/check-requirement-ids.py` | 419 | Gate: Mengengleichheit Spezifikation ↔ Katalog, Dubletten, Lücken, Buchstaben-Suffixe, Spalten, Quellzeilen, AK-IDs | geändert |
| `scripts/check-acceptance-ids.py` | 304 | Gate: AK-IDs eindeutig und lückenlos, AK ↔ FR-IDs wechselseitig, jede AK verifizierbar | neu |
| `scripts/check-interface-profiles.py` | 241 | Gate: Profile parsen, halten das Schema, sind kommentiert und tragen die Werte ihrer Namen | neu |
| `scripts/check-keymap-consistency.py` | 357 | Gate: repariert für Spezifikation 2.0 — klassischer Tastensatz, Referenztabelle, Kontextgleichheit | geändert |
| `scripts/check-gates-negative.sh` | 244 | Negativtest der Gates; um Gate 5 (Anforderungs-IDs) und Gate 6 (Abnahmekriterien) erweitert | geändert |
| `scripts/check-action-ids.py` | 345 | Kommentar auf die Sektion 6.3 der Spezifikation 2.0 umgestellt (inhaltlich unverändert) | geändert |
| `scripts/check-no-telemetry.sh` | 179 | Kommentar auf die Sektion 2.3 der Spezifikation 2.0 und die AK-ID AC-M0-03 umgestellt | geändert |
| `docs/KEYMAP.md` | 252 | Verweis auf die Sektion 6.1 und 6.3 der Spezifikation 2.0 umgestellt | geändert |
| `SECURITY.md` | 185 | Verweis auf die Sektionen 9.2, 9.4, 9.5 und 4.8 der Spezifikation 2.0 umgestellt | geändert |
| `docs/security/THREAT_MODEL.md` | 549 | Verweise auf die Sektionen 9, 4.8, 2 und 2.3 der Spezifikation 2.0 umgestellt | geändert |
| `docs/architecture/README.md` | 145 | Verweis auf die Sektion 5 der Spezifikation 2.0 umgestellt | geändert |
| `tests/security/README.md` | 66 | Verweis auf die Sektionen 9 und 10 der Spezifikation 2.0 umgestellt | geändert |

**Nicht von mir angefasst:** `docs/REQUIREMENTS.md` gehört dem Owner und wurde nicht
verändert. Die Änderungen an `crates/phocommander-core/` und `crates/phocommander-ops/` im
Arbeitsbaum (neue Dateien wie `checksum.rs`, `entry.rs`, `copy.rs`, `sync.rs`, `port.rs` und die
Verzeichnisse `error/`, `local/`, `path/`, `formats/`) stammen aus einem parallelen Arbeitsstrang
und sind nicht Teil dieser Übergabe; die Folge ist ein rotes Dateigrößen-Gate, siehe Abschnitt
3.5. Ebenso liegt ein fremder Commit `f6853c6` im Verlauf, der einen Teil dieser Dateien
mitgenommen hat — ich habe selbst keinen Commit erzeugt.

---

## 3. Ausgaben der Prüfskripte

Alle Ausgaben sind in dieser Sitzung erzeugt und hier wörtlich wiedergegeben.

### 3.1 ID-Abgleich — `scripts/check-requirement-ids.py`

```console
$ python3 scripts/check-requirement-ids.py
catalogue   : /run/csi/mount-root/nas/eab0d61a99b6696edb3d2aff87b585e8/phocommander/docs/REQUIREMENTS-IDS.md
specification: /run/csi/mount-root/nas/eab0d61a99b6696edb3d2aff87b585e8/phocommander/docs/REQUIREMENTS.md
requirements in the specification : 129
rows in the catalogue             : 129

OK    the specification and the catalogue are readable
OK    the identifier sets are equal
OK    no identifier occurs twice
OK    the numbering is gapless inside every block
OK    every row carries the seven columns of the catalogue
OK    every source names the line of the requirement
OK    every acceptance criterion exists in the acceptance document

RESULT: PASS — 7 of 7 checks passed
(exit code 0)
```
Der Abgleich belegt die Mengengleichheit: 129 Anforderungen in der Spezifikation, 129 Zeilen im
Katalog, gleiche Menge, keine Dubletten. Der Test „the numbering is gapless inside every block"
bewertet die Nummerierung *innerhalb* der thematischen Blöcke; die Lücken *zwischen* den Blöcken
(FR-06 → FR-10, FR-22 → FR-23, FR-41 → FR-50 usw.) sind Teil der Nummerierung der Spezifikation
und werden nicht als Fehler gemeldet. Die Buchstaben-Suffixe werden getrennt geprüft: FR-22a bis
FR-22l und NFR-09a/NFR-09b müssen auf einer Basisnummer ohne Lücke sitzen.

### 3.2 Konsistenz der AK-IDs — `scripts/check-acceptance-ids.py`

```console
$ python3 scripts/check-acceptance-ids.py
acceptance   : /run/csi/mount-root/nas/eab0d61a99b6696edb3d2aff87b585e8/phocommander/docs/ACCEPTANCE-CRITERIA.md
catalogue    : /run/csi/mount-root/nas/eab0d61a99b6696edb3d2aff87b585e8/phocommander/docs/REQUIREMENTS-IDS.md
requirements in the catalogue : 129
criteria in the document      : 105

OK    the acceptance document and the catalogue are readable
OK    the acceptance identifiers are unique and gapless
OK    every criterion references a requirement of the catalogue
OK    every requirement of M0, M1 and M2 has a criterion
OK    the catalogue and the acceptance document name each other
OK    every criterion is verifiable

RESULT: PASS — 6 of 6 checks passed
(exit code 0)
```
Der Abgleich belegt beides: jede AK-Zeile referenziert eine existierende FR-ID (Check 3), und
jede FR-ID aus M0 bis M2 hat mindestens eine AK (Check 4). Check 5 prüft zusätzlich die
Wechselseitigkeit — der Katalog und das Akzeptanzdokument nennen einander.

### 3.3 TOML-Validierung der Profile

```console
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/profiles/clean.toml

(exit code 0)
```
```console
$ python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" config/profiles/full.toml

(exit code 0)
```

### 3.4 Gate für die Interface-Profile — `scripts/check-interface-profiles.py`

```console
$ python3 scripts/check-interface-profiles.py
profiles  : /run/csi/mount-root/nas/eab0d61a99b6696edb3d2aff87b585e8/phocommander/config/profiles
            config/profiles/clean.toml
            config/profiles/full.toml

OK    all 2 profiles parse as TOML
OK    every profile carries exactly the keys of the schema
OK    every key and every section carries a comment
OK    the shipped profiles hold the values of their names

RESULT: PASS — 4 of 4 checks passed
(exit code 0)
```
### 3.5 Die übrigen Gates des Repositorys (unverändert weitergeführt)

```console
$ python3 scripts/check-config-schema.py
config    : /run/csi/mount-root/nas/eab0d61a99b6696edb3d2aff87b585e8/phocommander/config
files     : 9
           config/defaults.toml
           config/keymap.toml
           config/profiles/clean.toml
           config/profiles/full.toml
           config/templates/camera.toml
           config/templates/invoice.toml
           config/templates/music.toml
           config/templates/podcast.toml
           config/templates/series.toml

OK    all 9 files under config/ parse as TOML
OK    the configuration file carries every section of the specification
OK    the default values match the specification
OK    every key and every section carries a comment
OK    the shipped rename templates are complete
OK    the overrides match the action catalogue
OK    the configuration file and the Rust schema agree

RESULT: PASS — 7 of 7 checks passed
(exit code 0)
```
```console
$ python3 scripts/check-action-ids.py
keymap    : /run/csi/mount-root/nas/eab0d61a99b6696edb3d2aff87b585e8/phocommander/config/keymap.toml
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
(exit code 0)
```
```console
$ python3 scripts/check-keymap-consistency.py
=== A. The specification names the catalogue and the reference ===
  section 6.1 names config/keymap.toml and docs/KEYMAP.md  ok

=== B. The classic key set of the specification ===
  F2             panel.rename               F2               ok
  F3             panel.view                 F3               ok
  F4             panel.edit                 F4               ok
  F5             panel.copy                 F5               ok
  F6             panel.move                 F6               ok
  F7             panel.mkdir                F7               ok
  F8             panel.delete               F8               ok
  F9             app.menu                   F9               ok
  F10            app.quit                   F10              ok
  F3             viewer.find_next           F3               ok
  F3             rename.conflict_next       F3               ok
  Shift+F3       viewer.find_previous       Shift+F3         ok
  Shift+F3       rename.conflict_previous   Shift+F3         ok

=== C. Action catalogue -> reference table (docs/KEYMAP.md) ===

=== D. The three contexts hold the same actions ===
  keyboard / dpad / controller : ok

=== E. Actions that no rule names ===
  panel.prev_tab: config/keymap.toml and docs/KEYMAP.md mark Ctrl+Shift+Tab as the symmetric completion of Ctrl+Tab; the specification names only the forward direction

Actions in the catalogue        : 35
Actions documented in KEYMAP.md : 36

Accepted deviations (documented in the repository): 3
  - panel.prev_tab: config/keymap.toml and docs/KEYMAP.md mark Ctrl+Shift+Tab as the symmetric completion of Ctrl+Tab; the specification names only the forward direction
  - panel.drives is bound to Alt+F1; config/keymap.toml documents Alt+F2 in the comment as the classic variant for the drive list of the other panel
  - the action palette is opened with the Menu key, and with the Play/Pause key on a remote control that has no Menu key
Undocumented deviations: 0
RESULT: PASS — specification, catalogue and reference agree.
(exit code 0)
```
```console
$ ./scripts/check-file-size.sh
HARD  724 lines  ./crates/phocommander-ops/src/copy.rs
SOFT  437 lines  ./crates/phocommander-ops/src/edit.rs
SOFT  442 lines  ./crates/phocommander-ops/src/error.rs
SOFT  475 lines  ./crates/phocommander-ops/src/job.rs
HARD  606 lines  ./crates/phocommander-ops/src/progress.rs
SOFT  435 lines  ./crates/phocommander-ops/src/sync.rs

Source files measured : 59 (extensions: rs)
Soft limit            : 400 lines (warning)
Hard limit            : 550 lines (failure)
Soft-limit violations : 4
Hard-limit violations : 2
RESULT: FAIL — a source file exceeds the hard limit of 550 lines. Split it.
(exit code 1)
```

**Hinweis zu diesem Gate:** Der Lauf ist rot, weil der parallele Arbeitsstrang zwei
Quelldateien angelegt hat, die über der harten Grenze von 550 Zeilen liegen. Die Dateien
gehören nicht zu dieser Übergabe (siehe Abschnitt 2).

```
HARD  724 lines  ./crates/phocommander-ops/src/copy.rs
HARD  606 lines  ./crates/phocommander-ops/src/progress.rs
```

Der Katalog und die Abnahmekriterien sind davon nicht betroffen: keine der Dateien wird von
den Gates dieser Übergabe geprüft. Der Befund ist an den Owner weiterzugeben, weil er den
Meilenstein M1 blockiert.

```console
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
(exit code 0)
```
```console
$ ./scripts/check-no-telemetry.sh

Manifests scanned     : 15
Dependency names read : 61
Forbidden crate names : 48
Violations            : 0
RESULT: PASS — no dependency of the telemetry or analytics category is referenced.
(exit code 0)
```
```console
$ python3 scripts/check-docs-links.py
SOURCE                                     TARGET                                                                                                                               STATUS
-----------------------------------------  -----------------------------------------------------------------------------------------------------------------------------------  ----------------
AGENTS.md:7                                docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
AGENTS.md:26                               docs/WHERE_TO_CHANGE.md                                                                                                              ok  (resolves)
CHANGELOG.md:9                             docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
CHANGELOG.md:19                            README.md                                                                                                                            ok  (resolves)
CHANGELOG.md:21                            docs/KEYMAP.md                                                                                                                       ok  (resolves)
CHANGELOG.md:24                            docs/glossary.md                                                                                                                     ok  (resolves)
CHANGELOG.md:26                            docs/user-guide/                                                                                                                     ok  (resolves)
CHANGELOG.md:29                            docs/architecture/README.md                                                                                                          ok  (resolves)
CHANGELOG.md:31                            docs/WHERE_TO_CHANGE.md                                                                                                              ok  (resolves)
CHANGELOG.md:33                            docs/REQUIREMENTS-IDS.md                                                                                                             ok  (resolves)
CHANGELOG.md:35                            docs/ACCEPTANCE-CRITERIA.md                                                                                                          ok  (resolves)
CHANGELOG.md:37                            docs/security/THREAT_MODEL.md                                                                                                        ok  (resolves)
CHANGELOG.md:39                            docs/templates/M0-ACCEPTANCE.md                                                                                                      ok  (resolves)
CHANGELOG.md:41                            SECURITY.md                                                                                                                          ok  (resolves)
CHANGELOG.md:41                            PRIVACY.md                                                                                                                           ok  (resolves)
CHANGELOG.md:42                            CODE_OF_CONDUCT.md                                                                                                                   ok  (resolves)
CHANGELOG.md:42                            CONTRIBUTING.md                                                                                                                      ok  (resolves)
CHANGELOG.md:43                            AGENTS.md                                                                                                                            ok  (resolves)
CHANGELOG.md:46                            scripts/                                                                                                                             ok  (resolves)
CODE_OF_CONDUCT.md:41                      SECURITY.md                                                                                                                          ok  (resolves)
CONTRIBUTING.md:5                          AGENTS.md                                                                                                                            ok  (resolves)
CONTRIBUTING.md:6                          docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
CONTRIBUTING.md:9                          docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
CONTRIBUTING.md:12                         docs/WHERE_TO_CHANGE.md                                                                                                              ok  (resolves)
CONTRIBUTING.md:33                         SECURITY.md                                                                                                                          ok  (resolves)
CONTRIBUTING.md:89                         AGENTS.md                                                                                                                            ok  (resolves)
CONTRIBUTING.md:145                        .github/workflows/ci.yml                                                                                                             ok  (resolves)
CONTRIBUTING.md:160                        docs/KEYMAP.md                                                                                                                       ok  (resolves)
CONTRIBUTING.md:161                        docs/WHERE_TO_CHANGE.md                                                                                                              ok  (resolves)
CONTRIBUTING.md:173                        CODE_OF_CONDUCT.md                                                                                                                   ok  (resolves)
CONTRIBUTING.md:183                        LICENSE-MIT                                                                                                                          ok  (resolves)
CONTRIBUTING.md:184                        LICENSE-APACHE                                                                                                                       ok  (resolves)
CONTRIBUTING.md:198                        SECURITY.md                                                                                                                          ok  (resolves)
PRIVACY.md:170                             docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
PRIVACY.md:178                             SECURITY.md                                                                                                                          ok  (resolves)
README.md:22                               docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
README.md:81                               docs/WHERE_TO_CHANGE.md                                                                                                              ok  (resolves)
README.md:87                               docs/KEYMAP.md                                                                                                                       ok  (resolves)
README.md:127                              docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
README.md:128                              docs/REQUIREMENTS-IDS.md                                                                                                             ok  (resolves)
README.md:129                              docs/KEYMAP.md                                                                                                                       ok  (resolves)
README.md:130                              docs/user-guide/                                                                                                                     ok  (resolves)
README.md:131                              docs/glossary.md                                                                                                                     ok  (resolves)
README.md:132                              docs/WHERE_TO_CHANGE.md                                                                                                              ok  (resolves)
README.md:133                              docs/architecture/README.md                                                                                                          ok  (resolves)
README.md:134                              AGENTS.md                                                                                                                            ok  (resolves)
README.md:135                              CONTRIBUTING.md                                                                                                                      ok  (resolves)
README.md:136                              SECURITY.md                                                                                                                          ok  (resolves)
README.md:137                              PRIVACY.md                                                                                                                           ok  (resolves)
README.md:138                              CODE_OF_CONDUCT.md                                                                                                                   ok  (resolves)
README.md:139                              CHANGELOG.md                                                                                                                         ok  (resolves)
README.md:145                              PRIVACY.md                                                                                                                           ok  (resolves)
README.md:147                              LICENSE-MIT                                                                                                                          ok  (resolves)
README.md:147                              LICENSE-APACHE                                                                                                                       ok  (resolves)
README.md:154                              LICENSE-APACHE                                                                                                                       ok  (resolves)
README.md:155                              LICENSE-MIT                                                                                                                          ok  (resolves)
SECURITY.md:10                             docs/security/THREAT_MODEL.md                                                                                                        ok  (resolves)
SECURITY.md:15                             docs/REQUIREMENTS.md                                                                                                                 ok  (resolves)
SECURITY.md:171                            PRIVACY.md                                                                                                                           ok  (resolves)
SECURITY.md:181                            PRIVACY.md                                                                                                                           ok  (resolves)
deliverable-rust-architect.md:399          docs/adr/ADR-0001-cargo-workspace-and-module-boundaries.md                                                                           ok  (resolves)
deliverable-rust-architect.md:400          docs/adr/ADR-0002-filesystem-abstraction.md                                                                                          ok  (resolves)
deliverable-rust-architect.md:401          docs/adr/ADR-0003-error-hierarchy-and-error-codes.md                                                                                 ok  (resolves)
deliverable-rust-architect.md:402          docs/adr/ADR-0004-archive-extraction-security.md                                                                                     ok  (resolves)
deliverable-security-manager.md:649        docs/security/THREAT_MODEL.md                                                                                                        ok  (resolves)
deliverable.security-manager.md:653        docs/security/THREAT_MODEL.md                                                                                                        ok  (resolves)
docs/ACCEPTANCE-CRITERIA.md:3              REQUIREMENTS-IDS.md                                                                                                                  ok  (resolves)
docs/ACCEPTANCE-CRITERIA.md:3              REQUIREMENTS.md                                                                                                                      ok  (resolves)
docs/ACCEPTANCE-CRITERIA.md:10             REQUIREMENTS-IDS.md                                                                                                                  ok  (resolves)
docs/KEYMAP.md:10                          ../config/keymap.toml                                                                                                                ok  (resolves)
docs/KEYMAP.md:13                          ../config/defaults.toml                                                                                                              ok  (resolves)
docs/KEYMAP.md:16                          REQUIREMENTS.md                                                                                                                      ok  (resolves)
docs/KEYMAP.md:45                          REQUIREMENTS.md                                                                                                                      ok  (resolves)
docs/KEYMAP.md:98                          ../config/keymap.toml                                                                                                                ok  (resolves)
docs/REQUIREMENTS-IDS.md:3                 REQUIREMENTS.md                                                                                                                      ok  (resolves)
docs/REQUIREMENTS-IDS.md:17                ACCEPTANCE-CRITERIA.md                                                                                                               ok  (resolves)
docs/REQUIREMENTS-IDS.md:24                ACCEPTANCE-CRITERIA.md                                                                                                               ok  (resolves)
docs/WHERE_TO_CHANGE.md:7                  REQUIREMENTS.md                                                                                                                      ok  (resolves)
docs/WHERE_TO_CHANGE.md:24                 ../config/defaults.toml                                                                                                              ok  (resolves)
docs/WHERE_TO_CHANGE.md:25                 ../config/keymap.toml                                                                                                                ok  (resolves)
docs/WHERE_TO_CHANGE.md:45                 ../.github/workflows/ci.yml                                                                                                          ok  (resolves)
docs/WHERE_TO_CHANGE.md:47                 REQUIREMENTS.md                                                                                                                      ok  (resolves)
docs/WHERE_TO_CHANGE.md:48                 REQUIREMENTS-IDS.md                                                                                                                  ok  (resolves)
docs/WHERE_TO_CHANGE.md:50                 architecture/README.md                                                                                                               ok  (resolves)
docs/WHERE_TO_CHANGE.md:51                 user-guide/                                                                                                                          ok  (resolves)
docs/WHERE_TO_CHANGE.md:52                 KEYMAP.md                                                                                                                            ok  (resolves)
docs/WHERE_TO_CHANGE.md:53                 glossary.md                                                                                                                          ok  (resolves)
docs/WHERE_TO_CHANGE.md:54                 ../SECURITY.md                                                                                                                       ok  (resolves)
docs/WHERE_TO_CHANGE.md:55                 ../PRIVACY.md                                                                                                                        ok  (resolves)
docs/WHERE_TO_CHANGE.md:56                 ../CONTRIBUTING.md                                                                                                                   ok  (resolves)
docs/WHERE_TO_CHANGE.md:56                 ../AGENTS.md                                                                                                                         ok  (resolves)
docs/WHERE_TO_CHANGE.md:57                 ../CODE_OF_CONDUCT.md                                                                                                                ok  (resolves)
docs/WHERE_TO_CHANGE.md:58                 ../CHANGELOG.md                                                                                                                      ok  (resolves)
docs/acceptance/M0-ACCEPTANCE.md:19        logs/                                                                                                                                ok  (resolves)
docs/architecture/README.md:5              ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/architecture/README.md:139            ../WHERE_TO_CHANGE.md                                                                                                                ok  (resolves)
docs/architecture/README.md:143            ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/architecture/README.md:144            ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/architecture/README.md:145            ../glossary.md                                                                                                                       ok  (resolves)
docs/glossary.md:8                         REQUIREMENTS.md                                                                                                                      ok  (resolves)
docs/security/THREAT_MODEL.md:4            ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/security/THREAT_MODEL.md:540          ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/security/THREAT_MODEL.md:541          ../../SECURITY.md                                                                                                                    ok  (resolves)
docs/security/THREAT_MODEL.md:542          ../../PRIVACY.md                                                                                                                     ok  (resolves)
docs/security/THREAT_MODEL.md:544          ../../deny.toml                                                                                                                      ok  (resolves)
docs/templates/M0-ACCEPTANCE.md:21         ../ACCEPTANCE-CRITERIA.md                                                                                                            ok  (resolves)
docs/templates/M0-ACCEPTANCE.md:23         ../REQUIREMENTS-IDS.md                                                                                                               ok  (resolves)
docs/user-guide/getting-started.de.md:3    getting-started.en.md                                                                                                                ok  (resolves)
docs/user-guide/getting-started.de.md:12   ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.de.md:13   ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/user-guide/getting-started.de.md:88   ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.de.md:172  ../../PRIVACY.md                                                                                                                     ok  (resolves)
docs/user-guide/getting-started.de.md:180  ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.de.md:186  ../glossary.md                                                                                                                       ok  (resolves)
docs/user-guide/getting-started.de.md:187  ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/user-guide/getting-started.de.md:193  multi-rename.de.md                                                                                                                   ok  (resolves)
docs/user-guide/getting-started.de.md:194  ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.de.md:195  ../glossary.md                                                                                                                       ok  (resolves)
docs/user-guide/getting-started.en.md:3    getting-started.de.md                                                                                                                ok  (resolves)
docs/user-guide/getting-started.en.md:11   ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.en.md:12   ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/user-guide/getting-started.en.md:85   ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.en.md:168  ../../PRIVACY.md                                                                                                                     ok  (resolves)
docs/user-guide/getting-started.en.md:176  ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.en.md:182  ../glossary.md                                                                                                                       ok  (resolves)
docs/user-guide/getting-started.en.md:183  ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/user-guide/getting-started.en.md:189  multi-rename.en.md                                                                                                                   ok  (resolves)
docs/user-guide/getting-started.en.md:190  ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/getting-started.en.md:191  ../glossary.md                                                                                                                       ok  (resolves)
docs/user-guide/multi-rename.de.md:3       multi-rename.en.md                                                                                                                   ok  (resolves)
docs/user-guide/multi-rename.de.md:11      ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/user-guide/multi-rename.de.md:12      ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/multi-rename.de.md:199     getting-started.de.md                                                                                                                ok  (resolves)
docs/user-guide/multi-rename.de.md:200     ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/multi-rename.de.md:201     ../glossary.md                                                                                                                       ok  (resolves)
docs/user-guide/multi-rename.en.md:3       multi-rename.de.md                                                                                                                   ok  (resolves)
docs/user-guide/multi-rename.en.md:10      ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/user-guide/multi-rename.en.md:11      ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/multi-rename.en.md:193     getting-started.en.md                                                                                                                ok  (resolves)
docs/user-guide/multi-rename.en.md:194     ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/multi-rename.en.md:195     ../glossary.md                                                                                                                       ok  (resolves)
tests/README.md:3                          ../docs/REQUIREMENTS.md                                                                                                              ok  (resolves)
tests/README.md:5                          ../docs/ACCEPTANCE-CRITERIA.md                                                                                                       ok  (resolves)
tests/security/README.md:3                 ../../docs/REQUIREMENTS.md                                                                                                           ok  (resolves)
tests/security/README.md:5                 ../../docs/security/THREAT_MODEL.md                                                                                                  ok  (resolves)

Markdown files scanned : 36
Repository links checked: 145
External links skipped : 20
Broken links           : 0
RESULT: PASS — every repository link resolves.
(exit code 0)
```
### 3.6 Belege für die Kriterien, die in dieser Sitzung ausführbar waren

Die folgenden Kriterien der `docs/ACCEPTANCE-CRITERIA.md` tragen den Marker `PASS` oder
`PARTIAL`, weil ihr Befehl hier tatsächlich gelaufen ist. Alle übrigen 88 Kriterien tragen
`NOT RUN` mit dem exakten Grund. Ein grep, der nichts findet, gibt keine Zeile aus — der
Exit-Code 1 in der Ausgabe ist das Zeichen dafür.

```console
$ grep -n "^license" Cargo.toml crates/*/Cargo.toml platform/*/Cargo.toml apps/*/Cargo.toml tests/Cargo.toml xtask/Cargo.toml
Cargo.toml:26:license = "MIT OR Apache-2.0"
crates/phocommander-archive/Cargo.toml:19:license.workspace = true
crates/phocommander-config/Cargo.toml:18:license.workspace = true
crates/phocommander-core/Cargo.toml:18:license.workspace = true
crates/phocommander-keymap/Cargo.toml:17:license.workspace = true
crates/phocommander-ops/Cargo.toml:16:license.workspace = true
crates/phocommander-remote/Cargo.toml:16:license.workspace = true
crates/phocommander-ui/Cargo.toml:16:license.workspace = true
crates/phocommander-viewer/Cargo.toml:15:license.workspace = true
platform/phocommander-platform/Cargo.toml:16:license.workspace = true
apps/phocommander-android/Cargo.toml:16:license.workspace = true
apps/phocommander-desktop/Cargo.toml:16:license.workspace = true
apps/phocommander-ios/Cargo.toml:16:license.workspace = true
tests/Cargo.toml:21:license.workspace = true
xtask/Cargo.toml:15:license.workspace = true
(exit code 0)
```
```console
$ grep -rn -i "license_key\|activation\|subscription\|paywall" --include="*.rs" .

(exit code 1)
```
```console
$ grep -rn -i "telemetry\|analytics\|sentry\|posthog\|amplitude\|mixpanel" --include="*.rs" .

(exit code 1)
```
```console
$ grep -nE "^name = \"[a-z0-9_-]*(rar|unrar|rarfile|rar5|unrar5)[a-z0-9_-]*\"" Cargo.lock

(exit code 1)
```
```console
$ grep -nE "^[a-z0-9_-]*(rar|unrar|rarfile|rar5|unrar5)[a-z0-9_-]* *= " Cargo.toml crates/*/Cargo.toml platform/*/Cargo.toml apps/*/Cargo.toml

(exit code 1)
```
```console
$ grep -rn "unwrap()\|expect(\|unsafe " --include="*.rs" .

(exit code 1)
```

**Wichtige Einschränkung:** In dieser Umgebung ist keine Rust-Toolchain installiert
(`cargo` ist nicht vorhanden). Die Gates, die Rust brauchen — `cargo deny check`,
`cargo audit`, `cargo clippy`, `cargo test`, `cargo build --locked` und die Benchmarks — konnten
deshalb **nicht** ausgeführt werden. Die betroffenen Kriterien tragen deshalb `NOT RUN` mit dem
Grund „no Rust toolchain is installed in this environment" und **kein** `PASS`. Der Verlauf der
M0-Abnahme unter `docs/acceptance/logs/` enthält Ausgaben früherer Läufe; sie sind in diesem
Dokument als Beleg zitiert, aber nicht von mir neu erzeugt.

---

## 4. Anforderungsmatrix

| ID | Titel | Meilenstein | Priorität | AK vorhanden |
|---|---|---|---|---|
| FR-01 | The application is free of charge and has no paid tier | M0 | Must | ja (AC-M0-01) |
| FR-02 | The application starts without an account, a login or a network connection | M0 | Must | ja (AC-M0-02) |
| FR-03 | No telemetry, analytics, advertising or crash reporting is collected | M0 | Must | ja (AC-M0-03) |
| FR-04 | The start view carries no banner, no promotional area and no onboarding overlay | M0 | Must | ja (AC-M0-04) |
| FR-05 | Every user-visible string is externalised and translatable | M2 | Must | ja (AC-M2-01) |
| FR-06 | The product ships under MIT OR Apache-2.0 without a proprietary component | M0 | Must | ja (AC-M0-05) |
| FR-10 | Two independent directory panels are shown side by side or stacked | M1 | Must | ja (AC-M1-01) |
| FR-11 | The arrangement can be switched at run time without a restart | M2 | Should | ja (AC-M2-02) |
| FR-12 | Each panel owns its tabs, and each tab keeps its own path, sorting and view state | M2 | Must | ja (AC-M2-03) |
| FR-13 | A tab can be moved from one panel to the other | M2 | Should | ja (AC-M2-04) |
| FR-14 | A directory tree can be toggled per panel | M2 | Should | ja (AC-M2-05) |
| FR-15 | A third area previews the marked file without opening it | M2 | Should | ja (AC-M2-06) |
| FR-16 | Operations act on the active panel; a transfer to the other panel is always explicit | M1 | Must | ja (AC-M1-02) |
| FR-17 | The column set is configurable: name, size, date, type | M1 | Must | ja (AC-M1-03) |
| FR-18 | The sort order is configurable and is remembered per tab | M1 | Must | ja (AC-M1-04) |
| FR-20 | Copy, move, rename, delete and create-directory are available | M1 | Must | ja (AC-M1-05) |
| FR-21 | Every operation runs as a background job and the interface stays responsive | M1 | Must | ja (AC-M1-06) |
| FR-22 | The conflict policy is configurable: ask, never, always, keep both | M1 | Must | ja (AC-M1-07) |
| FR-22a | The copy mode is selectable: quick, verified, atomic, sync-mirror, sync-update, sync-two-way, sync-preview | M1 | Must | ja (AC-M1-08) |
| FR-22b | The default copy mode is `verified`, the default synchronisation mode is `sync-preview` | M1 | Must | ja (AC-M1-09) |
| FR-22c | The `verified` mode compares the checksums of source and destination and reports a mismatch as a job error | M1 | Must | ja (AC-M1-10) |
| FR-22d | The `atomic` mode writes to a temporary file, verifies it and renames it into place | M2 | Must | ja (AC-M2-07) |
| FR-22e | `sync-mirror` is the only mode that deletes, and the deletion appears in the plan before it runs | M2 | Must | ja (AC-M2-08) |
| FR-22f | Every synchronisation produces a plan with the state per entry, and no data moves before the plan is confirmed | M2 | Must | ja (AC-M2-09) |
| FR-22g | The job cue shows state, progress, rate, estimate and the current item | M1 | Must | ja (AC-M1-11) |
| FR-22h | The verification phase has its own progress and is visible as a separate state | M1 | Must | ja (AC-M1-12) |
| FR-22i | A queued job can be reprioritised, retargeted, and its conflict and verification modes changed | M1 | Must | ja (AC-M1-13) |
| FR-22j | A running job can be paused, resumed and cancelled, and a change takes effect at the next item boundary | M1 | Must | ja (AC-M1-14) |
| FR-22k | The failed items of a job can be retried on their own | M2 | Should | ja (AC-M2-10) |
| FR-22l | The job list can be reordered, filtered by state and inspected | M2 | Should | ja (AC-M2-11) |
| FR-23 | Overwriting is not the default | M1 | Must | ja (AC-M1-15) |
| FR-24 | Every job reports progress with a rate and can be cancelled | M1 | Must | ja (AC-M1-16) |
| FR-25 | A job collects its errors instead of stopping at the first one | M1 | Should | ja (AC-M1-17) |
| FR-26 | Undo exists for copy operations and is labelled as partial | M2 | Should | ja (AC-M2-12) |
| FR-27 | The trash is the default path for deletion | M1 | Must | ja (AC-M1-18) |
| FR-28 | The path length limit of the platform is detected and reported in plain language | M3 | Must | nein (M ≥ 3) |
| FR-30 | The rename dialog offers placeholders for name, extension, parent, counter, date, time, size, short checksum, random value and case conversion | M1 | Must | ja (AC-M1-19) |
| FR-31 | The dialog offers search-and-replace, optionally with regular expressions | M1 | Must | ja (AC-M1-20) |
| FR-32 | The dialog offers a counter with start value, step and zero-padded width | M1 | Must | ja (AC-M1-21) |
| FR-33 | The rename result is deterministic: the same input always yields the same output | M1 | Must | ja (AC-M1-22) |
| FR-34 | The dialog shows a live preview of every new name before the rename runs | M1 | Must | ja (AC-M1-23) |
| FR-35 | Conflicts are marked and listed in a conflict jump list reachable with F3 and Shift+F3 | M1 | Must | ja (AC-M1-24) |
| FR-36 | Named templates can be stored and re-applied | M1 | Must | ja (AC-M1-25) |
| FR-37 | The series episode template detects season and episode in an existing name and offers them as pre-filled values | M1 | Must | ja (AC-M1-26) |
| FR-38 | The series name defaults to the parent directory, is always visible and can be overridden | M1 | Must | ja (AC-M1-27) |
| FR-39 | Season and episode numbers are zero-padded, and the width grows automatically with a three-digit number | M1 | Must | ja (AC-M1-28) |
| FR-40 | Characters the target platform forbids are reported before the rename and a replacement is suggested | M1 | Must | ja (AC-M1-29) |
| FR-41 | The rename is atomic per selection: either every file is renamed or none is | M2 | Should | ja (AC-M2-13) |
| FR-50 | The viewer opens in an additional area without leaving the application | M1 | Must | ja (AC-M1-30) |
| FR-51 | Text is shown with character set detection for UTF-8 and UTF-16 | M1 | Must | ja (AC-M1-31) |
| FR-52 | A hex dump shows offset, hex and printable ASCII | M1 | Must | ja (AC-M1-32) |
| FR-53 | Common image formats are displayed | M1 | Must | ja (AC-M1-33) |
| FR-54 | The contents of an archive can be browsed and a single entry extracted | M1 | Must | ja (AC-M1-34) |
| FR-55 | File metadata is shown: size, type, timestamps, checksum | M1 | Must | ja (AC-M1-35) |
| FR-56 | PDF text and page navigation are supported | M2 | Should | ja (AC-M2-14) |
| FR-57 | Syntax highlighting is supported | M2 | Should | ja (AC-M2-15) |
| FR-58 | Audio and video metadata are shown with a still frame | M9 | Could | nein (M ≥ 3) |
| FR-59 | The viewer is read-only; editing is a separate later feature | M1 | Must | ja (AC-M1-36) |
| FR-60 | The number of actually supported formats is stated truthfully in the about dialog | M2 | Must | ja (AC-M2-16) |
| FR-65 | An archive appears as a normal entry and opens as a virtual directory | M1 | Must | ja (AC-M1-37) |
| FR-66 | Archive entries can be listed, viewed and copied to the other panel without extracting first | M1 | Must | ja (AC-M1-38) |
| FR-67 | ZIP, TAR with GZip, BZip2, XZ and Zstd, and 7z are supported | M1 | Must | ja (AC-M1-39) |
| FR-68 | Archives can be created, including encrypted ones | M1 | Must | ja (AC-M1-40) |
| FR-69 | RAR and the exotic formats are not supported | M0 | Must | ja (AC-M0-06) |
| FR-70 | Every extraction goes through the hardened path resolution | M1 | Must | ja (AC-M1-41) |
| FR-71 | An archive can be tested for integrity | M2 | Should | ja (AC-M2-17) |
| FR-75 | The search matches name, size, date and content | M6 | Must | nein (M ≥ 3) |
| FR-76 | Text, hex and binary search are supported | M6 | Should | nein (M ≥ 3) |
| FR-77 | The boolean operators AND, OR and NOT are supported | M6 | Could | nein (M ≥ 3) |
| FR-78 | The content search runs in the background with a growing result list | M6 | Must | nein (M ≥ 3) |
| FR-79 | Two directories are compared for content, date and size | M2 | Must | ja (AC-M2-18) |
| FR-80 | The synchronisation transfers differences in both directions from a shown plan | M6 | Must | nein (M ≥ 3) |
| FR-85 | A remote source behaves like a local directory in every operation | M6 | Must | nein (M ≥ 3) |
| FR-86 | SFTP, FTP, SMB and WebDAV are supported | M6 | Must | nein (M ≥ 3) |
| FR-87 | S3-compatible targets are supported | M7 | Must | nein (M ≥ 3) |
| FR-88 | Cloud sign-in uses OAuth 2.0 directly against the provider | M7 | Must | nein (M ≥ 3) |
| FR-89 | The application operates no server of its own | M7 | Must | nein (M ≥ 3) |
| FR-90 | SMB 1 and NTLMv1 are not implemented | M6 | Must | nein (M ≥ 3) |
| FR-91 | SMB signing and SMB 3 transport encryption are required where the server supports them | M6 | Must | nein (M ≥ 3) |
| FR-92 | The WebDAV client is part of the project, not a copyleft dependency | M6 | Must | nein (M ≥ 3) |
| FR-93 | Every network operation has a timeout | M6 | Must | nein (M ≥ 3) |
| FR-94 | The response size is bounded independently of the announced content length | M6 | Must | nein (M ≥ 3) |
| FR-95 | A server that offers an older protocol version is refused, not silently accepted | M6 | Must | nein (M ≥ 3) |
| FR-100 | Every operation is bound to a named action, not to a physical key | M1 | Must | ja (AC-M1-42) |
| FR-101 | Keyboard, pointer, touch, D-pad and controller each bind to the same actions | M1 | Must | ja (AC-M1-43) |
| FR-102 | The action palette lists every action that is reachable in the current context | M8 | Must | nein (M ≥ 3) |
| FR-103 | The action palette is fully operable with a D-pad and the confirm key | M8 | Must | nein (M ≥ 3) |
| FR-104 | Focus and selection are separate states in the D-pad and controller contexts | M4 | Must | nein (M ≥ 3) |
| FR-105 | The multi-selection on a remote control uses an explicit selection mode | M8 | Must | nein (M ≥ 3) |
| FR-106 | The interface adapts structurally per context, not only in colour and font size | M2 | Must | ja (AC-M2-19) |
| FR-107 | Every action reachable with the keyboard is reachable with the D-pad | M8 | Must | nein (M ≥ 3) |
| FR-110 | The user chooses between a clean interface and a full interface | M2 | Must | ja (AC-M2-20) |
| FR-140 | The clean interface shows only the essential commands | M2 | Must | ja (AC-M2-21) |
| FR-141 | The full interface shows every command and every panel | M2 | Must | ja (AC-M2-22) |
| FR-142 | The choice can be made per platform and per device context | M2 | Should | ja (AC-M2-23) |
| FR-143 | Every action can be shown or hidden individually | M2 | Should | ja (AC-M2-24) |
| FR-144 | The interface profile can be exported and imported | M2 | Should | ja (AC-M2-25) |
| FR-145 | The interface profile contains no data beyond the settings it configures | M2 | Must | ja (AC-M2-26) |
| FR-120 | English and German are complete at the release of M2 | M2 | Must | ja (AC-M2-27) |
| FR-121 | The catalogue key set is identical in every language | M2 | Must | ja (AC-M2-28) |
| FR-122 | A missing translation falls back to English and then to the key, never to an empty string | M2 | Must | ja (AC-M2-29) |
| FR-123 | Numbers, sizes, dates and times follow the system locale | M2 | Must | ja (AC-M2-30) |
| FR-124 | The language can be changed at run time without a restart | M2 | Must | ja (AC-M2-31) |
| FR-125 | Adding a language requires no change to the source code | M2 | Must | ja (AC-M2-32) |
| FR-126 | The interface language and the content language are independent | M2 | Should | ja (AC-M2-33) |
| NFR-01 | A directory of 10,000 entries is on screen within 100 ms | M2 | Must | ja (AC-M2-34) |
| NFR-02 | Switching between loaded directories takes less than 16 ms | M2 | Must | ja (AC-M2-35) |
| NFR-03 | A key press produces a visible reaction within 50 ms | M2 | Must | ja (AC-M2-36) |
| NFR-04 | The application is usable within 800 ms of start on macOS | M2 | Must | ja (AC-M2-37) |
| NFR-05 | Idle memory with two panels open stays below 120 MB | M2 | Must | ja (AC-M2-38) |
| NFR-06 | A sequential copy reaches at least 80 % of the sequential read and write bandwidth | M2 | Must | ja (AC-M2-39) |
| NFR-07 | A name search across 100,000 files completes within 30 s | M6 | Must | nein (M ≥ 3) |
| NFR-08 | Listing an archive with 5,000 entries takes less than 500 ms | M2 | Must | ja (AC-M2-40) |
| NFR-09 | No operation that can take longer than 50 ms blocks the interface | M1 | Must | ja (AC-M1-44) |
| NFR-09a | The verification of a copy raises the read volume to at most twice the copied volume | M1 | Must | ja (AC-M1-45) |
| NFR-09b | The job cue updates at most ten times per second, and never blocks the interface | M1 | Must | ja (AC-M1-46) |
| NFR-10 | The whole operation is operable with the keyboard alone | M2 | Must | ja (AC-M2-41) |
| NFR-11 | Text contrast is at least 4.5:1, focus frames at least 3:1 | M2 | Must | ja (AC-M2-42) |
| NFR-12 | `prefers-reduced-motion` is respected | M2 | Must | ja (AC-M2-43) |
| NFR-13 | A screen reader can traverse the interface through AccessKit | M2 | Must | ja (AC-M2-44) |
| NFR-14 | No source file exceeds 400 lines soft or 550 lines hard | M0 | Must | ja (AC-M0-07) |
| NFR-15 | No `unsafe`, no `unwrap()` and no `expect()` on a reachable path | M0 | Must | ja (AC-M0-08) |
| NFR-16 | Every public function carries a WHAT, WHY, EXAMPLE doc comment | M0 | Must | ja (AC-M0-09) |
| NFR-17 | Every public function has at least one unit test | M1 | Must | ja (AC-M1-47) |
| NFR-18 | No dependency carries a copyleft licence | M0 | Must | ja (AC-M0-10) |
| NFR-19 | No dependency carries a known vulnerability | M0 | Must | ja (AC-M0-11) |
| NFR-20 | The crate dependency direction is `apps` → `platform` → `crates` | M0 | Must | ja (AC-M0-12) |
| NFR-21 | The build is reproducible from the committed lock file | M0 | Must | ja (AC-M0-13) |
| NFR-22 | The application is fully operable with the keyboard, the pointer and the touch | M2 | Must | ja (AC-M2-45) |
**Zusammenfassung der Matrix**

| Meilenstein | Anforderungen | davon mit AK | Abnahmekriterien |
|---|---|---|---|
| M0 | 13 | 13 | 13 |
| M1 | 47 | 47 | 47 |
| M2 | 45 | 45 | 45 |
| M3 | 1 | 0 | — |
| M4 | 1 | 0 | — |
| M6 | 14 | 0 | — |
| M7 | 3 | 0 | — |
| M8 | 4 | 0 | — |
| M9 | 1 | 0 | — |
| **Summe** | **129** | **105** | **105** |

Belegstatus der 105 Kriterien: 5 `PASS`, 12 `PARTIAL`, 88 `NOT RUN`.
Aufteilung nach Art: 74 automatisierbar, 31 manuelle Testschritte (jeder als `[manual]`
gekennzeichnet und mit einer exakt beschriebenen Bediensequenz).

---

## 5. Widerspruchsliste

Geprüft wurde die Spezifikation 2.0 gegen den Repository-Stand. Die folgenden Punkte sind
Inkonsistenzen — keine davon wurde stillschweigend korrigiert, weil `docs/REQUIREMENTS.md` dem
Owner gehört.

| # | Fundstelle | Widerspruch | Umgang in dieser Übergabe |
|---|---|---|---|
| W-1 | Spezifikation 9.2 gegen `docs/security/THREAT_MODEL.md` | Die Spezifikation führt 12 Härtungsregeln H-01…H-12, das Bedrohungsmodell 17 Regeln H1…H17 mit **anderer Zuordnung**. H-09 der Spezifikation ist „TLS-Zertifikate werden streng geprüft", im Bedrohungsmodell ist das H8. H-11 der Spezifikation ist „Antwortgröße ist begrenzt", im Bedrohungsmodell H15. H-12 der Spezifikation ist „Anmeldedaten im Schlüsselbund", im Bedrohungsmodell H10. | Die Spezifikation ist verbindlich; die Referenzen in den Kriterien nennen deshalb die Spezifikation (Abschnitt 9.2) und **keine** H-Nummer. Das Bedrohungsmodell muss nachgezogen werden — Owner-Entscheidung. |
| W-2 | Spezifikation 12.6 gegen den Repository-Stand | Die Dokumentationslandschaft nennt `docs/ARCHITECTURE.md`, `docs/reviews/` und `docs/guides/`. Im Repository existiert stattdessen `docs/architecture/README.md`; `docs/reviews/` und `docs/guides/` fehlen. | Nicht verändert. Der Linkcheck läuft grün, weil die vorhandenen Dokumente verlinkt sind. Die Namensabweichung ist eine Owner-Entscheidung. |
| W-3 | Spezifikation 7.5 gegen `config/defaults.toml` | 7.5 sagt: „Die Konfigurationsoberfläche hat vier Gruppen: appearance, panels, commands und input." Die ausgelieferte Konfigurationsdatei hat zehn Sektionen (general, appearance, panels, keymap, viewer, archives, privacy, advanced, input, remote), und `scripts/check-config-schema.py` prüft genau diese zehn. | Ungeklärt: ob 7.5 die Oberfläche der Einstellungen oder die ganze Konfigurationsdatei meint. Bis zur Entscheidung bleibt die Datei unverändert — Owner-Entscheidung. |
| W-4 | Spezifikation 11.2 (M0.1) gegen den Workspace | M0.1 verlangt „twelve crates". Der Workspace hat 14 Mitglieder (8 Crates unter `crates/`, `platform`, 3 Apps, `tests`, `xtask`), das Modulverzeichnis in 5.2 zählt 12 Module einschließlich `phocommander-i18n`, das es noch nicht gibt. | Als Begriffsklärung an den Owner: „zwölf" bezieht sich auf das Modulverzeichnis, nicht auf die Workspace-Mitglieder. |
| W-5 | Spezifikation 6.2 und FR-101 gegen `config/keymap.toml` | Die Spezifikation kennt fünf Bedienkontexte (keyboard, pointer, touch, dpad, controller). Der Aktionskatalog führt nur drei Kontextabschnitte (`input.keyboard`, `input.dpad`, `input.controller`); für pointer und touch gibt es keine eigene Tabelle. | Nicht verändert. `scripts/check-action-ids.py` prüft weiterhin die drei vorhandenen Abschnitte. Owner-Entscheidung: eigene Abschnitte für pointer und touch oder dokumentierte Vererbung. |
| W-6 | Spezifikation 4.10 gegen die Nummerierung | Die Nummernfolge ist thematisch, nicht lückenlos (FR-110, dann FR-140…FR-145). Die alte Nummerierung 1.1 hatte diese IDs mit anderen Bedeutungen. | Kein Widerspruch, sondern eine Entscheidung: die Nummern der Spezifikation 2.0 gelten, die alten Bedeutungen sind im Änderungsprotokoll des Katalogs als ersetzt ausgewiesen. |

---

## 6. Offene Punkte, die eine Owner-Entscheidung brauchen

| # | Frage | Warum sie offen ist | Vorschlag |
|---|---|---|---|
| O-1 | Wie werden die Härtungsregeln nummeriert — nach der Spezifikation (H-01…H-12) oder nach dem Bedrohungsmodell (H1…H17)? | Beide Dokumente sind im Repository, beide werden zitiert, die Nummern bedeuten verschiedene Dinge (W-1). | Das Bedrohungsmodell an die Nummerierung der Spezifikation angleichen; die zusätzlichen Regeln des Modells (H13…H17) als Unterregeln führen. Sonst ist jede Referenz auf „H-11" mehrdeutig. |
| O-2 | Gilt „four groups" in 7.5 für die Einstellungsoberfläche oder für die ganze Konfigurationsdatei? | Die ausgelieferte Datei hat zehn Sektionen und wird von einem Gate geprüft, das genau diese zehn erwartet (W-3). | 7.5 auf die Einstellungsoberfläche eingrenzen und die Datei als „the central configuration file" davon trennen. |
| O-3 | Bekommen `pointer` und `touch` eigene Abschnitte im Aktionskatalog? | FR-101 verlangt fünf Kontexte, die Datei führt drei (W-5). | Einen leeren, dokumentierten Vererbungsvermerk einführen: pointer und touch erben die Tastaturbelegung, bis eine eigene Tabelle nötig wird. |
| O-4 | Darf das `clean`-Profil die Statusleiste behalten? | Der Auftrag nennt für `clean` nur „zwei Panels, Pfad, Dateiliste, Eingabezeile"; das Beispiel in 7.2 der Spezifikation setzt `show_status_bar = true`, und die Statusleiste trägt den Job-Cue (6.10). | Die aktuelle Umsetzung folgt der Spezifikation (`show_status_bar = true`) und begründet das im Kommentar der Datei. Ein Veto des Owners würde eine zweite Datei `clean-minimal.toml` ergeben, nicht das Weglassen des Hireks. |
| O-5 | Bekommt das Profilschema eine Sektion `[platform.<name>]`? | ADR-0005 Entscheidung 6 und FR-142 verlangen eine plattformspezifische Auswahl; 7.2 zeigt keine solche Sektion. | Der Katalog `scripts/check-interface-profiles.py` erlaubt die optionale Sektion bereits. Die beiden ausgelieferten Profile verwenden sie nicht, weil sie die Vorgabe für jede Plattform sind. Die Entscheidung fällt mit der Implementierung der Auflösungsreihenfolge in M2. |
| O-6 | Wie heißen die beiden Prüfskripte im Katalog der Gates? | `scripts/check-requirement-ids.py` und `scripts/check-acceptance-ids.py` sind neu bzw. umgebaut, `scripts/check-interface-profiles.py` ist neu. Die CI-Datei `.github/workflows/ci.yml` führt sie noch nicht. | Die drei Gates in den `quality`-Job der CI aufnehmen. Ich habe die CI-Datei nicht angefasst, weil eine Änderung an der Gate-Liste eine Owner-Entscheidung ist. |
| O-7 | Wird das Akzeptanzdokument für M0 neu geschrieben? | `docs/acceptance/M0-ACCEPTANCE.md` ist der Beleg des Meilensteins M0 und wurde gegen die alte AK-Liste geschrieben. Die AK-IDs wurden neu vergeben. | Nach der Freigabe dieser Übergabe das Abnahmedokument gegen die neue AK-Liste neu ziehen; die Vorlage `docs/templates/M0-ACCEPTANCE.md` ist dafür vorbereitet. |

---

## 7. Was ich bewusst nicht getan habe

- `docs/REQUIREMENTS.md` ist unverändert. Die Spezifikation gehört dem Owner (Spezifikation 12.3,
  Regel 6).
- Kein Commit, kein Push, kein Tag. Die Übergabe erfolgt über die Dateien im Arbeitsbaum und
  dieses Dokument.
- Keine Rust-Datei angefasst. Die Änderungen an `crates/phocommander-core/`,
  `crates/phocommander-ops/` und `crates/phocommander-archive/` im Arbeitsbaum stammen aus einem
  parallelen Arbeitsstrang.
- Kein Kriterium als `PASS` markiert, dessen Befehl nicht gelaufen ist. 88 der 105 Kriterien
  tragen `NOT RUN` mit dem exakten Grund; das ist nach Abschnitt 13.2 der Spezifikation
  ausdrücklich zulässig und ehrlich.
- Die Inhalte des Repositorys sind englisch; nur dieses Übergabedokument ist deutsch.
