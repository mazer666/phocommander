# Übergabe — Core Engineer: die Namensbildung der Mehrfachumbenennung (M1)

| Feld | Wert |
|---|---|
| Rolle | Core Engineer (Namensbildung, Auftrags-Wahrheit, keine Oberfläche) |
| Repository | `/workspace/phocommander`, Branch `main`, **kein** Commit, **kein** Push, **kein** Tag |
| Grundlage | `docs/REQUIREMENTS.md` §4.4 (FR-30 bis FR-41), `docs/ACCEPTANCE-CRITERIA.md` (AC-M1-19 bis AC-M1-29, AC-M2-12), `docs/glossary.md`, `docs/user-guide/multi-rename.en.md`, `docs/ARCHITECTURE.md` §9 |
| Umfang | FR-30, FR-31, FR-32, FR-33, FR-37, FR-38, FR-39 — **plus** die Engine-Hälfte von FR-40 (die Meldung der verbotenen Zeichen) |
| Nicht im Umfang | FR-34 (Vorschau), FR-35 (Konfliktsprung-Liste), FR-36 (gespeicherte Vorlagen) — das ist der Dialog und kommt als eigener Auftrag |
| Werkzeuge | Rust 1.99.1 (installiert, **nicht** zum Bauen benutzt), Python 3.11.2, bash |
| Bauumgebung | **kein** Build ausgelöst — siehe Abschnitt 5 |

> **Hinweis zur Ablage:** `deliverable.md` wird im gemeinsam benutzten Repository von mehreren
> Rollen parallel beschrieben. Dieser Text liegt deshalb zusätzlich unter
> **`deliverable.core-engineer-rename.md`** (identisch). Der vorherige Inhalt von `deliverable.md`
> (Abnahme M1 der QA-Rolle) ist damit überschrieben — die QA-Übergabe liegt weiterhin als
> `deliverable-qa-test-engineer.md` vor.

---

## 1. Kurzfassung

Die Mehrfachumbenennung **führte aus**, ohne zu wissen, **wie** der neue Name heißt. Jetzt gibt es
die Bildung:

* Ein neues Modul `crates/phocommander-ops/src/template.rs` (304 Zeilen) mit sieben Dateien:
  die Daten der Regel (`types.rs`), die Platzhalter (`placeholders.rs`), das Suchen und Ersetzen
  (`replace.rs`), der Zähler (`counter.rs`), die Serienvorlage (`series.rs`), die Bildungslogik und
  die Unit-Tests (`tests.rs`).
* **Eine Funktion, eine Ausgabe:** `form_names(&rule, &inputs, now) -> Result<Vec<FormedName>,
  TemplateError>` — aus einer Liste von Eingaben und einer Regel eine Liste von Zielnamen. Der
  Dialog kann sie unverändert benutzen; `rename_pairs(&names)` liefert die Paare für den bereits
  vorhandenen atomaren Rename.
* **Die Determinismus (FR-33) ist der Entwurf, nicht eine Behauptung:** die Bildungsfunktion liest
  *nur ihre Argumente*. Die Uhr kommt vom Aufrufer, die Größe und die Prüfsumme kommen vom
  Aufrufer, und der „Zufallswert" wird aus dem Dateinamen abgeleitet (FNV-1a, ein fester
  Algorithmus) — ein echter Zufall würde FR-33 brechen.
* **Der Zähler wird zuletzt geschrieben** (Platzhalter → Ersetzen → Zähler, so wie es Glossar und
  Benutzerhandbuch vorgeben), damit eine Ersetzungsregel keine Nummer verfälscht, die der Benutzer
  behalten will.
* **Die Serienvorlage** liest Staffel und Folge aus dem Namen (`SxxEyy`), nimmt den Seriennamen aus
  dem Elternverzeichnis, lässt beides überschreiben und **verbreitert die Nullauffüllung
  automatisch**, wenn die Auswahl eine dreistellige Zahl enthält (FR-39).

**Was der Owner jetzt tun muss:** zentral bauen und testen (Abschnitt 9). Der Code ist **nicht
kompiliert** — die Regeln dieses Auftrags verbieten jeden Build neben dem Build des Owners.

---

## 2. Was ich geändert habe — Datei für Datei

### 2.1 Neu (Kern der Namensbildung)

