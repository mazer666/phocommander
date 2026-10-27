# Übergabe — UI Engineer: der Dialog der Mehrfachumbenennung (M1)

| Feld | Wert |
|---|---|
| Rolle | UI Engineer (Dialog der Mehrfachumbenennung: Vorschau, Konfliktsprung-Liste, gespeicherte Vorlagen) |
| Repository | `/workspace/phocommander`, Branch `main`, **kein** Commit, **kein** Push, **kein** Tag |
| Grundlage | `docs/REQUIREMENTS.md` §4.4 (FR-34, FR-35, FR-36), `docs/WHERE_TO_CHANGE.md`, `docs/KEYMAP.md` |
| Umfang | FR-34 (Live-Vorschau), FR-35 (Konfliktsprung-Liste mit F3/Shift+F3), FR-36 (benannte Vorlagen) — plus die Naht zur Engine |
| Nicht im Umfang | die Ausführung der Umbenennung (der Befehl `Command::Rename` wird erzeugt, aber die Desktop-App führt ihn noch nicht aus — das ist die Arbeit der Kern-Rolle), die Wiederherstellung des Originalnamens bei einem Einzelfehler, FR-40 (die Meldung der verbotenen Zeichen in der Oberfläche) |
| Werkzeuge | Python 3.11.2, bash — **kein** `cargo build`, **kein** `cargo test`, **kein** `cargo fmt` (Bau-Gate, siehe Abschnitt 7) |
| Bauumgebung | **kein Build ausgelöst** — der Code ist **nicht übersetzt und nicht gelaufen** |

> **Hinweis zur Ablage:** `deliverable.md` wird im gemeinsam benutzten Repository von mehreren
> Rollen parallel beschrieben. Dieser Text liegt deshalb zusätzlich unter
> **`deliverable.ui-engineer-rename.md`** (identisch). Der vorherige Inhalt von `deliverable.md`
> (die Übergabe der Core-Rolle) bleibt unter `deliverable.core-engineer-rename.md` erhalten.

---

## 1. Was ich gebaut habe

Die Engine der Namensbildung stand und ist getestet (`crates/phocommander-ops/src/template.rs`).
Was fehlte, war die Oberfläche: ein Dialog, in dem der Benutzer die Regel sieht, die Vorschau
live verfolgt, die Konflikte findet und die Regeln unter einem Namen speichert.

### 1.1 Die Naht: `RenameEngine` und `TemplateStore` (Traits in der UI-Crate)

`docs/WHERE_TO_CHANGE.md` beantwortet die Crate-Frage: *„Does it need to know about pixels,
widgets or events? → `phocommander-ui`“* und *„no crate below `crates/` may depend on a sibling
crate“*. Die UI-Crate darf `phocommander-ops` also nicht kennen — genau wie bei `Catalogue`
und `Formats` ist die Naht ein **Trait in der UI-Crate**, die **Implementierung in der
Composition Root** (`apps/phocommander-desktop/src/rename.rs`).

```rust
// crates/phocommander-ui/src/rename/engine.rs
pub trait RenameEngine {
    fn preview(&self, request: &RenameRequest) -> RenamePreview;
}

// crates/phocommander-ui/src/rename/template.rs
pub trait TemplateStore {
    fn templates(&self) -> Result<Vec<NamedTemplate>, TemplateStoreError>;
    fn store(&self, template: &NamedTemplate) -> Result<(), TemplateStoreError>;
}
```

**Warum so und nicht anders:** die Regel des Dialogs und die Datei-Fakten sind *Daten*, keine
Verhaltensweise. Die UI-Crate braucht sie, um die Felder zu zeigen, und sie braucht die *Regel*
des Dialogs, um sie an die Engine zu geben. Hätte die UI-Crate die Typen von `phocommander-ops`
importiert, wäre die Grenze gebrochen (`scripts/check-crate-boundaries.sh` wäre rot). Mit dem
Trait trägt die UI-Crate ihre eigene Regel (`crates/phocommander-ui/src/rename/rule.rs`) und
die Composition Root übersetzt sie (`rule_of`). Das ist dasselbe Muster wie bei `Catalogue`
(Schlüssel) und `Formats` (Locale-Regeln).

**Die Vorschau muss billig sein.** Zwei Entscheidungen:

1. **Der Dialog fragt die Engine nur, wenn sich die Regel geändert hat.** `RenameDialog`
   hält ein `dirty`-Flag; `update_preview()` fragt die Engine genau einmal pro Änderung und
   cached das Ergebnis. Ein Frame, der eine unveränderte Regel zeichnet, kostet **nur das
   Zeichnen** — der Zähler `engine_calls()` beweist das im Test.
