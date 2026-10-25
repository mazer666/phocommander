# M2b — Internationalisierung: lokalisierte Formatierung und Sprachwechsel zur Laufzeit

**Rolle:** UI-Engineer · **Stand:** 25.10.2026 · **Meilenstein:** M2b (FR-123, FR-124)
**Patch:** `deliverable.ui-engineer.patch` (19 Dateien, +1534 / −140) · **Kein Commit, kein Push.**

---

## 1. Was ich gebaut habe

Die Oberfläche schrieb die Größen und die Zeiten selbst fest (`listing.rs`: `format_size`
fest „KiB/MiB", binär; `format_time` fest „YYYY-MM-DD HH:MM"). Beides ist englisch und von
der Sprache des Systems unabhängig. Der Katalog der Texte war ein `Box<dyn Catalogue>`, das
beim Bauen der App übergeben wurde — ein Wechsel zur Laufzeit war nicht möglich.

### 1.1 Die Naht: `Formats` (FR-123)

Genau so, wie es für die Texte bereits ist: das Trait liegt in der UI-Crate, die
Implementierung sitzt in der Composition Root.

```rust
// crates/phocommander-ui/src/format.rs
pub trait Formats {
    fn number(&self, value: u64) -> String;
    fn size(&self, bytes: u64) -> String;
    fn date(&self, moment: SystemTime) -> String;
    fn time(&self, moment: SystemTime) -> String;
    fn date_time(&self, moment: SystemTime) -> String { /* Standard: Datum + Uhrzeit */ }
}
```

Die Signaturen nehmen `SystemTime` und `u64` — nicht die Uhrzeit als Text. Die Formatierung
ist Sache der Implementierung.

* **Rückfall ohne Trait:** `PlainFormats` — die alten englischen Formate (binäre Einheiten,
  `YYYY-MM-DD`, `HH:MM`). `WorkSurfaceApp::new(...)` nimmt weiterhin nur den Katalog; die
  Formate sind ein Baustein (`with_formats`). **Die über 120 Tests der UI mussten therefore
  nicht umgebaut werden** — die vorhandenen Signaturen `WorkSurfaceApp::new(surface, keymap,
  catalogue)` bleiben unverändert.
* **Die Oberfläche holt sich die Texte über das Trait:** `Row::size_text(&dyn Formats)` und
  `Row::date_text(&dyn Formats)`; `format_size`, `format_time` und `civil_from_days` sind aus
  `listing.rs` **entfallen** (die Zivilrechnung ist in `format.rs` umgezogen).
* **Die Crate-Grenze bleibt:** `phocommander-ui` kennt `phocommander-i18n` nicht. Das Gate
  `scripts/check-crate-boundaries.sh` meldet 0 Verstöße (Ausgabe in Abschnitt 5).

### 1.2 Die Implementierung: `LocaleFormats` (Composition Root)

```rust
// apps/phocommander-desktop/src/catalogue.rs  (neben dem TranslatorCatalogue)
pub struct LocaleFormats { format: LocaleFormat }
impl Formats for LocaleFormats { /* number, size, date, time, date_time */ }
```

`LocaleFormats` benutzt den `LocaleFormat` des i18n-Crates (CLDR-Daten über `icu`).

**Wichtiger Nebenbefund:** `crates/phocommander-i18n/src/format.rs` existierte, war aber im
Crate **nie als Modul deklariert** — `LocaleFormat` war nicht benutzbar, und der Import von
`DecimalFormatterOptions` war falsch (`icu::decimal::DecimalFormatterOptions` existiert nicht,
sondern `icu::decimal::options::DecimalFormatterOptions`). Ich habe das Modul aktiviert, den
Import korrigiert und die zwei fehlerhaften Tests der Datei berichtigt (der Moment trug
08:30 statt 14:30 UTC; die englische Datumsform „May 17, 2024" enthält keine Ziffer „5").
**Ohne diese Korrektur wäre der erste Build des i18n-Crates rot gewesen.**

### 1.3 Der Sprachwechsel zur Laufzeit (FR-124)