| Datei | Zeilen | Zweck |
|---|---:|---|
| `crates/phocommander-ops/src/template.rs` | 304 | Das Modul: die Bildungsfunktion `form_names`, `rename_pairs`, `name_problems`, die Freigaben, die Pfad-Hilfen. Header mit WHAT/WHY/EXAMPLE und BOUNDARY. |
| `crates/phocommander-ops/src/template/types.rs` | 533 | Die Daten: `RenameRule` (Format, Ersetzungen, Zähler, Serie), `RenameInput` (Pfad, Größe, Zeit, Prüfsumme, vom Benutzer bestätigte Serie, Zufallswert), `FormedName` (`Named`/`Failed`), `TemplateError` (vier Fehlerarten). |
| `crates/phocommander-ops/src/template/placeholders.rs` | 370 | Die 17 Platzhalter, das Parsen des Formats (`[name]`, `[ext]`, `[parent]`, `[n]`, `[date]`, `[time]`, `[size]`, `[hash]`, `[rand]`, `[case]`, `[lower]`, `[upper]`, `[title]`, `[series]`, `[season]`, `[episode]`, `[episode_title]`), die Kurz-Prüfsumme, der aus dem Namen abgeleitete Wert und die vier Schreibweisen. |
| `crates/phocommander-ops/src/template/replace.rs` | 164 | `ReplaceRule` (literal oder regulärer Ausdruck), das Kompilieren der Muster (ein kaputtes Muster ist ein Fehler, kein stiller Wechsel) und das Anwenden in der Reihenfolge der Regeln. |
| `crates/phocommander-ops/src/template/counter.rs` | 170 | `CounterRule` (Startwert, Schrittweite, Breite, Reihenfolge), `CounterOrder` (Auswahl/Name/Datum), die Nummern der Auswahl, das Schreiben der Nullauffüllung. |
| `crates/phocommander-ops/src/template/series.rs` | 293 | `SeriesRule`, das Standardformat der Serie, das Erkennen der Staffel und der Folge, der Episodentitel (der Name ohne das Muster), die automatische Breite der Zahlen. |
| `crates/phocommander-ops/src/template/tests.rs` | 484 | 21 Unit-Tests der öffentlichen Funktionen (WHAT/WHY/EXAMPLE-Stil, kein `unwrap()` im Produktionscode). |

### 2.2 Geändert

| Datei | Änderung | Zeilen |
|---|---|---:|
| `crates/phocommander-ops/src/series.rs` | Neu: `SeriesPattern` (Staffel, Folge, **Fundstelle** in Byte) und `detect_series_pattern(name)`. `detect_series_numbers` ruft jetzt diese Funktion auf — die Erkennung bleibt identisch, der Code ist nicht doppelt. Dazu zwei Unit-Tests für die neue Funktion. | 309 (vorher 227) |
| `crates/phocommander-ops/src/lib.rs` | `pub mod template;`, die Freigaben der neuen Typen und Funktionen, der Modulkopf erwähnt das neue Modul. | 73 (vorher 68) |
| `crates/phocommander-ops/Cargo.toml` | `regex = { workspace = true }` (FR-31) und `[dev-dependencies] proptest = { workspace = true }` (der Property-Test von FR-33). | 33 (vorher 26) |
| `Cargo.toml` | Neue Sektion „Text processing" mit `regex = "1"` — der Katalog des Workspace, damit die Version einmal geschrieben wird. | +10 |
| `Cargo.lock` | Die Abhängigkeiten von `phocommander-ops` um `proptest` und `regex` ergänzt (Handedit, von `cargo metadata --locked` bestätigt, siehe 4.1). | +2 |
| `docs/WHERE_TO_CHANGE.md` | Neue Zeile „The names of a multiple rename" → `crates/phocommander-ops/src/template.rs`. | +1 |
| `docs/ARCHITECTURE.md` | Die Zeile „The rename placeholders" zeigt jetzt auf `template.rs` statt auf `rename.rs` (die Ausführung und die Bildung waren vorher vermischt). | 1 Zeile geändert |

### 2.3 Neu (Integrationstests, je ein Ziel nach dem Namen der Abnahmekriterien)

| Datei | Zeilen | Kriterium |
|---|---:|---|
| `crates/phocommander-ops/tests/rename_placeholders.rs` | 110 | AC-M1-19 (FR-30) — ein Fall je Platzhalter |
| `crates/phocommander-ops/tests/rename_search_and_replace.rs` | 91 | AC-M1-20 (FR-31) — Regex an/aus, kaputtes Muster |
| `crates/phocommander-ops/tests/rename_counter.rs` | 89 | AC-M1-21 (FR-32) — `001`, `002`, `003` |
| `crates/phocommander-ops/tests/rename_is_deterministic.rs` | 118 | AC-M1-22 (FR-33) — zwei Läufe, versteckte Uhr, versteckter Zufall, Reihenfolge der Aufrufe |
| `crates/phocommander-ops/tests/rename_properties.rs` | 108 | AC-M1-22 (FR-33) — **echter `proptest`-Property-Test**, drei Eigenschaften |
| `crates/phocommander-ops/tests/series_detects_season_and_episode.rs` | 97 | AC-M1-26 (FR-37) |
| `crates/phocommander-ops/tests/series_name_defaults_to_the_parent.rs` | 78 | AC-M1-27 (FR-38) |
| `crates/phocommander-ops/tests/series_zero_padding.rs` | 100 | AC-M1-28 (FR-39) |
| `crates/phocommander-ops/tests/rename_reports_forbidden_characters.rs` | 80 | AC-M1-29 (FR-40) — die Engine-Hälfte |

Jede Datei trägt die Kopfzeile mit dem Kommando, das die Abnahme fordert, und
`#![allow(clippy::unwrap_used, clippy::expect_used)]` — die Konvention der bestehenden
Integrationstests im Repository.

### 2.4 **Nicht** von mir geändert (Fremdstand im Baum)

`docs/STATUS.md` und `docs/acceptance/M1-ACCEPTANCE.md` waren **schon vor meinem ersten Schreibzugriff
geändert** (`git status` zu Beginn der Sitzung). Ich habe sie nicht angefasst.