2. **Die Naht ist eine Frage, keine Rückfrage.** Der Trait gibt die *ganze* Vorschau in einem
   Rutsch zurück (Zeilen **und** Konflikte), damit die UI nicht zweimal fragt.

**Bekannte Kostengrenze (siehe Abschnitt 8):** die Konfliktplanung des Ops-Crates
(`plan_renames`) ist O(n²) und ruft das Dateisystem 2n-mal auf. Bei sehr großen Auswahlen
dominiert sie den Tastendruck. Ich habe das Ops-Crate **nicht** angefasst (es gehört der
Core-Rolle) — der Vorschlag steht in Abschnitt 8.

### 1.2 FR-34 — die Live-Vorschau

`crates/phocommander-ui/src/rename/dialog.rs` (Zustand) und `crates/phocommander-ui/src/rename/draw.rs`
(Zeichnung). Die Vorschau zeigt **jede** Datei der Auswahl mit ihrem neuen Namen; eine Datei,
deren Name nicht gebildet werden kann, zeigt den Grund in der Warnfarbe.

* Die Vorschau entsteht aus **denselben Feldern**, die der Benutzer bearbeitet (`Fields` →
  `rule()` → `RenameRequest`). Eine Regel und die Felder können nicht auseinanderlaufen.
* Nur der **sichtbare Ausschnitt** wird gezeichnet (`VISIBLE_ROWS = 12`), der Cursor wird in
  das Fenster gescrollt (`ensure_cursor_visible`).
* Die Vorschau folgt **jedem Tastendruck**: `insert()`/`backspace()` setzen das `dirty`-Flag,
  und die nächste Frame fragt die Engine.

### 1.3 FR-35 — die Konfliktsprung-Liste

* Der Dialog baut aus den Konflikten der Engine eine Liste von **Zeilenindizes** (`conflicts()`).
  Eine Zeile ist markiert, wenn ein Konflikt ihren Zielpfad **oder** ihren Quellpfad nennt.
* **F3** und **Shift+F3** sind die Aktionen `rename.conflict_next` und
  `rename.conflict_previous` — sie stehen seit dem M1-Katalog in `config/keymap.toml`
  (Tastaturkontext) und in `docs/KEYMAP.md`. Die Auflösung läuft über die Bindungstabelle
  (`KeymapView::binding_for`), nicht über eine fest verdrahtete Taste: „the surface with the
  focus decides“ (`docs/KEYMAP.md`, Zeile 88ff.).
* **Am Ende der Liste bleibt der Cursor auf dem letzten Eintrag** — kein Wrap-around
  (so verlangt es AC-M1-24).
* Ist ein Konflikt da, **darf die Umbenennung nicht laufen**: die Umbenennung ist atomar
  (FR-41), also wird die ganze Auswahl verweigert (`pairs()` liefert `None`).

### 1.4 FR-36 — die benannten Vorlagen

* Der Dialog hält die Liste der Vorlagen (`templates()`), kann eine Vorlage anwenden
  (`apply_template()` — die Regel erscheint in den Feldern, damit der Benutzer sie sieht) und
  kann die aktuelle Regel unter einem Namen speichern (`current_template()` + `add_template()`).
* Ein Name steht für **eine** Regel: ein zweiter Vorlage mit demselben Namen ersetzt das erste.
* **Wo die Vorlagen liegen**, entscheidet die Composition Root: `FileTemplateStore` schreibt
  `rename-templates.txt` in das Konfigurationsverzeichnis. Das **Format** der Datei gehört der
  UI-Crate (`encode_templates`/`decode_templates`), damit die Composition Root nur Bytes
  bewegt und keine Datei neu erfindet.
* Das Format ist bewusst schlicht und **streng**: eine unbekannte Zeile ist ein Fehler
  (`key-unknown:<key>`), kein stillschweigendes Ignorieren — dieselbe Haltung wie bei der
  Konfigurationsdatei.

### 1.5 Der Dialog in der Shell

`crates/phocommander-ui/src/app/rename.rs`:

* **F2** (`panel.rename`) öffnet den Dialog mit den markierten Einträgen des Panels, das den
  Fokus hat (ohne Markierung: der Eintrag unter dem Cursor). Das Panel, nicht das andere:
  FR-16.
* Solange der Dialog offen ist, **erreicht keine Taste das Panel und den Baum** — der Dialog
  ist ein Fenster über der Oberfläche.
* **Enter** bedeutet je nach Fokus: im Namensfeld → Vorlage speichern; in der Vorlagenliste →
  Vorlage anwenden; im Umschalter → Suchen/ Ersetzen auf regulären Ausdruck umschalten; sonst →
  die Umbenennung bestätigen.
