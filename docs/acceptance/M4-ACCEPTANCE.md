# Abnahme des Meilensteins M4 — die Naht der Eingabe

**Stand:** 26.10.2026 · **Prüfer:** Leitende Instanz · **Ergebnis:** **in Arbeit** — der
Eingangskanal ist abgenommen, das Layout des Fingers und der Android-Bau sind offen

---

## 1. Was dieser Meilenstein umfasst

M4 macht die Anwendung **bedienbar, ohne Tastatur**. Die Spezifikation nennt in FR-101 fünf
Kontexte — die Tastatur, den Zeiger, den Finger, die Fernbedienung und das Gamepad — und sie
verlangt, dass alle fünf **dieselben Aktionen** erreichen. Der Meilenstein baut die Naht, an der
diese fünf Eingänge zusammentreffen:

| Teil | Inhalt | Stand |
|---|---|---|
| Die Ereignisse | `PointerEvent`, `Gesture`, `Direction`, `PinchDirection`, `ControllerButton` | **fertig** |
| Die Bindungen | `Binding::Pointer`, `Binding::Gesture`, `Binding::Controller` und ihre Schreibweisen | **fertig** |
| Die Erkennung | Der Übersetzer aus den Ereignissen des Rahmenwerks: Klick, Doppelklick, Rechtsklick, Ziehen, Rollen, Tippen, langer Druck, Wischen, Pinch | **fertig** |
| Die Tabelle | Fünf Kontexte in `config/keymap.toml` und als Vorgabe im Code | **fertig** |
| Die Oberfläche des Fingers | Gestapelte Bedienfelder, 44 Punkte hohe Zeilen, Blätter statt Dialoge, kein Hover | **offen** |
| Der Android-Bau | Projektstruktur, `cdylib`, SAF, NDK | **offen** — die Sandbox hat keinen NDK |

## 2. Der gemessene Stand

| Was | Wie gemessen | Ergebnis |
|---|---|---|
| Die Tests des Workspace | `cargo test --workspace --locked` | **1316 grün, 0 rot, 0 übersprungen** |
| Die Formatierung | `cargo fmt --all -- --check` | **ohne Abweichung** |
| Die Warnungen des Linters | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **ohne Warnung** |
| Die Konsistenz-Gates | `scripts/check-*.py`, `scripts/check-*.sh` | **13 Stück, alle PASS** |
| Die Gates beweisen sich selbst | `scripts/check-gates-negative.sh` | **38 Gates**, jedes wird rot und wieder grün. Gate 9 prüft zwei Übereinstimmungen und läuft deshalb viermal |

Die Zahl der Tests ist seit der Abnahme von M3 von 1277 auf **1316** gestiegen: 39 neue Tests
für die Ereignisse, die Schreibweisen, die Erkennung der Gesten, die fünf Kontexte, die Vorgaben
und das Überschreiben durch die Datei des Benutzers.

## 3. Was dieser Meilenstein gefunden hat

**Die Eingabe kannte nur die Tastatur.** `InputEvent` und `Binding` trugen nur `Key` und
`Palette`. Die Tabelle der Bindungen konnte damit **keinen Klick und keinen Tipp** benennen —
obwohl die Spezifikation in FR-101 genau das verlangt, und obwohl die Oberfläche bereits
`Context::Pointer` und `Context::Touch` kannte. Die beiden Kontexte waren vorhanden und trugen
nichts.

**Es gab keine Standardbindungen im Code.** Die Bindungen standen ausschließlich in
`config/keymap.toml`. Diese Datei wird aber **nie geschrieben** — sie entsteht beim Benutzer
gar nicht, und `read_keymap()` lieferte deshalb `None`. Die Oberfläche hatte damit **keine
einzige Bindung**: kein `F5`, kein `Tab`, nichts. Der Mangel fiel beim Schreiben der Vorgaben
auf, weil die Vorgabe `Grip` als Bindung im Controller-Kontext stand — und `Grip` ist keine
Taste. Die Datei war seit jeher unlesbar, und niemand hatte es bemerkt, weil niemand sie las.

**Der Wisch wurde doppelt ausgelöst.** Die Geste ging hinaus, als der Finger die Strecke
überquerte, und noch einmal, als er aufhob. Der Test der Geste hat es gefunden.

**Der Formatter und der Generator widersprachen sich.** Die erzeugte Datei der Vorgaben wurde
vom `cargo fmt` umgebrochen, und der Generator verglich die Bytes. Ein byteweiser Vergleich
meldet eine Abweichung, die keine ist. Der Vergleich prüft jetzt den **Inhalt** — die Bindungen
— und lässt die Schreibweise zu.

