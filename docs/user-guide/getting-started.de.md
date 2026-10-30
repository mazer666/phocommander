# Erste Schritte mit Phocommander

> Deutsch | [English](getting-started.en.md)

Phocommander ist ein Zwei-Fenster-Dateimanager mit eingebautem Betrachter. Auf dieser
Seite erfahren Sie, wie Sie das Programm starten, wie Sie sich darin bewegen, welche
zehn Tasten Sie zuerst brauchen und wie Sie eine Datei kopieren.

**Verfügbarkeit:** Die Bedienung über die Tastatur und die beiden Panels stehen ab dem
Meilenstein **M1** zur Verfügung, das Kopieren ebenfalls. Die Menüleiste und die
Registerkarten kommen mit **M2**. Der vollständige Tastenkatalog steht in
[`docs/KEYMAP.md`](../KEYMAP.md), die verbindliche Spezifikation in
[`docs/REQUIREMENTS.md`](../REQUIREMENTS.md).

---

## 1. Programm starten

Phocommander ist ein Rust-Programm ohne Installationsassistenten. Nach dem Start
sehen Sie sofort die Arbeitsfläche — keine Landingpage, keine Anmeldung, kein
Konto.

| Plattform | Start |
|---|---|
| macOS | `cargo run -p phocommander-desktop` aus dem Repository, oder die fertige `.app` aus dem Release |
| Windows, Linux, BSD | `cargo run -p phocommander-desktop` aus dem Repository, oder das Paket der Distribution |
| Android, iPadOS | Noch nicht verfügbar — geplant für M4 (Android) und M5 (iPadOS) |

> **Stand M0:** Das Programm startet, nennt Hostplattform, primäre Modifikatortaste,
> Konfigurationsverzeichnis und Schema-Version der Konfiguration und beendet sich
> wieder. Es öffnet noch kein Fenster — eine leere Oberfläche wäre schlechter als
> ein Programm, das sagt, was es ist. Die Arbeitsfläche erscheint mit M1.

Beim ersten Start öffnet sich links ein Panel mit dem Benutzerverzeichnis, rechts ein
Panel mit demselben Verzeichnis. Beide Panels sind gleichwertig: Welches Panel die
aktive Aktion trifft, entscheidet der Fokus.

> **Zwei Panels, eine Quelle, ein Ziel.** Links steht die Quelle, rechts das Ziel —
> aber die Panels sind nicht festgelegt. Beide können Quelle oder Ziel sein. Der
> Wechsel des Zielpanels ist immer eine ausdrückliche Handlung (`Ctrl+Enter`), damit
> keine Datei versehentlich im falschen Verzeichnis landet.

---

## 2. Navigieren

| Taste | Wirkung |
|---|---|
| `↑` `↓` | Eine Zeile nach oben oder unten |
| `PgUp` `PgDn` | Eine Seite weiter |
| `Home` `End` | Anfang oder Ende der Liste |
| `Ctrl+PgUp` | Eine Ebene höher |
| `Tab` | Das aktive Panel wechseln |
| `Ctrl+Enter` | Die Markierung im anderen Panel öffnen |
| `Ctrl+R` | Das Verzeichnis neu einlesen |
| `Ctrl+U` | Die beiden Panels tauschen |

Im Verzeichnisbaum eines Panels blenden Sie mit den Pfeiltasten in die
Ordnerstruktur. Beim Verlassen eines Ordners bleibt die Ansichtart erhalten —
derselbe Verzeichnistyp wird immer gleich dargestellt.

**Die Eingabezeile** steht am unteren Rand. Dort tippen Sie einen Pfad ein und drücken
Enter, um direkt dorthin zu springen — schneller als sich durch die Hierarchie zu
klicken. Eine Kommandozeilenhistorie ist über `Ctrl+Down` erreichbar.

> Der Verzeichnisbaum, die Eingabezeile und das Filterfeld sind Bestandteil von
> **M2**. Vorher navigieren Sie über die Tasten der Tabelle.

---

## 3. Die ersten zehn Tasten