* **Esc** schließt den Dialog, ohne etwas umzubenennen.
* **Tab/Shift+Tab** wandern in einer **festen Reihenfolge** durch die Teile des Dialogs
  (deterministische Fokusreihenfolge, Abschnitt 6.4 der Spezifikation).
* Die Pfeiltasten bewegen den Cursor in der Vorschau und in der Vorlagenliste, den Cursor im
  Textfeld.

---

## 2. Das Zustandsmodell des Dialogs

| Struktur | Felder | Übergänge | Fehlerfälle |
|---|---|---|---|
| `RenameDialog` (Zustand, außerhalb von egui) | `side: PanelSide`, `files: Vec<RenameFile>`, `fields: Fields`, `focus: RenameFocus`, `preview: Option<RenamePreview>`, `dirty: bool`, `engine_calls: usize`, `cursor: usize`, `scroll: usize`, `conflicts: Vec<usize>`, `templates: Vec<NamedTemplate>`, `template_cursor: usize` | `new()` → offen · `set_format`/`insert`/`backspace`/`delete`/`set_regex` → `dirty = true` → `update_preview(engine)` fragt **einmal** die Engine und baut `conflicts` neu · `set_focus`/`focus_next` → Fokus wandert, der Cursor des Textfeldes springt ans Ende · `move_cursor`/`move_template_cursor` → Cursor wandert · `jump_to_next_conflict`/`jump_to_previous_conflict` → Cursor auf den nächsten/vorherigen Konflikt · `apply_template`/`add_template` → Vorlage in die Felder bzw. in die Liste · `pairs()` → der Befehl `Command::Rename` | `preview == None` → noch keine Vorschau, `pairs()` liefert `None` · Konflikt in der Liste → `pairs()` liefert `None` (FR-41, atomar) · Zeile ohne Namen → `pairs()` liefert `None` · Regel ohne Format → die Engine antwortet mit `problem`, die Vorschau ist leer · kaputtes Suchmuster → `problem` = `InvalidPattern` · Zählerfeld ohne Zahl → das Feld wird in der Warnfarbe gezeichnet, die Regel nutzt den Standardwert |
| `Fields` (die Felder der Regel) | `format`, `search`, `replacement`, `regex`, `counter_start`, `counter_step`, `counter_width`, `series_name`, `template_name`, `caret` | `insert`/`backspace`/`delete` → Text und Cursor · `move_caret`/`caret_to_start`/`caret_to_end` → Cursor · `apply_rule` → die Regel einer Vorlage in die Felder · `rule()` → die Regel, die die Engine bekommt | Zählerfeld ohne Zahl → `is_invalid()` meldet es, `rule()` nutzt den Standardwert · Suchtext leer → keine Ersetzungsregel in der Regel |
| `RenameFocus` (der Fokus) | 11 Teile: Format, Search, Replacement, Regex, CounterStart, CounterStep, CounterWidth, SeriesName, TemplateName, Templates, Preview | `focus_next(backwards)` → der nächste Teil in der festen Reihenfolge | — |
| `RenamePreview` (die Antwort der Engine) | `rows: Vec<PreviewRow>`, `conflicts: Vec<Conflict>`, `problem: Option<PreviewProblem>` | `may_rename()` → nur wenn kein Konflikt, kein `problem` und jede Zeile einen Namen hat | `problem` = EmptyFormat / UnknownValue / InvalidPattern / InvalidName |
| `PreviewRow` | `source`, `target: Option<PathBuf>`, `name: Option<String>`, `problem: Option<PreviewProblem>` | `old_name()` → der alte Name für die Anzeige | `name == None` → die Zeile trägt den Grund |
| `Conflict` | `kind: ConflictKind`, `path: PathBuf` | `message_key()` → der Text aus dem Katalog | TargetExists / DuplicateTarget / MissingSource |

**Ein Fehlerfall, den ich bewusst entschieden habe:** eine Datei, die ihren Namen behält
(Ziel == Quelle), ist **kein** Paar und wird **nicht** umbenannt. `plan_renames` des
Ops-Crates meldet für ein solches Paar „target-exists“ — der Dialog würde dann jede Datei als
Konflikt zeigen, deren Regel sie nicht ändert. Die Composition Root lässt solche Paare aus dem
Plan (`pairs_of`) und der Dialog ebenfalls (`pairs()`), damit die Anzeige und der Plan
übereinstimmen.

---

## 3. Die Tastenbelegung des Dialogs