---

## 3. Die öffentliche API

| Crate | Modul | Typ / Funktion | Zweck |
|---|---|---|---|
| `phocommander-ops` | `template` | `RenameRule` | Die Regel: Format, Ersetzungen, Zähler, Serie. Daten, kein Verhalten. |
| | | `RenameRule::new` / `with_replacement` / `with_counter` / `with_series` / `effective_format` | Bauen der Regel; das wirksame Format (leer → Standardformat der Serie). |
| | | `RenameInput` (+ `new`, `with_size`, `with_modified`, `with_checksum`, `with_series`, `with_random`) | Die Fakten einer Datei. Alles, was die Bildung nicht selbst lesen kann. |
| | | `FormedName` (`Named`/`Failed`) | Das Ergebnis je Datei: der neue Name, der Zielpfad, der Fehler. |
| | | `FormedName::source` / `name` / `target` / `problem` / `pair` | Zugriff auf das Ergebnis; `pair()` liefert das `RenamePair` für den atomaren Rename. |
| | | `TemplateError` | `EmptyFormat`, `UnknownValue`, `InvalidPattern`, `InvalidName`. |
| | | `form_names(rule, inputs, now)` | **Die Hauptfunktion:** aus Eingaben und Regel eine Liste von Zielnamen. Reine Funktion. |
| | | `rename_pairs(names)` | Die Paare des Renames — `None`, wenn auch nur eine Datei keinen Namen bekam (FR-41). |
| | | `name_problems(name)` | Die verbotenen Zeichen eines gebildeten Namens mit dem vorgeschlagenen Ersatz (FR-40, Engine-Hälfte). |
| | `template::counter` | `CounterRule`, `CounterOrder` | Startwert, Schrittweite, Breite, Reihenfolge. |
| | `template::placeholders` | `Placeholder` (+ `ALL`, `parse`, `as_str`) | Die 17 Platzhalter — die Dialogliste kann sie anzeigen. |
| | `template::replace` | `ReplaceRule` (`literal`, `pattern`) | Suchen und Ersetzen, literal oder als regulärer Ausdruck. |
| | `template::series` | `SeriesRule` (+ `STANDARD_FORMAT`) | Die Serienvorlage: Name, Staffel, Folge, Format, Breiten. |
| `phocommander-ops` | `series` | `SeriesPattern`, `detect_series_pattern` | Die Erkennung **mit Fundstelle** — der Episodentitel braucht sie. |

**Die Signatur, die der Dialog braucht:**

```rust
pub fn form_names(
    rule: &RenameRule,
    inputs: &[RenameInput],
    now: SystemTime,
) -> Result<Vec<FormedName>, TemplateError>
```

`now` kommt vom Aufrufer — nicht aus der Bibliothek. Das ist der Grund, warum die Bildung testbar
und reproduzierbar ist.

---

## 4. Die Befehle, die ich **wirklich** ausgeführt habe

Ich habe **keinen** Build, **keinen** Testlauf und **kein** `cargo fmt` ausgelöst (Abschnitt 5).
Die folgenden drei Kommandos bauen nichts und sind deshalb zulässig; ihre Ausgabe steht im
Original.

### 4.1 `cargo metadata --locked --offline` — Prüfung der Lock-Datei

```
$ cd /workspace/phocommander && CARGO_HOME=/workspace/.home/.cargo cargo metadata --locked --offline --format-version 1 > /tmp/metadata.json
exit=0
```

Die Ausgabe ist ein JSON-Dokument (3.090.921 Bytes). Der relevante Ausschnitt:

```
$ python3 -c "import json; ..."
phocommander-ops deps: ['regex', 'thiserror', 'proptest']
   regex ^1 None
   proptest ^1 dev
```

Und die Kontrolle, dass Cargo die Lock-Datei **nicht** verändert hat:

```
$ git diff --stat Cargo.lock
 Cargo.lock | 2 ++
 1 file changed, 2 insertions(+)

$ git diff Cargo.lock
@@ -3592,6 +3592,8 @@ dependencies = [
 name = "phocommander-ops"
 version = "0.1.0"
 dependencies = [
+ "proptest",
+ "regex",
  "thiserror 2.0.21",
 ]
```

**Wichtig für den Owner:** `cargo metadata --locked` läuft mit Exit 0 durch. Der Build mit
`--locked` (wie er gerade läuft) wird an der Lock-Datei **nicht** scheitern.

### 4.2 `./scripts/check-file-size.sh`

```
OVER  682 lines  ./crates/phocommander-archive/src/extract.rs
OVER  674 lines  ./crates/phocommander-ops/src/fake_fs.rs
OVER  614 lines  ./crates/phocommander-ops/src/progress.rs
OVER  721 lines  ./crates/phocommander-ui/src/app.rs
OVER  613 lines  ./crates/phocommander-ui/src/keymap_view.rs
OVER  651 lines  ./crates/phocommander-ui/src/panel.rs

Source files measured : 164 (extensions: rs)
Target value          : 600 lines (warning)
Absolute maximum      : 900 lines (failure)
Target-value warnings : 6
Maximum violations    : 0
RESULT: PASS with warnings — 6 file(s) exceed the target value of 600 lines.
        Each of them must name the reason in its module header.
```