**Die Prüfung der Bindungen kannte nur drei Kontexte.** Sie sah `dpad` und `controller` und maß
an ihnen, ob alle Aktionen gebunden sind. Die Kontexte `pointer` und `touch` waren ungeprüft —
und wären auch leer geblieben, ohne dass ein Gate es gemerkt hätte.

**Der Formatierer von `benches/interaction.rs` war nicht gelaufen.** Der Commit, der die
Benchmarks brachte, war nach dem letzten `cargo fmt` gemacht. Das Gate war zu diesem Zeitpunkt
nicht grün.

## 4. Die Entscheidungen, die dieser Meilenstein trägt

| Entscheidung | Warum |
|---|---|
| **Der Code trägt die Vorgaben, die Datei überschreibt sie.** | Eine Datei, die fehlt, darf die Anwendung nicht der Bedienung berauben. `scripts/generate-keymap-defaults.py` erzeugt die Vorgaben aus `config/keymap.toml`, und `scripts/check-keymap-consistency.py` beweist, dass beide übereinstimmen. |
| **Die Datei des Benutzers überschreibt nur die Bindung, die sie nennt.** | Eine Datei mit einem kaputten Eintrag nimmt der Anwendung nicht die ganze Belegung. |
| **Der Kontext entscheidet, nicht die Quelle des Ereignisses.** | Die Übersetzung erzeugt jedes Signal, das die Oberfläche sieht. Die Tabelle des Kontexts entscheidet, welches Signal eine Aktion auslöst. Die Tastatur löst keinen Tipp aus, und der Finger keine Taste — das ergibt sich von selbst, weil die Tabelle des einen Kontexts die Signale des anderen nicht kennt. |
| **Die Gesten erkennt die Oberfläche selbst.** | Das Rahmenwerk liefert die rohen Ereignisse eines Fingers, aber keine Geste. Der lange Druck entsteht nicht aus einem Ereignis, sondern aus der Zeit, die der Finger ruht — die Oberfläche fragt daher einmal je Bild nach. |
| **Die Tasten des Gamepads sind eine eigene Art der Bindung.** | Der Griff, der Abzug und der Daumenstick sind keine Tasten einer Tastatur, und die Fernbedienung des Fernsehers nennt genau diese Signale. |

## 5. Was offen bleibt

| # | Was | Warum |
|---|---|---|
| 1 | Das Layout des Fingers | Die Bedienfelder werden gestapelt, die Zeilen sind 44 Punkte hoch, die Dialoge kommen als Blatt von unten, und es gibt keinen Hover. Die Metrik der Zeilenhöhe ist bereits kontextabhängig gesetzt, die Anordnung der Bedienfelder noch nicht. |
| 2 | Der Android-Bau | Die Sandbox hat keinen Android-NDK. Die Projektstruktur, das `cdylib` und die Anbindung an das Storage Access Framework sind ohne NDK zu schreiben, der Build nicht. |
| 3 | Die Tastenbelegung des Fernsehers und des Gamepads im Rahmenwerk | Die Ereignisse des Controllers sind benannt und gebunden; das Rahmenwerk muss sie noch liefern. Für die Fernbedienung des Fernsehers ist die Aktionspalette der vorgesehene Weg, und die ist bereits vorhanden. |
| 4 | Der Auslöser des Sprachwechsels | Unverändert offen — die Konfigurationsdatei wird einmal pro Sekunde gelesen. |
| 5 | Die Abnahme von M1 und M3 | Unverändert offen — beide brauchen echte Geräte. |

## 6. Die Belegstellen

| Thema | Datei |
|---|---|
| Die Ereignisse des Zeigers und des Fingers | `crates/phocommander-ui/src/keymap_view/gesture.rs` |
| Die Tasten des Gamepads | `crates/phocommander-ui/src/keymap_view/controller.rs` |
| Die Vorgaben der Bindungen | `crates/phocommander-ui/src/keymap_view/defaults.rs` |
| Die Erkennung der Gesten | `crates/phocommander-ui/src/input/pointer.rs` |
| Die Verarbeitung im Bild | `crates/phocommander-ui/src/app/input.rs` |
| Die Tabelle der fünf Kontexte | `config/keymap.toml` |
| Die Erzeugung der Vorgaben | `scripts/generate-keymap-defaults.py` |
| Die Prüfung der Übereinstimmung | `scripts/check-keymap-consistency.py` |
| Die Beschreibung der Bindungen | `docs/KEYMAP.md` |