| Vorgabe | Aktion | Kontext | plattformabhängig? | implementiert? | Testname |
|---|---|---|---|---|---|
| FR-35 / AC-M1-24 | `rename.conflict_next` (F3) — zum nächsten Konflikt | multiple-rename dialog | nein (der Schlüssel ist in der Bindungstabelle) | ja | `the_conflict_jump_list_is_reachable_with_f3_and_shift_f3` (egui_kittest) + `the_next_conflict_is_reachable_and_the_end_of_the_list_stays` (Unit) |
| FR-35 / AC-M1-24 | `rename.conflict_previous` (Shift+F3) — zum vorherigen Konflikt | multiple-rename dialog | nein | ja | `the_conflict_jump_list_is_reachable_with_f3_and_shift_f3` + `the_previous_conflict_is_reachable` |
| FR-35 / AC-M1-24 | am Ende der Liste bleibt der Cursor auf dem letzten Eintrag | multiple-rename dialog | nein | ja | `the_next_conflict_is_reachable_and_the_end_of_the_list_stays` |
| Tastaturführung | `panel.rename` (F2) — den Dialog öffnen | panel | nein | ja | `the_conflict_jump_list_is_reachable_with_f3_and_shift_f3` |
| Tastaturführung | Esc — den Dialog schließen, ohne umzubenennen | multiple-rename dialog | nein | ja | (manuell, kein Test) |
| Tastaturführung | Enter — Vorlage speichern (im Namensfeld), Vorlage anwenden (in der Vorlagenliste), Umbenennung bestätigen (sonst) | multiple-rename dialog | nein | ja | `a_stored_template_can_be_applied_again` (Teil) |
| Tastaturführung | Tab / Shift+Tab — der Fokus wandert in fester Reihenfolge | multiple-rename dialog | nein | ja | `the_focus_walks_the_dialog_in_a_fixed_order`, `the_backward_focus_stays_at_the_beginning` |
| Tastaturführung | Pfeiltasten — Cursor in der Vorschau und in der Vorlagenliste; Textcursor im Textfeld | multiple-rename dialog | nein | ja | (manuell) |
| Tastaturführung | Buchstaben/Ziffern — der Text in das Feld, das den Fokus hat | multiple-rename dialog | nein (die Tastaturbelegung des Systems) | ja | `the_preview_follows_every_change_of_the_rule` |
| FR-31 | der Umschalter „regulärer Ausdruck" | multiple-rename dialog | nein | ja (Enter im Fokus `Regex`, und die Maus über das Kästchen) | `the_preview_follows_every_change_of_the_rule` (der Umschalter ändert die Regel) |
| NFR-11 | sichtbare Fokusmarkierung (Rahmen um das Feld, um den Umschalter, um die Zeile) | multiple-rename dialog | nein (Farbe und Stärke kommen aus `Palette`/`Metrics`) | ja | `the_warning_reaches_the_contrast_the_specification_names` (Farbe), manuell (Rahmen) |

**Die Maus ist optional**: jede Aktion des Dialogs ist per Tastatur erreichbar; die Maus ist
zusätzlich möglich (Umschalter, Klick in ein Feld).

---

## 4. Die Dateiliste