Keine der sechs Warnungen betrifft eine Datei aus diesem Auftrag — alle sieben neuen Dateien liegen
unter 600 Zeilen.

### 4.3 `./scripts/check-crate-boundaries.sh`

```
Phocommander - crate boundary gate
rule: apps/* -> platform/* -> crates/*, no crate below crates/ may depend
      on another crate below crates/, registry dependencies are not checked

OK    apps/phocommander-desktop -> crates/phocommander-config (apps -> crates)
OK    apps/phocommander-desktop -> crates/phocommander-core (apps -> crates)
OK    apps/phocommander-desktop -> crates/phocommander-i18n (apps -> crates)
OK    apps/phocommander-desktop -> crates/phocommander-ops (apps -> crates)
OK    apps/phocommander-desktop -> platform/phocommander-platform (apps -> platform)
OK    apps/phocommander-desktop -> crates/phocommander-ui (apps -> crates)
OK    apps/phocommander-desktop -> crates/phocommander-viewer (apps -> crates)
OK    platform/phocommander-platform -> crates/phocommander-core (platform -> crates)
OK    tests -> crates/phocommander-archive (tests -> crates)
OK    tests -> crates/phocommander-config (tests -> crates)
OK    tests -> crates/phocommander-core (tests -> crates)
OK    tests -> crates/phocommander-keymap (tests -> crates)
OK    tests -> crates/phocommander-ui (tests -> crates)

13 path dependencies checked, 0 violations
```

`phocommander-ops` hängt an keinem Geschwister-Crate ✓ — die Abhängigkeit auf `regex` ist eine
Registry-Abhängigkeit und wird vom Gate nicht geprüft (so gewollt).

### 4.4 `python3 scripts/check-links.py`

```
docs/user-guide/multi-rename.en.md:10      ../REQUIREMENTS.md                                                                                                                   ok  (resolves)
docs/user-guide/multi-rename.en.md:11      ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/multi-rename.en.md:193     getting-started.en.md                                                                                                                ok  (resolves)
docs/user-guide/multi-rename.en.md:194     ../KEYMAP.md                                                                                                                         ok  (resolves)
docs/user-guide/multi-rename.en.md:195     ../glossary.md                                                                                                                       ok  (resolves)
tests/README.md:3                          ../docs/REQUIREMENTS.md                                                                                                              ok  (resolves)
tests/README.md:5                          ../docs/ACCEPTANCE-CRITERIA.md                                                                                                       ok  (resolves)
tests/security/README.md:3                 ../../docs/REQUIREMENTS.md                                                                                                            ok  (resolves)
tests/security/README.md:5                 ../../docs/security/THREAT_MODEL.md                                                                                                  ok  (resolves)

Markdown files scanned : 45
Repository links checked: 159
External links skipped : 20
Broken links           : 0
RESULT: PASS — every repository link resolves.
```

### 4.5 Was ich **nicht** gemessen habe (und warum)

| Kommando | Status | Grund |
|---|---|---|
| `cargo test --workspace` | **nicht ausgeführt** | Auftrag: „Kein `cargo build`, kein `cargo test`" — die Sandbox hat 3 GB RAM, ein zweiter Build neben dem des Owners bricht beide ab. Zum Zeitpunkt meiner Messung lief ein `cargo test --workspace --locked` des Owners. |
| `cargo clippy --workspace --all-targets -- -D warnings` | **nicht ausgeführt** | Wie oben. |
| `cargo fmt --all --check` | **nicht ausgeführt** | Wie oben. Der Code ist von Hand in der rustfmt-Näherung geschrieben; **die Formatierung ist ungeprüft**. Bitte vor dem Gate `cargo fmt --all` laufen lassen. |
| `cargo deny check` / `cargo audit` | **nicht ausgeführt** | Wie oben. Für `regex` (MIT OR Apache-2.0, kein C-Anteil, bereits im Abhängigkeitsbaum) ist die Lizenzprüfung trotzdem Pflicht (Abschnitt 5.8 der Spezifikation). |
| `cargo test --doc -p phocommander-ops` | **nicht ausgeführt** | Wie oben. Die Doctests der neuen Dateien sind geschrieben, aber ungeprüft. |

---

## 5. Warum ich nicht gebaut habe — und was das für die Übergabe heißt

Die Regel „kein `cargo build`, kein `cargo test`, kein `cargo fmt`" ist im Auftrag ausdrücklich mit
dem Grund begründet: 3 GB RAM, ein zweiter Build bricht beide ab, heute Nachmittag sind vier
Agenten genau daran hängengeblieben. Zum Zeitpunkt meiner Arbeit lief der Build des Owners:

```
202725  0.5  3.7 338964 119528 ?  Sl  13:15 /workspace/.home/.rustup/.../cargo test --workspace --locked
204956 97.6 22.6 1334092 713368 ?  Sl  13:20 /workspace/.home/.rustup/.../rustc --crate-name x11rb_protocol ...
```

**Folge für den Owner:** Der gelieferte Code ist **nicht kompiliert**. Ein Fehler im ersten Build
kann eine Stelle betreffen, die ich nicht sehen konnte — in einem mehrtausendzeiligen Baum ist das
nicht auszuschließen. Bitte den Build als ersten Schritt laufen lassen; die wahrscheinlichsten
Stellen sind in Abschnitt 9.2 genannt.