| Schritt | Datei | Wirkung |
|---|---|---|
| Katalog austauschen | `WorkSurfaceApp::set_catalogue(Box<dyn Catalogue>)` | der nächste Frame zeichnet die neue Sprache |
| Formate austauschen | `WorkSurfaceApp::set_formats(Box<dyn Formats>)` | die Größen und die Daten folgen dem neuen Locale |
| Neu zeichnen | `set_*` → `egui::Context::request_repaint()` | **kein Neustart, kein Neuzeichnen-Pflicht** |
| Sprache bauen | `phocommander_desktop::language::InterfaceLanguage` | Katalog **und** Formate einer Locale gehören zusammen |
| Sprache anwenden | `phocommander_desktop::desktop::DesktopApp::set_language` | baut die Sprache neu und hängt sie an die Schale |

Der Katalog und die Formate werden **zusammen** gewechselt, weil eine Sprache nicht nur die
Texte ist, sondern auch die Schreibweise der Zahlen und der Daten (FR-123).

**Der Auslöser.** Die Oberfläche hat noch keine Einstellungsfläche, und die native Menüleiste
(`muda`) ist noch nicht gebaut. Der Weg, den der Benutzer heute hat, ist die
Konfigurationsdatei: `language = "de"` in `config.toml`. Der Desktop-Build liest die Datei
alle **eine Sekunde** nach und wendet eine geänderte Sprache sofort an
(`ConfigWatcher` → `DesktopApp::apply_the_language_of_the_configuration`). Nur die Sprache wird
zur Laufzeit angewendet; die übrigen Einstellungen werden beim Start gelesen, weil eine Änderung
der Sicherheitsregeln einen Neustart braucht.

**Die Statuszeile folgt der Sprache.** Vorher stand dort der bereits aufgelöste Text. Jetzt
hält die Schale den **Schlüssel** und die Argumente (`StatusMessage`) und liest den Text im
zeichnenden Frame aus dem Katalog. Ein Sprachwechsel trifft damit auch die Meldung, die
gerade offen war.

### 1.4 Das Zustandsmodell der Sprache

| Struktur | Felder | Übergänge | Fehlerfälle |
|---|---|---|---|
| `WorkSurfaceApp` | `catalogue: Box<dyn Catalogue>`, `formats: Box<dyn Formats>`, `egui_context: Option<egui::Context>` | `set_catalogue` / `set_formats` → Feld tauschen + `request_repaint`; `update()` merkt sich den egui-Kontext des ersten Frames | Katalog nicht lesbar → `EmptyCatalogue` (die Oberfläche zeigt den Schlüssel, FR-122); Locale ohne CLDR-Daten → `PlainFormats`; `set_*` vor dem ersten Frame → kein Repaint nötig, das nächste Frame zeigt die neue Sprache |
| `InterfaceLanguage` | `locale: Locale`, `catalogue: Option<Box<dyn Catalogue>>`, `formats: Option<Box<dyn Formats>>` | `of(locale)` baut beide; `take_catalogue()` / `take_formats()` geben sie an die Schale | zweimal genommen → `EmptyCatalogue` / `PlainFormats` (nie leer, nie ein Absturz) |
| `ConfigWatcher` | `path: Option<PathBuf>`, `stamp: Option<FileStamp>`, `next_check: Instant` | `changed()` → höchstens einmal pro Sekunde, Vergleich von Änderungszeit **und** Länge | Datei fehlt → `stamp = None`, es wird nichts gemeldet; kein Konfigurationsverzeichnis → `without_file()` |
| `StatusMessage` | `key: String`, `arguments: Vec<String>` | wird beim Zeichnen in den Text übersetzt | Schlüssel unbekannt → der Schlüssel selbst, nie leer (FR-122) |

### 1.5 Die Tastenbelegung

**In diesem Teil wurde keine Taste geändert.** Die Belegung (`docs/KEYMAP.md`,
`config/keymap.toml`) bleibt unberührt — die Formatierung und der Sprachwechsel sind keine
Tastenaktionen.

| Vorgabe | Aktion | Kontext | plattformabhängig? | Implementiert? | Testname |
|---|---|---|---|---|---|
| — (keine neue Bindung) | Sprache zur Laufzeit wechseln | alle | nein | **Mechanismus ja, Bindung nein** | `a_change_of_the_language_reaches_the_screen_in_the_next_frame` |
| F2/F3/F5… (F1–F10) | unverändert | Tastatur | nein | ja (unberührt) | — |

Die Bindung für den Sprachwechsel ist **offen** (Abschnitt 7): die Aktionspalette und die
native Menüleiste, in die sie gehörte, sind noch nicht gebaut.