| Pfad | Zeilen | Zweck |
|---|---|---|
| `crates/phocommander-ui/src/rename.rs` | 53 | der Modul-Index: die Module und die öffentlichen Namen |
| `crates/phocommander-ui/src/rename/dialog.rs` | 540 | der Zustand des Dialogs: Felder, Fokus, Vorschau, Sprungliste, Vorlagen |
| `crates/phocommander-ui/src/rename/focus.rs` | 105 | die Teile des Dialogs in der festen Reihenfolge des Tab-Keys |
| `crates/phocommander-ui/src/rename/fields.rs` | 287 | die Puffer der Regel, der Cursor, das Lesen und Schreiben des Textes |
| `crates/phocommander-ui/src/rename/engine.rs` | 576 | die Naht `RenameEngine` und die Typen der Antwort (Vorschau, Zeile, Problem, Konflikt, Paar) |
| `crates/phocommander-ui/src/rename/engine/tests.rs` | 83 | die Tests der Naht |
| `crates/phocommander-ui/src/rename/rule.rs` | 441 | die Regel des Dialogs: Format, Ersetzung, Zähler, Serie |
| `crates/phocommander-ui/src/rename/template.rs` | 526 | die benannten Vorlagen, die Naht `TemplateStore`, das Textformat der Datei |
| `crates/phocommander-ui/src/rename/draw.rs` | 425 | die Zeichnung: Felder, Umschalter, Vorlagenliste, Vorschau, Hinweiszeile |
| `crates/phocommander-ui/src/rename/tests.rs` | 357 | die Tests des Zustands (Vorschau, Sprungliste, Vorlagen, Fokus) |
| `crates/phocommander-ui/src/app/rename.rs` | 380 | die Anbindung an die Shell: öffnen, Tasten, bestätigen, zeichnen |
| `crates/phocommander-ui/src/app.rs` | 748 | die Felder der Shell (`rename`, `rename_engine`, `template_store`), der Aufruf von `panel.rename`, der Aufruf des Zeichnens |
| `crates/phocommander-ui/src/app/input.rs` | 275 | die Tasten und der Text gehen an den Dialog, solange er offen ist |
| `crates/phocommander-ui/src/commands.rs` | 154 | `Command::Rename` trägt jetzt die **Paare** (Quelle → Ziel) statt nur der Pfade |
| `crates/phocommander-ui/src/i18n/keys.rs` | 563 | 22 neue Übersetzungsschlüssel (`rename-*`) |
| `crates/phocommander-ui/src/theme.rs` | 307 | die neue Warnfarbe `Palette::warning` (Konflikte und Fehler) |
| `crates/phocommander-ui/src/lib.rs` | 73 | das Modul `rename` und die öffentlichen Namen |
| `crates/phocommander-ui/tests/rename_conflict_jump.rs` | 175 | der UI-Test aus AC-M1-24 (F3/Shift+F3 über echte Tastenereignisse) |
| `apps/phocommander-desktop/src/rename.rs` | 429 | die Composition Root: `OpsRenameEngine` (über `phocommander-ops`) und `FileTemplateStore` |
| `apps/phocommander-desktop/src/filesystem.rs` | 165 | der Adapter des Dateisystems des Hosts auf `phocommander_ops::port::FileSystem` |
| `apps/phocommander-desktop/src/main.rs` | 374 | die Verdrahtung von Engine und Vorlagenspeicher in die Anwendung |
| `apps/phocommander-desktop/src/lib.rs` | 32 | die neuen Module der Composition Root |
| `crates/phocommander-i18n/locales/{de,en,fr}/rename.ftl` | 27/27/28 | die 22 neuen Texte in drei Sprachen |
| `crates/phocommander-i18n/src/catalogue.rs` | 543 | die neue Domain `rename` in der Liste der Kataloge |
| `scripts/check-i18n-keys.py` | — | die neue Domain in der Liste des Gates |
| `docs/WHERE_TO_CHANGE.md` | — | zwei Zeilen: der Dialog und die Adapter der Composition Root |

**Keine Datei über 600 Zeilen außer `app.rs` (748)** — sie trägt die Begründung im Modulkopf
(`SIZE:`), wie die anderen vier über dem Zielwert im Repository.

---

## 5. Die Befehle, die ich wirklich ausgeführt habe

Alle Ausgaben sind **Originalausgaben**. Ich habe **kein** `cargo` aufgerufen (Bau-Gate).

### 5.1 `python3 scripts/check-i18n-keys.py` → exit 0

```
locales   : .../crates/phocommander-i18n/locales
            de: 137 keys
            en: 137 keys
            fr: 137 keys
reference : de (137 keys)

OK    de: the key set is identical to de
OK    en: the key set is identical to de
OK    fr: the key set is identical to de

RESULT: PASS — 3 locales hold the same 137 keys
```

### 5.2 `python3 scripts/check-i18n-hardcoded.py` → exit 0

```
Files read               : 44
Visible texts in the code : 0
RESULT: PASS — the interface layer holds no text of its own.
```

(Der erste Lauf dieses Gates war **rot** — 11 Fehlermeldungen in `rename/template.rs` waren
englische Sätze im Quelltext. Ich habe sie auf Bezeichner umgestellt (`name-empty`,
`key-unknown:<key>` …) und das Gate erneut laufen lassen.)

### 5.3 `./scripts/check-crate-boundaries.sh` → exit 0

```
13 path dependencies checked, 0 violations
```

### 5.4 `./scripts/check-file-size.sh` → exit 0 (mit Warnungen)

```
OVER  614 lines  ./crates/phocommander-ops/src/progress.rs
OVER  748 lines  ./crates/phocommander-ui/src/app.rs
OVER  613 lines  ./crates/phocommander-ui/src/keymap_view.rs
OVER  651 lines  ./crates/phocommander-ui/src/panel.rs

Source files measured : 178 (extensions: rs)
Target value          : 600 lines (warning)
Absolute maximum      : 900 lines (failure)
Target-value warnings : 6
Maximum violations    : 0
RESULT: PASS with warnings — 6 file(s) exceed the target value of 600 lines.
```

### 5.5 `python3 scripts/check-action-ids.py` → exit 0

```
OK    the keyboard bindings match the key table of the specification
OK    dpad and controller refer to the action palette
OK    no key is bound to two actions of the same surface
OK    every action ID is in dotted notation
OK    every action is referenced in the acceptance criteria or the specification
OK    the overrides in defaults.toml name known actions

RESULT: PASS — 9 of 9 checks passed
```