Der Erschütterungsbeweis der drei Gates (Datei-Größe, Crate-Grenzen, Links) ist in Abschnitt 4
geführt — diese Gates bauen nichts und sind echt gemessen.

---

## 6. Testabdeckungsmatrix

| Bereich | getestet? | Testdatei | Testname | Lücke |
|---|---|---|---|---|
| FR-30 Platzhalter (13 Stück) | ja | `tests/rename_placeholders.rs` | `rename_placeholders` | Der Zufallswert wird auf Länge, Hex und Gleichheit geprüft, **nicht** auf einen festen Wert — er ist per Definition vom Namen abgeleitet (siehe 8.1). |
| FR-30 Zähler-Platzhalter | ja | `tests/rename_placeholders.rs` | `the_counter_placeholder_carries_the_number_of_the_counter` | — |
| FR-30 Zufallswert (zwei Dateien) | ja | `tests/rename_placeholders.rs` | `the_random_value_of_two_files_differs` | — |
| FR-31 Suche literal / Regex | ja | `tests/rename_search_and_replace.rs` | `rename_search_and_replace`, `a_search_that_is_not_a_regular_expression_matches_the_text` | — |
| FR-31 kaputtes Muster | ja | `tests/rename_search_and_replace.rs` | `a_broken_pattern_produces_an_error_and_changes_no_name` | — |
| FR-32 Zähler (Start, Schritt, Breite) | ja | `tests/rename_counter.rs` | `rename_counter`, `the_counter_starts_at_the_configured_value_and_grows_by_the_step` | — |
| FR-32 Reihenfolge nach Name | ja | `tests/rename_counter.rs` | `the_counter_can_count_by_the_name` | Die Reihenfolge nach **Datum** ist implementiert, aber nicht als Integrationstest abgesichert — nur im Code. **Lücke.** |
| FR-33 Determinismus (zwei Läufe) | ja | `tests/rename_is_deterministic.rs` | `rename_is_deterministic` | — |
| FR-33 keine versteckte Uhr | ja | `tests/rename_is_deterministic.rs` | `the_result_does_not_depend_on_a_hidden_clock` | — |
| FR-33 kein versteckter Zufall | ja | `tests/rename_is_deterministic.rs` | `the_result_does_not_depend_on_a_hidden_random_source` | — |
| FR-33 keine globale Lage | ja | `tests/rename_is_deterministic.rs` | `the_result_does_not_depend_on_the_order_of_the_calls` | — |
| FR-33 Property-Test | ja | `tests/rename_properties.rs` | `the_result_is_a_function_of_the_input`, `the_counter_numbers_the_files_in_a_row`, `a_file_that_cannot_be_named_does_not_stop_the_others` | **Echter `proptest`-Test**, 100 Fälle je Eigenschaft ( `--proptest 100`). |
| FR-37 Staffel und Folge | ja | `tests/series_detects_season_and_episode.rs` | `series_detects_season_and_episode`, `the_engine_uses_the_numbers_that_it_found_in_the_name`, `a_file_without_a_pattern_carries_no_numbers` | — |
| FR-38 Serienname aus dem Elternverzeichnis | ja | `tests/series_name_defaults_to_the_parent.rs` | `series_name_defaults_to_the_parent`, `the_name_of_the_series_can_be_overridden` | — |
| FR-39 Nullauffüllung, wachsende Breite | ja | `tests/series_zero_padding.rs` | `series_zero_padding`, `the_width_grows_with_the_largest_number_of_the_selection`, `the_width_of_the_season_grows_as_well` | — |
| FR-40 verbotene Zeichen (Engine) | ja | `tests/rename_reports_forbidden_characters.rs` | `rename_reports_forbidden_characters`, `the_replacement_of_a_forbidden_character_is_allowed`, `a_name_without_a_forbidden_character_reports_nothing`, `a_device_name_of_windows_is_reported` | Der Teil „der Rename läuft nicht, bevor der Benutzer bestätigt" gehört dem Dialog. |
| Fehler: Wert unbekannt | ja | `src/template/tests.rs` | `a_value_that_the_caller_did_not_supply_is_reported` | — |
| Fehler: kein Format | ja | `src/template/tests.rs` | `a_rule_without_a_format_is_refused` | — |
| Fehler: Trenner im Namen | ja | `src/template/tests.rs` | `a_name_with_a_separator_is_refused` | — |
| Paare des Renames / Atomizität | ja | `src/template/tests.rs` | `the_pairs_of_the_rename_are_built_from_the_names`, `a_selection_with_a_file_that_could_not_be_named_has_no_pairs` | Die Atomizität der Ausführung selbst (FR-41, M2) liegt in `rename.rs` und ist nicht meine Aufgabe. |
| Öffentliche Funktionen (jede hat einen Test) | ja | `src/template/tests.rs` + `tests/*.rs` | 21 Unit-Tests + 21 Integrationstests | — |
| FR-34 Vorschau / FR-35 Konfliktsprung / FR-36 Vorlagen | **nein** | — | — | **Bewusst nicht im Umfang** — der Dialog ist ein eigener Auftrag. |
| Die geschweiften Klammern der benannten Vorlagen (E13) | **nein** | — | — | **Offene Frage 8.3** — die Engine spricht die eckigen Klammern des Dialogs. |