---

## 2. Die Dateien

| Pfad | Zeilen | Zweck |
|---|---:|---|
| `crates/phocommander-ui/src/format.rs` | 267 | **neu** — das Trait `Formats` und der Rückfall `PlainFormats` (die Naht zwischen Oberfläche und Locale) |
| `crates/phocommander-ui/src/app/language.rs` | 230 | **neu** — der Sprachwechsel der Schale: `set_catalogue`, `set_formats`, `with_formats`, `catalogue()`, `formats()`, `request_repaint()` |
| `crates/phocommander-ui/src/app/language/tests.rs` | 158 | **neu** — 3 Unit-Tests und 1 Test mit dem egui-Harness |
| `crates/phocommander-ui/src/app.rs` | 720 | die Schale: Felder `formats` und `egui_context`, `StatusMessage` statt `Option<String>`, der Repaint, das `DrawStyle` bekommt die Formate |
| `crates/phocommander-ui/src/listing.rs` | 517 | `format_size`/`format_time`/`civil_from_days` **entfallen**; `Row::size_text(&dyn Formats)`, `Row::date_text(&dyn Formats)` |
| `crates/phocommander-ui/src/listing/tests.rs` | 207 | 2 neue Tests (die Größe der Zeile folgt den Formaten), 2 umgestellt |
| `crates/phocommander-ui/src/draw.rs` | 286 | `DrawStyle` trägt die Formate der Locale |
| `crates/phocommander-ui/src/draw/list.rs` | 312 | die Spalten „Größe" und „Datum" fragen die Formate |
| `crates/phocommander-ui/src/lib.rs` | 67 | `pub mod format;` und `pub use format::{Formats, PlainFormats};` |
| `crates/phocommander-i18n/src/lib.rs` | 172 | `pub mod format;` + `pub use format::{FormatError, LocaleFormat};` — der previously unbenutzbare `LocaleFormat` |
| `crates/phocommander-i18n/src/format.rs` | 425 | der Import von `DecimalFormatterOptions` korrigiert, zwei Tests berichtigt (die Datei ist jetzt **zum ersten Mal** kompiliert) |
| `apps/phocommander-desktop/src/catalogue.rs` | 216 | **neu** — `LocaleFormats` (die Implementierung des Traits über `LocaleFormat`), `EmptyCatalogue`, `formats_for`, `catalogue_for` |
| `apps/phocommander-desktop/src/language.rs` | 313 | **neu** — `InterfaceLanguage`, `locale_of`, `ConfigWatcher` |
| `apps/phocommander-desktop/src/desktop.rs` | 131 | **neu** — `DesktopApp`: die Anwendung, die die Schale und den Wächter hält und die Sprache anwendet |
| `apps/phocommander-desktop/src/main.rs` | 347 | `build_desktop_app` (die Anwendung mit dem Wächter), `configuration_path`; `build_translator` **entfallen** (darin stand ein `unreachable!()` — ein Panic im Produktionscode) |
| `apps/phocommander-desktop/src/lib.rs` | 29 | die neuen Module `desktop` und `language` |
| `apps/phocommander-desktop/src/tests.rs` | 123 | 1 neuer Test, 1 umgestellt (`build_translator` gibt es nicht mehr) |
| `docs/WHERE_TO_CHANGE.md` | — | 2 Zeilen: wo die Formate und wo die Adapter liegen |
| `docs/ARCHITECTURE.md` | — | 1 Zeile: die Formate in der Liste „Wo was geändert wird" |

**Keine neue Crate, keine neue Abhängigkeit.** Das i18n-Crate bekam keine neue Crate; die
UI-Crate bekam keine neue Crate (das `icu` liegt im i18n-Crate, `egui_kittest` war schon
dev-dependency). `Cargo.toml` und `Cargo.lock` sind **unverändert**.

---

## 3. Die Tests

**20 neue Tests**, 7 vorhandene Tests des i18n-Crates laufen zum ersten Mal.