### 5.6 `python3 scripts/check-keymap-consistency.py` → exit 0

```
Undocumented deviations: 0
RESULT: PASS — specification, catalogue and reference agree.
```

### 5.7 `python3 scripts/check-requirement-ids.py` → exit 0

```
OK    every row carries the seven columns of the catalogue
OK    every source names the line of the requirement
OK    every acceptance criterion exists in the acceptance document

RESULT: PASS — 7 of 7 checks passed
```

### 5.8 `python3 scripts/check-docs-links.py` → exit 0

```
Markdown files scanned : 46
Repository links checked: 159
External links skipped : 20
Broken links           : 0
RESULT: PASS — every repository link resolves.
```

### 5.9 Die Kontraste (NFR-11) — mit Python gerechnet

```
light warning  #a83a00 on #ffffff : 6.42
dark  warning  #ffa85c on #242424 : 8.14
light focus    #0b57d0 on #ffffff : 6.39
dark  focus    #8ab4f8 on #242424 : 7.37
light text     #1a1a1a on #ffffff : 17.4
dark  text     #e8e8e8 on #242424 : 12.67
```

Die Warnfarbe (neu, für die Konflikte und die Fehler) erreicht **6,42:1** (hell) und **8,14:1**
(dunkel) — beide über den geforderten 4,5:1 für Text. Der Fokusrahmen erreicht 6,39:1 und
7,37:1 (gefordert: 3:1). Der Test
`the_warning_reaches_the_contrast_the_specification_names` in `theme.rs` prüft das im
Testlauf — **der Test ist nicht gelaufen**, siehe Abschnitt 7.

### 5.10 Die Übereinstimmung der Übersetzungsschlüssel (eigene Prüfung)
Ein Skript hat die 129 `TextKey`-Werte der UI-Crate mit den Katalogen verglichen:

```
UI keys: 129
--- de: catalogue 137 keys
   in the UI but not in the catalogue: []
   in the catalogue but not in the UI: ['error-binding-empty', ...  'panel-undo-notice']
--- en: catalogue 137 keys
   in the UI but not in the catalogue: []
--- fr: catalogue 137 keys
   in the UI but not in the catalogue: []
```

Die acht Schlüssel, die nur im Katalog stehen, sind die Fehlermeldungen, die die UI-Crate
**als Zeichenkette** benutzt (z. B. `error-transfer-same-panel`) — so war es schon vorher.

### 5.11 Die zweite, Compiler-nahe Prüfung

Weil ich nicht bauen durfte, habe ich den Code zusätzlich **von Hand auf Leihfehler (E0503)
geprüft** — und dabei einen echten Fehler gefunden: in `rename/fields.rs` rief
`self.buffer_mut(focus)` den Puffer des Feldes mutabel und las danach `self.caret`. Der Cursor
ist ein Teil desselben `Fields`, also war die zweite Leihe ein Fehler. Die drei Methoden
(`insert`, `backspace`, `delete`) lesen den Cursor jetzt **vor** der mutablen Leihe und
schreiben ihn **nach** ihr Ende — dasselbe Muster wie in `dialog.rs::write_to_field`.

Eine zweite Rolle hat denselben Fehler unabhängig gefunden und mir die Zeilen gemeldet; die
Korrektur ist bestätigt. **Weitere Compilerfehler kann ich nicht ausschließen** — das ist der
Grund, warum der Build beim Owner bleiben muss.

---

## 6. Die Tests, die ich geschrieben habe