| Taste | Wirkung |
|---|---|
| `F2` | Umbenennen — bei mehreren markierten Dateien öffnet sich der Mehrfach-Umbenennungsdialog |
| `F3` | Betrachter öffnen |
| `F4` | Editor — geplant für M9, die Belegung ist reserviert |
| `F5` | Kopieren in das andere Panel |
| `F6` | Verschieben in das andere Panel |
| `F7` | Neues Verzeichnis anlegen |
| `F8` | Löschen |
| `F9` | Menü |
| `F10` | Beenden |
| `Tab` | Aktives Panel wechseln |
| `Ctrl+Enter` | Markierung im anderen Panel öffnen |

Die vollständige Liste mit Kontext und Plattformvariante steht in
[`docs/KEYMAP.md`](../KEYMAP.md).

---

## 4. Eine Datei kopieren

Das ist der erste Vorgang, den Sie üben sollten — er zeigt das Zwei-Fenster-Prinzip
in einer Minute.

1. **Quellpanel wählen.** Drücken Sie `Tab`, bis das Panel mit der Datei, die Sie
   kopieren wollen, das aktive ist. Das aktive Panel ist der hell umrandete.
2. **Datei markieren.** Bewegen Sie den Cursor mit den Pfeiltasten auf die Datei und
   drücken Sie `Ins`. Die Datei ist markiert.
3. **Ziel wählen.** Drücken Sie `Ctrl+Enter` — die Datei wird im anderen Panel
   geöffnet. Das Zielpanel ist damit das Verzeichnis, in das kopiert wird. Sie
   können aber auch im Zielpanel navigieren, ohne die Datei zu öffnen.
4. **Kopieren.** Drücken Sie `F5`.

Der Vorgang läuft als **Auftrag** im Hintergrund: die Oberfläche bleibt bedienbar,
der Auftrag zeigt Fortschritt, eine Ratenangabe und lässt sich abbrechen.

### Was bei einem Konflikt passiert

Existiert im Ziel bereits eine Datei mit demselben Namen, entscheidet die
Konfliktstrategie — **Überschreiben ist nie die Voreinstellung**:

| Strategie | Wirkung |
|---|---|
| `ask` (Vorgabe) | Sie werden gefragt |
| `never` | Vorhandene Datei bleibt, der Auftrag meldet den Konflikt |
| `always` | Vorhandene Datei wird überschrieben |
| `keep_both` | Beide bleiben erhalten, die neue Datei wird umbenannt |

Die Strategie steht in der Konfigurationsdatei unter `[panels] overwrite_policy`.

---

## 5. Mehrere Dateien markieren

| Taste | Wirkung |
|---|---|
| `Ins` | Markierung des Eintrags unter dem Cursor umschalten |
| `Ctrl+A` | Alle Einträge markieren |
| `Ctrl+Num-` | Markierung aufheben |
| `Ctrl+Num*` | Markierung invertieren |
| `Leertaste` | Markierung nach einem Muster — tippen Sie danach das Muster ein |

F5, F6 und F8 wirken auf die Markierung. Ist nichts markiert, wirkt die Aktion auf
den Eintrag unter dem Cursor.

---

## 6. Datei ansehen

Drücken Sie `F3`. Der Betrachter öffnet sich in einem zusätzlichen Bereich, ohne dass
ein externes Programm startet. In der ersten Stufe zeigt er:

- Text und Quellcode (mit Zeichensatz-Erkennung, UTF-8 und UTF-16),
- Hexdump (Offset, Hex, ASCII),
- Bilder (PNG, JPEG, GIF, WebP, BMP, TIFF),
- den Inhalt eines Archivs,
- Metadaten (Größe, Typ, Zeitstempel, Prüfsumme).

PDF und Syntaxhervorhebung folgen mit **M2**, Audio und Video mit **M9**. Der
Betrachter ist **schreibgeschützt** — er verändert keine Datei.

Im Betrachter suchen Sie mit `F3` weiters und mit `Shift+F3` zurück.

---

## 7. Wo Ihre Einstellungen liegen

Phocommander schreibt eine einzige Konfigurationsdatei im TOML-Format an dem Ort,
den Ihr Betriebssystem dafür vorsieht:

| Plattform | Ort |
|---|---|
| macOS | `~/Library/Application Support/Phocommander/` |
| Windows | `%APPDATA%\Phocommander\` |
| Linux, BSD | `$XDG_CONFIG_HOME/phocommander/` (Standard: `~/.config/phocommander/`) |
| Android, iPadOS | Sandbox der App |

Darin stehen die Einstellungen, die Tastenbelegung, die Favoriten und — wenn Sie es
wünschen — die zuletzt geöffneten Verzeichnisse. Mehr dazu in
[`PRIVACY.md`](../../PRIVACY.md).

### Die Sprache wechseln

Die Oberfläche spricht Deutsch, Englisch und Französisch. Sie wechseln die Sprache
während das Programm läuft — ohne Neustart, und ohne dass etwas verloren geht:

| Aktion | Taste |
|---|---|
| Deutsch | `Ctrl+Shift+D` |
| Englisch | `Ctrl+Shift+E` |
| Französisch | `Ctrl+Shift+F` |

Auf macOS und iPadOS ist `Ctrl` die `Cmd`-Taste. Auf einem Touchbildschirm stehen die
drei Einträge im Menü der Anwendung.

Die Wahl wird in die Konfigurationsdatei geschrieben und bleibt damit nach einem
Neustart erhalten. Geändert wird nur die eine Zeile, die die Sprache trägt — die
Kommentare in Ihrer Datei bleiben, wo sie sind.

---

## 8. Eine Datei löschen

`F8` löscht die markierten Einträge. Eine gelöschte Datei verschwindet **nicht**: sie landet
im **Papierkorb Ihres Betriebssystems**, genau wie eine Datei, die Sie im Dateimanager des
Systems in den Papierkorb ziehen. Sie können sie von dort zurückholen, bis Sie den Papierkorb
leeren.

Ein Ordner landet als Ganzes im Papierkorb, mit allem, was er enthält.

Eine Ausnahme gibt es: eine **Synchronisation** löscht endgültig. Sie haben ihren Plan gesehen
und bestätigt, und der Plan nennt jede Datei, die geht. Ein Papierkorb, der eine Kopie des Ziels
jeder Synchronisation enthielte, füllte das Medium.

Ein Datenträger ohne Papierkorb — eine Netzwerkfreigabe, eine eingebundene Festplatte — meldet
den Fehler, statt still zu löschen. Wählen Sie dort einen anderen Weg, um die Datei zu
entfernen.

---

## 9. Wenn etwas nicht funktioniert

| Symptom | Ursache und Abhilfe |
|---|---|
| Eine Taste tut nichts | Die Bindung wurde geändert. Prüfen Sie die `[keymap]`-Sektion in der Konfigurationsdatei und die Tabelle in [`docs/KEYMAP.md`](../KEYMAP.md). |
| `Ctrl` wirkt nicht auf macOS | Auf macOS und iPadOS ist `Cmd` der primäre Modifikator. `Ctrl+Enter` heißt dort `Cmd+Enter`. |
| Eine Datei ist im falschen Verzeichnis gelandet | `Ctrl+U` tauscht die Panels, `Ctrl+Enter` öffnet im anderen Panel. Prüfen Sie vor dem Kopieren, welches Panel aktiv ist. |
| Der Papierkorb nimmt nichts an | Auf einem gemounteten Laufwerk oder einem Netzlaufwerk gibt es keinen Papierkorb. Wählen Sie beim Löschen eine andere Vorgehensweise. |
| Der Betrachter zeigt eine Binärdatei als Text | `F3` erkennt den Inhalt nicht. Der Hexdump (`F3` im Betrachter, dann Hex-Ansicht) zeigt immer die Rohdaten. |

Weitere Hinweise finden Sie in der [Glossarübersicht](../glossary.md) und in der
[Spezifikation](../REQUIREMENTS.md).

---

## Weiterlesen

- [Mehrfach umbenennen](multi-rename.de.md) — Serienfolgen, Platzhalter, Konfliktsprung
- [Tastenkürzel-Referenz](../KEYMAP.md) — alle Aktionen, alle Kontexte
- [Glossar](../glossary.md) — die Begriffe, die in diesem Projekt eine eigene Bedeutung haben