| Crate | Datei | Tests | Was sie beweisen |
|---|---:|---:|---|
| `phocommander-ui` | `src/format.rs` | 3 | die Rückfallformate: „2.0 KiB", „2024-05-17", „14:30" |
| `phocommander-ui` | `src/listing/tests.rs` | 2 (+2 umgestellt) | die Größe der Zeile folgt dem Trait: „1.1 MiB" (binär) gegen „1,2 MB" (dezimal, Komma) |
| `phocommander-ui` | `src/app/language/tests.rs` | 4 | der Katalog und die Formate sind austauschbar; **ein Test mit `egui_kittest`**: nach dem Wechsel trägt der Accessibility-Baum die Texte des neuen Katalogs und keine des alten |
| `phocommander-i18n` | `src/format.rs` | 7 (vorhanden, jetzt aktiv) | „1.234" gegen „1,234", „1,2 MB" gegen „1.2 MB", die Reihenfolge des Datums, die 24-Stunden- gegen die 12-Stunden-Uhr |
| `phocommander-desktop` | `src/catalogue.rs` | 5 | die Formate der Locale: „1,2 MB" (de) gegen „1.2 MB" (en); eine unbekannte Locale fällt auf die Rückfallformate zurück |
| `phocommander-desktop` | `src/language.rs` | 5 | die Zuordnung der Einstellung zur Locale; der Wächter meldet eine Änderung der Datei und meldet eine ruhige Datei nicht |
| `phocommander-desktop` | `src/tests.rs` | 1 (neu) | die Anwendung des Desktop-Builds wird aus der Konfiguration gebaut |

---

## 4. Echte Messungen

**Ich habe nichts gebaut und nichts gemessen, was einen Build braucht** (die Anweisung: kein
`cargo build`, kein `cargo test`, kein `cargo fmt` — ein zweiter Build neben dem des Owners
bricht beide ab). Die Gates, die ohne Build laufen, sind gelaufen; ihre Ausgabe steht unten.

```
$ python3 scripts/check-i18n-hardcoded.py
Files read               : 35
Visible texts in the code : 0
RESULT: PASS — the interface layer holds no text of its own.

$ bash scripts/check-crate-boundaries.sh
13 path dependencies checked, 0 violations

$ bash scripts/check-file-size.sh
Source files measured : 148 (extensions: rs)
Target value          : 600 lines (warning)
Absolute maximum      : 900 lines (failure)
Target-value warnings : 6
Maximum violations    : 0
RESULT: PASS with warnings — 6 file(s) exceed the target value of 600 lines.

$ python3 scripts/check-i18n-keys.py
RESULT: PASS — 3 locales hold the same 115 keys

$ python3 scripts/check-links.py
Broken links           : 0
RESULT: PASS — every repository link resolves.

$ git apply --check --reverse deliverable.ui-engineer.patch
(Patch OK: der Patch passt auf den Arbeitsbaum.)
```

**Die sechs Dateien über 600 Zeilen sind dieselben wie vor meiner Änderung** — ich habe keine
neue hinzugefügt. `crates/phocommander-ui/src/app.rs` war schon über 600 (665 Zeilen) und ist
jetzt 720; die Begründung im Modulkopf (`SIZE:`) ist entsprechend ergänzt.

**Syntaxprüfung ohne Build.** Weil ich nicht bauen durfte, habe ich jede geänderte Datei mit
`rustc --edition 2024 --crate-type lib --emit=metadata -o /dev/null <datei>` geprüft: das
Kompilieren bricht nach dem Parsen ab, weil die Fremd-Crates nicht auf dem Pfad liegen. **Die
Ausgabe enthielt in keiner Datei einen Syntaxfehler** — nur „unresolved import"/„can't find
crate" für die Crates, die es nicht auf dem Pfad gibt.

**Nicht gemessen:** die Dauer des Zeichnens (NFR-01), der Kontrast (NFR-11), die Bildrate bei
10.000/100.000 Einträgen. Diese Werte habe ich nicht verändert und nicht neu gemessen; die
Messungen des Projekts stehen in `docs/acceptance/logs/`.

---

## 5. Der Nachweis aus dem echten UI-Test

**NOT RUN.** Der Test `a_change_of_the_language_reaches_the_screen_in_the_next_frame` in
`crates/phocommander-ui/src/app/language/tests.rs` ist geschrieben, aber **nicht gelaufen**:
ein Testlauf braucht einen Build, und der Build war mir untersagt.

Was der Test tut, in der Reihenfolge:

1. Er baut die Schale mit einem Katalog, der jeden Schlüssel mit `en-` markiert, und zeichnet
   einen Frame mit dem Harness von egui (`Harness::new_ui_state` + `run_steps(1)`).