| Test | Datei | Was er beweist |
|---|---|---|
| `the_preview_follows_every_change_of_the_rule` | `rename/tests.rs` | die Vorschau folgt jeder Änderung der Regel (FR-34) |
| `the_engine_is_not_asked_again_when_nothing_changed` | `rename/tests.rs` | die Engine wird **einmal pro Änderung** gefragt, nicht pro Frame |
| `a_conflict_is_marked_and_the_jump_list_names_it` | `rename/tests.rs` | ein Konflikt ist markiert und steht in der Sprungliste (FR-35) |
| `the_next_conflict_is_reachable_and_the_end_of_the_list_stays` | `rename/tests.rs` | F3 springt weiter; am Ende bleibt der Cursor (AC-M1-24) |
| `the_previous_conflict_is_reachable` | `rename/tests.rs` | Shift+F3 springt zurück |
| `the_rename_is_refused_while_a_conflict_is_on_the_list` | `rename/tests.rs` | ein Konflikt verweigert die ganze Umbenennung (FR-41) |
| `a_file_that_keeps_its_name_is_not_renamed` | `rename/tests.rs` | eine Datei, die ihren Namen behält, wird nicht umbenannt |
| `a_stored_template_can_be_applied_again` | `rename/tests.rs` | eine gespeicherte Vorlage lässt sich wieder anwenden (FR-36) |
| `a_template_with_the_same_name_replaces_the_old_one` | `rename/tests.rs` | ein Name steht für eine Regel |
| `the_focus_walks_the_dialog_in_a_fixed_order` | `rename/tests.rs` | die Fokusreihenfolge ist fest (NFR-11, Abschnitt 6.4) |
| `the_name_of_a_template_does_not_change_the_rule` | `rename/tests.rs` | der Name einer Vorlage ändert die Vorschau nicht |
| `a_template_survives_the_round_trip_through_the_text` | `rename/template.rs` | die Vorlage übersteht den Weg in die Datei und zurück (FR-36) |
| `a_line_with_an_unknown_key_is_an_error` | `rename/template.rs` | ein Tippfehler in der Datei wird gemeldet, nicht ignoriert |
| `the_preview_is_what_the_engine_forms` | `apps/.../rename.rs` | **die Vorschau der Composition Root ist genau die, die `phocommander_ops::template::form_names` bildet** |
| `a_target_that_exists_is_a_conflict` | `apps/.../rename.rs` | ein belegter Zielname wird als Konflikt gemeldet (FR-35) |
| `two_files_that_want_the_same_name_are_a_conflict` | `apps/.../rename.rs` | zwei Dateien mit demselben Wunschnamen sind ein Konflikt |
| `a_rule_without_a_format_is_a_problem_of_the_whole_rule` | `apps/.../rename.rs` | eine Regel ohne Format wird als Grund der ganzen Regel gemeldet |
| `a_stored_template_is_read_back` | `apps/.../rename.rs` | die gespeicherte Vorlage kommt wieder aus der Datei |
| `the_conflict_jump_list_is_reachable_with_f3_and_shift_f3` | `tests/rename_conflict_jump.rs` | der **UI-Test** aus AC-M1-24: F2 öffnet, F3 springt, F3 am Ende bleibt, Shift+F3 springt zurück |

**Die beiden Tests, die die Aufgabenstellung verlangt:**

* *„dass die Vorschau dem entspricht, was die Engine erzeugt“* →
  `the_preview_is_what_the_engine_forms` in `apps/phocommander-desktop/src/rename.rs`: der
  Adapter bildet dieselbe Frage, die der Dialog stellt, und der Test vergleicht die Namen der
  Vorschau mit den Namen von `form_names` desselben Regelsatzes.
* *„dass ein Konflikt in der Liste markiert und über F3 erreichbar ist“* →
  `the_conflict_jump_list_is_reachable_with_f3_and_shift_f3` (egui_kittest, echte
  Tastenereignisse) **und** die Unit-Tests der Sprungliste.
* *„dass eine gespeicherte Vorlage wieder angewendet werden kann“* →
  `a_stored_template_can_be_applied_again` (Dialog) **und** `a_stored_template_is_read_back`
  (Datei) **und** `a_template_survives_the_round_trip_through_the_text` (Format).

---

## 7. Was ich **nicht** getan habe — und warum

| Nicht getan | Grund |
|---|---|
| `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy` | **Bau-Gate des Auftrags**: ein zweiter Build neben dem des Owners bricht beide ab (3 GB RAM). Der Code ist deshalb **nicht übersetzt** — er kann Fehler enthalten, die nur der Compiler findet. **Hinweis für den Build:** jeder `cargo`-Aufruf braucht `< /dev/null` (das stdin enthielt sonst die Ausgabe des vorherigen Kommandos, und `rustc -` las sie als Quelltext). |
| Die Zeichendauer bei 10.000 und 100.000 Einträgen messen | Ohne Build gibt es keine Messung. Ich schreibe keine Zahl, die ich nicht gemessen habe. **Was ich stattdessen getan habe:** die Vorschau zeichnet nur 12 Zeilen (`VISIBLE_ROWS`), und die Engine wird nur bei einer Änderung der Regel gefragt (`dirty` + `engine_calls()`). **Offener Punkt:** die *Bildrate* des Dialogs ist damit nicht gemessen (NFR-01/NFR-02). |
| Der UI-Test aus AC-M1-24 ausführen (`cargo test -p phocommander-ui --test rename_conflict_jump -- --exact`) | Ohne Build nicht möglich. Der Test **ist geschrieben** und wartet auf den Lauf des Owners. |
| Die Umbenennung selbst ausführen | `Command::Rename` trägt jetzt die Paare (Quelle → Ziel). Die Desktop-App führt die Befehle noch nicht aus — das ist die Arbeit der Kern-Rolle (die App ist bisher eine Hülle). Die Brücke steht: `apps/phocommander-desktop/src/rename.rs` übersetzt die Paare in `phocommander_ops::rename::execute_renames`. |
| FR-40 (die verbotenen Zeichen der Zielplattform) in der Oberfläche | Die Engine meldet sie (`phocommander_ops::names`), aber die Naht der Vorschau kennt heute nur die vier Gründe der Engine. Die Anzeige der verbotenen Zeichen gehört in die Regel der UI (`docs/REQUIREMENTS.md` FR-40) und ist nicht Teil von FR-34/35/36. |
| `config/templates/*.toml` (die mitgelieferten Vorlagen) in den Dialog laden | Die mitgelieferten Vorlagen haben ein **anderes Format** (geschweifte Klammern, Erkennungsregeln) und gehören zur Konfiguration (`config/templates/`, Schema des Config-Crates). Der Dialog benutzt sein eigenes, strenges Format für die Vorlagen des Benutzers. Die Zusammenführung der beiden Formate ist eine Owner-Entscheidung (Abschnitt 8). |
| `docs/REQUIREMENTS.md`, `docs/ACCEPTANCE-CRITERIA.md`, `docs/STATUS.md` aktualisieren | Diese Dokumente gehören der Produkt- und der QA-Rolle. Ich habe `docs/WHERE_TO_CHANGE.md` ergänzt (das verlangt die Datei selbst für jede neue Datei). |