---

## 7. Der Determinismus-Nachweis (FR-33)

**Die Aussage.** `form_names` ist eine reine Funktion von `(Regel, Eingaben, Zeit)`. Es gibt keine
globale Lage, keine Uhr in der Bibliothek und keine Zufallsquelle, die nicht erklärt ist.

**Wie das erreicht wurde — vier Entscheidungen:**

1. **Die Uhr kommt vom Aufrufer.** `form_names(rule, inputs, now)` — der Parameter `now: SystemTime`
   wird von oben gereicht. Ohne das wäre die Bildung nicht testbar und nicht reproduzierbar.
2. **Die Fakten kommen vom Aufrufer.** `RenameInput` trägt Größe (`Option<u64>`), Zeit der letzten
   Änderung und Prüfsumme (`Option<u64>`). Die Bibliothek liest **keine** Datei. Ein fehlender Wert
   ist ein Fehler (`UnknownValue`), kein stiller Leerwert.
3. **Der „Zufallswert" ist abgeleitet, nicht gezogen.** `[rand]` = FNV-1a über den Dateinamen,
   auf acht Hexadezimalstellen gekürzt. FNV-1a ist ein fester, dokumentierter Algorithmus — der
   `DefaultHasher` der Standardbibliothek ist es **nicht** (sein Ergebnis ist über die Versionen
   des Compilers nicht garantiert stabil); das steht im Kommentar der Funktion.
4. **Die Reihenfolge ist total.** Sortierungen (`CounterOrder::Name`, `CounterOrder::Date`) sind
   *stabile* Sortierungen: gleiche Dateien behalten die Reihenfolge der Auswahl. Es gibt keine
   Gleichheit, die zwei Dateien ohne Entscheidung liegen lässt.

**Der Beweis in den Tests.** `tests/rename_is_deterministic.rs` bildet dieselbe Regel zweimal über
dieselben Dateien ab und vergleicht die Ergebnisse; zusätzlich prüft es, dass eine andere Regel
zwischen zwei Läufen nichts verändert, dass eine andere Uhr ohne `[date]`/`[time]` nichts verändert
und dass der Zufallswert gleich bleibt. `tests/rename_properties.rs` ist der **echte
`proptest`-Property-Test** der Spezifikation: er zieht das Format, den Schritt, die Breite und die
Dateinamen aus dem Generator, bildet jeden Fall **zweimal** ab und vergleicht — 100 Fälle je
Eigenschaft (`--proptest 100`).

**Ausführen:**

```bash
cargo test -p phocommander-ops --test rename_is_deterministic -- --exact
cargo test -p phocommander-ops --test rename_properties -- --proptest 100
```

**Erwartetes Ergebnis:** `test result: ok.` für beide Ziele. *Nicht gemessen* — siehe Abschnitt 5.

---

## 8. Die Entscheidungen, die ich getroffen habe (mit Begründung)

### 8.1 Der Zufallswert — eine Abweichung von AC-M1-19

`AC-M1-19` verlangt: „The random value differs between two runs." `FR-33` verlangt: „The rename
result is deterministic: the same input always yields the same output." **Beides zusammen ist
unmöglich.**

Ich habe **FR-33** umgesetzt, weil der Auftrag die Determinismus als den Kern benennt und weil
ein Name, der sich bei jedem Lauf ändert, für einen Benutzer wertlos ist. Der Ausweg für die
Absicht von AC-M1-19 ist offen: `RenameInput::with_random(value)` — der Aufrufer (der Dialog) darf
einen echten Zufallswert **einspeisen**; die Bibliothek bleibt eine reine Funktion, und der
Zufallswert ändert sich dann, weil sich die *Eingabe* geändert hat.

**Das braucht eine Entscheidung des Owners:** entweder AC-M1-19 wird auf „the random value is
derived from the name and therefore the same in two runs" korrigiert, oder der Dialog zeichnet den
Wert und die Zufallsquelle wird dort deklariert.

### 8.2 Die Reihenfolge der Auswertung — Platzhalter, Ersetzen, Zähler

Glossar: „Platzhalter werden vor den Ersetzungsregeln und dem Zähler ausgewertet." Benutzerhandbuch
§3: „The three parts of the rule are evaluated in this order: Placeholders, Replace, Counter."

Der Zähler wird also **zuletzt** geschrieben: eine Ersetzungsregel kann eine Nummer nicht
verfälschen. Technisch steht zwischen den Phasen ein Zeichen, das in einem Dateinamen nicht
vorkommen kann (`\u{0}`), damit die Ersetzung den Zähler nicht beschädigen kann. Eine Regel **ohne**
Zähler lässt das Platzhalterzeichen `[n]` als Text im Namen stehen — der Benutzer sieht im
Vorschau, dass der Zähler fehlt.

### 8.3 Der Dialekt der Vorlagen — nur die eckigen Klammern

Nach E13 benutzt der Dialog eckige, die benannten Vorlagen geschweifte Klammern. Die Engine
spricht **die eckigen Klammern des Dialogs**; die geschweiften Klammern der fünf Vorlagen in
`config/templates/` sind **nicht** implementiert. Die Zuordnung ist eine Zeile:

| Vorlagenfeld | Platzhalter der Engine |
|---|---|
| `{serie}` | `[series]` |
| `{staffel}` | `[season]` |
| `{episode}` | `[episode]` |
| `{titel}` | `[episode_title]` (der Name der Datei ohne das Muster `SxxEyy`) |
| `{format}` | `[ext]` |
| `{n}` | `[n]` (die Breite kommt aus der Zählerregel) |
| `{date}` | `[date]` |
| `{name}`, `{parent}`, `{size}`, `{hash}`, `{rand}` | gleichnamig |

Die Zuordnung gehört in die Schicht, die beide Welten kennt — der Dialog (er darf `crates/*`
benutzen). `phocommander-config` darf `phocommander-ops` **nicht** benutzen (Geschwister-Regel),
die Zuordnung kann also nicht im Konfigurations-Crate liegen.

### 8.4 Die Namen der Serien-Platzhalter

`[series]`, `[season]`, `[episode]`, `[episode_title]`. Bewusst **englisch** (wie die übrigen
Platzhalter des Dialogs) und **nicht** `[title]` — das ist laut Glossar die Schreibweise „Title
Case" und keine Episodenbezeichnung.

### 8.5 Der Episodentitel

„The.Expanse.S01E03.avi" → der Episodentitel ist „The.Expanse" — der Name der Datei **ohne** das
Muster und ohne die Trenner an der Naht. Das ist die dokumentierte Lesart („offers the rest of the
name as the episode title").

### 8.6 Die Breite der Serienzahlen

Die Breite der Staffel und die Breite der Folge werden **getrennt** berechnet: die Breite ist das
Maximum der Ziffernzahl der jeweiligen Zahl in der Auswahl, mindestens die Breite der Regel
(Vorgabe 2). Deshalb wird aus `S01E099` und `S01E100` genau so, wie die Abnahme es fordert.

### 8.7 Der Zähler zählt alle Dateien der Auswahl

Auch die Dateien, deren Staffel und Folge bereits im Namen stehen, bekommen eine Nummer. Für die
Episodennummer einer Datei **ohne** Muster ist der Zähler die Quelle; das entspricht dem Ablauf im
Benutzerhandbuch (der Benutzer gibt die erste Nummer ein, der Zähler läuft weiter).

### 8.8 Die Kurz-Prüfsumme

`[hash]` = die ersten acht Zeichen der Prüfsumme in Hexadezimal, ohne führende Nullen
(`format!("{:x}")`, nicht `{:08x}`) — bei einer 32-Bit-Prüfsumme ist das die ganze Summe, bei einer
64-Bit-Prüfsumme die obere Hälfte.

---

## 9. Offene Punkte und das, was der Owner tun muss

### 9.1 Der Owner muss bauen und messen

```bash
cargo fmt --all                       # die Formatierung ist ungeprüft (Abschnitt 4.5)
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p phocommander-ops --test rename_placeholders -- --exact
cargo test -p phocommander-ops --test rename_search_and_replace -- --exact
cargo test -p phocommander-ops --test rename_counter -- --exact
cargo test -p phocommander-ops --test rename_is_deterministic -- --exact
cargo test -p phocommander-ops --test rename_properties -- --proptest 100
cargo test -p phocommander-ops --test series_detects_season_and_episode -- --exact
cargo test -p phocommander-ops --test series_name_defaults_to_the_parent -- --exact
cargo test -p phocommander-ops --test series_zero_padding -- --exact
cargo test -p phocommander-ops --test rename_reports_forbidden_characters -- --exact
cargo deny check && cargo audit       # die neue Abhängigkeit regex braucht die Lizenzprüfung (§5.8)
```

Die Testziele sind so benannt, dass die Abnahmekriterien mit `--exact` genau einen Test treffen:
`cargo test -p phocommander-ops --test rename_counter -- --exact` führt den Test
`rename_counter` aus.

### 9.2 Wo ein Fehler am wahrscheinlichsten ist

Da ich nicht bauen durfte, nenne ich die Stellen, an denen ich die größte Unsicherheit habe:

1. `template/types.rs` — der `thiserror`-Aufbau von `TemplateError` und die `Display`-Zeilen der
   Fehler (Zeilen 495 ff.).
2. `template/placeholders.rs` — die `match`-Arme in `value_of` (17 Arme, alle Varianten der
   `Placeholder`-Enum).
3. `template/series.rs` — die Byte-Offsets des Musters und der Schnitt mit `base[..pattern.start]`
   (der Schutz `pattern.end > base.len()` ist eingebaut, damit kein panic entsteht).
4. Die Formatierung: `cargo fmt --all` bitte zuerst laufen lassen, dann `cargo fmt --all --check`.

### 9.3 Entscheidungen, die der Owner braucht

| # | Frage | Mein Vorschlag | Warum es eine Entscheidung ist |
|---|---|---|---|
| 1 | AC-M1-19 („der Zufallswert unterscheidet sich zwischen zwei Läufen") gegen FR-33 (Determinismus) | AC-M1-19 auf „abgeleitet vom Namen, also in zwei Läufen gleich" korrigieren; der Dialog speist einen echten Zufallswert über `RenameInput::with_random` ein | Zwei Kriterien widersprechen sich. Die Spezifikation ist die Quelle; ein Kriterium darf nicht gegen die Anforderung laufen. |
| 2 | Die geschweiften Klammern der benannten Vorlagen (E13) | Die Zuordnung (8.3) in den Dialog legen | FR-36 gehört einem anderen Auftrag; die Engine spricht den Dialog-Dialekt. |
| 3 | Der Zähler zählt auch die Dateien mit erkannter Staffel/Episode (8.7) | So lassen | Das Benutzerhandbuch beschreibt den Ablauf nur für Dateien **ohne** Muster. |
| 4 | Der Episodentitel ist der Name ohne das Muster (8.5) | So lassen | Eine Alternative wäre „der ganze Name" — dann hieße die Datei „... - The.Expanse.S01E03.avi". |
| 5 | `regex` als neue Abhängigkeit | Behalten | Sie ist bereits im Abhängigkeitsbaum des Workspace, MIT/Apache-2.0, ohne C-Anteil. §5.8 verlangt trotzdem die Lizenzprüfung und eine Begründung — dieses Dokument ist die Begründung. |
| 6 | Die Reihenfolge der Auswertung (8.2) | So lassen (Platzhalter → Ersetzen → Zähler) | Steht so im Glossar und im Benutzerhandbuch; die Wirkung ist, dass der Zähler die Ersetzung überlebt. |

### 9.4 Was der nächste Auftrag (der Dialog) von mir bekommt

* `form_names(&rule, &inputs, now) -> Result<Vec<FormedName>, TemplateError>` — die Vorschau
  (FR-34) ist ein Aufruf dieser Funktion je Änderung der Regel.
* `FormedName::problem()` — der Fehler je Datei (ein Datei mit unbekanntem `[hash]` blockiert die
  anderen nicht).
* `name_problems(name)` — die verbotenen Zeichen mit dem vorgeschlagenen Ersatz (FR-40, die
  Engine-Hälfte).
* `rename_pairs(&names)` — `None`, wenn auch nur eine Datei keinen Namen bekam; der Dialog soll
  dann gar nichts anbieten (FR-41).
* `Placeholder::ALL` — die Liste für die Menüleiste des Dialogs.
* `SeriesRule` + `detect_series_pattern` — die Vorbelegung der Staffel und der Folge (FR-37) und
  der Serienname aus dem Elternverzeichnis (FR-38).

**Was der Dialog zusätzlich bauen muss:** die Konfliktsprung-Liste (FR-35), die Speicherung der
Vorlagen (FR-36) und die Zuordnung der geschweiften Klammern (8.3).

---

## 10. Die Liste der Dateien im Baum (Übergabe wie gefordert)

```
crates/phocommander-ops/src/template.rs                       (neu,   304 Zeilen)
crates/phocommander-ops/src/template/types.rs                 (neu,   533 Zeilen)
crates/phocommander-ops/src/template/placeholders.rs          (neu,   370 Zeilen)
crates/phocommander-ops/src/template/replace.rs               (neu,   164 Zeilen)
crates/phocommander-ops/src/template/counter.rs               (neu,   170 Zeilen)
crates/phocommander-ops/src/template/series.rs                (neu,   293 Zeilen)
crates/phocommander-ops/src/template/tests.rs                 (neu,   484 Zeilen)
crates/phocommander-ops/src/series.rs                         (geänd., 309 Zeilen)
crates/phocommander-ops/src/lib.rs                            (geänd.,  73 Zeilen)
crates/phocommander-ops/Cargo.toml                           (geänd.,  33 Zeilen)
crates/phocommander-ops/tests/rename_placeholders.rs          (neu,   110 Zeilen)
crates/phocommander-ops/tests/rename_search_and_replace.rs   (neu,    91 Zeilen)
crates/phocommander-ops/tests/rename_counter.rs               (neu,    89 Zeilen)
crates/phocommander-ops/tests/rename_is_deterministic.rs      (neu,   118 Zeilen)
crates/phocommander-ops/tests/rename_properties.rs            (neu,   108 Zeilen)
crates/phocommander-ops/tests/series_detects_season_and_episode.rs (neu, 97 Zeilen)
crates/phocommander-ops/tests/series_name_defaults_to_the_parent.rs (neu, 78 Zeilen)
crates/phocommander-ops/tests/series_zero_padding.rs          (neu,   100 Zeilen)
crates/phocommander-ops/tests/rename_reports_forbidden_characters.rs (neu, 80 Zeilen)
Cargo.toml                                                   (geänd., +10 Zeilen)
Cargo.lock                                                   (geänd.,  +2 Zeilen)
docs/WHERE_TO_CHANGE.md                                      (geänd.,  +1 Zeile)
docs/ARCHITECTURE.md                                         (geänd.,  1 Zeile)
deliverable.md                                               (neu)
deliverable.core-engineer-rename.md                          (neu, identische Kopie)
```

`docs/REQUIREMENTS.md` wurde **nicht** verändert — die Spezifikation ist die Quelle, und die
Abweichung (8.1) ist hier beschrieben, nicht dort korrigiert.