2. Er prüft, dass der Accessibility-Baum des Frames Beschriftungen mit `en-` trägt — die
   Knöpfe der Werkzeugleiste tragen die Texte des Katalogs.
3. Er tauscht den Katalog gegen einen, der `de-` markiert (`set_catalogue`), und zeichnet
   einen weiteren Frame.
4. Er prüft, dass der Baum nun `de-` trägt **und** kein `en-` mehr trägt.

Der Test liest also genau das, was der Benutzer sieht: die Beschriftung der Knöpfe auf dem
Bildschirm, nicht einen Zustand im Speicher.

**Was der Owner tun muss, damit dieser Nachweis existiert:**
`cargo test -p phocommander-ui app::language` — und für den Rest der Abnahme
`cargo test --workspace --all-features`.

---

## 6. Was NOT RUN bleibt — und warum genau

| Nachweis | Status | Grund |
|---|---|---|
| `cargo test -p phocommander-ui` (20 neue Tests) | **NOT RUN** | die Anweisung untersagt `cargo test`; ein zweiter Build neben dem des Owners bricht beide ab (3 GB RAM im Sandbox, vier Agenten hängen heute daran) |
| `cargo test -p phocommander-i18n locale_format` (AC-M2b-05) | **NOT RUN** | derselbe Grund — der Test ist geschrieben, die Datei war vorher nie kompiliert |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **NOT RUN** | derselbe Grund |
| `cargo fmt --all -- --check` | **NOT RUN** | die Anweisung untersagt `cargo fmt`; der Code ist von Hand formatiert, aber **rustfmt kann abweichen** (Zeilenumbruch, Importreihenfolge) — bitte einmal laufen lassen |
| AC-M2b-06 (manuell): Sprache in den Einstellungen umschalten, einen offenen Dialog lesen | **NOT RUN** | die Sandbox hat keinen Bildschirm; **und**: es gibt noch keine Einstellungsfläche. Der von mir gebaute Weg ist die Konfigurationsdatei, nicht die Einstellungen |
| AC-M2b-08 (manuell): Oberflächensprache unabhängig von der Sprache der Dateinamen | **NOT RUN** | manuell; vom Code her ist es erfüllt (die Dateinamen laufen nie durch den Katalog) — der Nachweis ist die manuelle Beobachtung |
| `cargo build` des i18n-Crates mit dem aktivierten `format`-Modul | **NOT RUN** | derselbe Grund. **Ich habe die API von `icu` 2.4 gegen den Quelltext in `~/.cargo/registry` geprüft** (Felder, `try_new`, `format`, `Date::try_new_iso`, `Time::try_new`, `Decimal::from`, `multiply_pow10`, `DecimalFormatterOptions`) — die Prüfung ist eine Quelle, kein Build |

**Risiko, das der Owner kennen muss:** die beiden Tests der Desktop-Crate
`the_formats_of_a_german_locale_write_the_sizes_the_way_german_writes_them` und
`…english…` erwarten „1,2 MB" bzw. „1.2 MB". Das setzt voraus, dass die CLDR-Daten von
`icu` 2.4 die Locales „de" und „en" **ohne** Regionsangabe kennen. Die Dokumentation von
`icu` zeigt die Konstruktoren mit `locale!("en")` und `locale!("bn")` — also mit der
Sprachangabe allein ✓. Sollte die Datenlage abweichen, fällt `formats_for` auf `PlainFormats`
zurück und der Test meldet es; dann ist die Zuordnung „Locale → CLDR-Tag" zu entscheiden
(Abschnitt 7).

---

## 7. Offene Punkte für eine Entscheidung des Owners

1. **Womit wird die Sprache umgeschaltet?** Ich habe die Konfigurationsdatei gewählt, weil
   es heute keine Einstellungsfläche gibt. Die Alternative — ein Menüeintrag in der nativen
   Menüleiste (`muda`, noch nicht gebaut) — wäre die dauerhafte Lösung. Die Datei ist der
   Notbehelf, und sie kostet: die Anwendung wacht **einmal pro Sekunde** auf, um die Datei zu
   prüfen. Ein Thread, der die Datei überwacht und nur bei einer Änderung einen Repaint
   anfordert, wäre billiger — er ist aber ein Thread und damit eine Entscheidung, die ich
   nicht ohne Auftrag getroffen habe.