---

## 8. Offene Punkte für eine Owner-Entscheidung

1. **Die Kosten der Konfliktplanung bei sehr großen Auswahlen.** `plan_renames` des
   Ops-Crates ist O(n²) (er sucht für jedes Paar das Paar, das den Zielnamen freigibt) und
   ruft das Dateisystem 2n-mal auf (`metadata` für Quelle und Ziel). Bei 10.000 Dateien sind
   das 20.000 Dateisystemaufrufe **pro Tastendruck**; bei 100.000 Dateien ist die
   Vorschau nicht mehr bedienbar. **Mein Vorschlag:** die Prüfung auf doppelte Ziele in
   `phocommander-ops` mit einer `HashMap` linear machen (O(n)) und dem Adapter einen
   zwischengespeicherten Verzeichniseintrag geben, damit die `metadata`-Aufrufe nicht bei
   jedem Tastendruck wiederholt werden. Ich habe das Ops-Crate **nicht** angefasst, weil es
   der Core-Rolle gehört und der Auftrag hier die Oberfläche war.
2. **Zwei Formate für Vorlagen.** Die mitgelieferten Vorlagen (`config/templates/*.toml`,
   geschweifte Klammern, Erkennungsregeln, Seriennummern) und die Vorlagen des Benutzers
   (mein Textformat) sind zwei Sprachen. Sollen die mitgelieferten Vorlagen im Dialog
   wählbar sein? Dann braucht es eine Übersetzung zwischen den Formaten — am besten im
   Config-Crate, das die Schema-Verantwortung hat (AC-M1-25).
3. **F2 bei genau einer Auswahl.** `docs/KEYMAP.md` sagt: „With more than one entry selected,
   the multiple-rename dialog opens instead of the single rename field.“ Mein Dialog öffnet
   sich **immer** (mit einer Zeile, wenn nur eine Datei markiert ist), weil das
   Einzelsummen-Feld im Code noch nicht existiert. Ist das so gewollt, oder soll F2 bei
   genau einer Datei zunächst gar nichts tun?
4. **Eine Aktion für das Speichern einer Vorlage.** Das Speichern hängt heute an der
   Enter-Taste im Namensfeld (kontextabhängig). Ein eigenes `rename.template_save` im
   Katalog (z. B. Ctrl+S) wäre eindeutiger, verlangt aber eine Zeile in
   `config/keymap.toml`, eine in `docs/KEYMAP.md` und eine Aktion in `actions.rs`.
5. **`Command::Rename` trägt jetzt Paare statt Pfade.** Das ist eine Änderung der
   öffentlichen Schnittstelle der UI-Crate (die Desktop-App benutzt den Befehl noch nicht).
   Die Form ist die bessere, aber sie muss mit der Core-Rolle abgestimmt werden, wenn der
   Befehl dort ausgeführt wird.
6. **Die Reihenfolge der Prioritäten.** Die Vorschau ist der Kern; die Sprungliste und die
   Vorlagen sind fertig. Was zuerst an den Owner: der Bau, der Testlauf oder die Abstimmung
   der offenen Punkte?

---

## 9. Der Patch

`deliverable.ui-engineer-rename.patch` — 28 Dateien, +4903 / −17 (ohne die Dateien, die andere
Rollen parallel angefasst haben: `scripts/check-gates-negative.sh` gehört einer anderen Rolle
und ist **nicht** im Patch).