2. **Wie kommen die Formate zu einem Locale?** Die Implementierung benutzt den Katalognamen
   als CLDR-Tag („de", „en", „fr"). Eine Land-/Region-Variante („de-AT") hat keine eigenen
   Formate, solange der Katalog „de" heißt. Ob die Formate der **Sprache der Oberfläche**
   folgen oder dem **System** folgen, habe ich als „der Sprache der Oberfläche" gelesen:
   `language = "system"` nimmt die Locales des Systems, und der Katalog „de" nimmt die
   deutschen Formate. FR-123 sagt „folgen dem System der Sprache", AC-M2b-05 sagt „the
   separators follow the locale and not the build machine" — das passt zu meiner Lesart, aber
   es ist eine Auslegung.
3. **Die Zahlen in den Meldungen.** Die Zahlen der Statuszeile („7 Einträge") kommen aus dem
   Fluent-Katalog, und Fluent schreibt sie **ohne** Tausendertrennzeichen und ohne das
   Dezimalkomma der Locale. Die Trait-Methode `number` ist deshalb im UI-Crate vorhanden,
   wird aber von der Oberfläche noch nicht benutzt. FR-123 („numbers follow the system
   locale") ist damit **noch nicht vollständig erfüllt** — das ist eine Lücke, die in das
   Arbeitspaket „Katalog" gehört, nicht in die Formatierung der Dateiliste.
4. **Die Argumente der Meldungen haben keine Namen.** `Argument` kennt nur `Number`, `Total`
   und `Text`; der Katalog erwartet aber `$side`, `$key`, `$binding`, `$identifier`. Der
   Adapter `TranslatorCatalogue` gibt die Argumente heute gar nicht weiter, und die
   Meldungen mit Platzhaltern fallen deshalb auf den Schlüssel zurück („error-transfer-
   same-panel" statt des deutschen Textes). Das ist ein **vorbestehender Defekt** des
   Katalogadapters, unabhängig von meiner Änderung; ihn zu beheben hieße, `Argument` um
   benannte Argumente zu erweitern — das berührt die Fehlermeldungen aus dem
   Arbeitspaket „Schlüssel und Argumente" und gehört dorthin.
5. **Der Test des i18n-Crates war nie gelaufen.** `crates/phocommander-i18n/src/format.rs`
   enthielt zwei falsche Erwartungen (der Moment, die Datumsform). Ich habe sie berichtigt,
   aber **ohne Build** ist meine Erwartung an die CLDR-Ausgabe („May 17, 2024",
   „17.05.2024", „2:30 PM", „14:30") nicht gemessen, sondern aus den Regeln von CLDR
   abgeleitet. Der erste Testlauf sollte hier zuerst hinschauen.

---

## 8. Was ich bewusst nicht getan habe

* **Kein `cargo build`, `cargo test`, `cargo fmt`** — die Anweisung. Ich habe nur geschrieben
  und gelesen; die Syntaxprüfung mit `rustc --emit=metadata` hat keine Fremd-Crates gebaut.
* **Keine neue Crate, keine neue Abhängigkeit**, keine Änderung an `Cargo.toml` oder
  `Cargo.lock`.
* **Kein Commit, kein Push, kein Tag.**
* **Keine Änderung an den Katalogen** unter `crates/phocommander-i18n/locales/` — die drei
  Kataloge tragen weiterhin dieselben 115 Schlüssel.
* **Keine Tastenbelegung geändert** und kein `docs/KEYMAP.md` angefasst: es gibt keine neue
  Aktion.
* **`docs/STATUS.md` und `docs/ACCEPTANCE-CRITERIA.md` nicht angefasst** — sie gehören dem
  Produktmanagement und der Abnahme, und `docs/STATUS.md` trug beim Schreiben bereits die
  Änderung eines anderen Agenten. Die Zeilen für M2b in diesen Dateien sind nach dem
  Testlauf zu ergänzen (die offenen Punkte aus Abschnitt 6 und 7 gehören in die
  Abnahmekriterien).
* **Die Endung bestimmt das Format eines Containers nicht** und **die Fehlermeldungen tragen
   einen Katalogschlüssel** — beide Entscheidungen sind unberührt; ich habe sie als gegeben
   behandelt.
