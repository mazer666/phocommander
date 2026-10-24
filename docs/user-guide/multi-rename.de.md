# Mehrfach umbenennen

> Deutsch | [English](multi-rename.en.md)

Der Mehrfach-Umbenennungsdialog ist die Funktion, für die es sich lohnt, Phocommander
zu benutzen: Sie markieren zwanzig Dateien, beschreiben einmal die Regel, und der
Dialog benennt alle zwanzig um — mit einer Vorschau, mit Konflikterkennung und ohne
stille Überraschungen.

**Verfügbarkeit:** Der Dialog kommt mit **M1**, zusammen mit den benannten Vorlagen.
Details zum Umfang stehen in [`docs/REQUIREMENTS.md`](../REQUIREMENTS.md), die
Tastenbelegung in [`docs/KEYMAP.md`](../KEYMAP.md).

---

## 1. Den Dialog öffnen

1. Markieren Sie die Dateien, die umbenannt werden sollen — mit `Ins`, `Ctrl+A`
   oder per Muster über die Leertaste.
2. Drücken Sie `F2`.

Bei einer einzelnen markierten Datei öffnet sich das einfache Umbenennungsfeld. Bei
mehreren Dateien öffnet sich der Mehrfach-Umbenennungsdialog.

---

## 2. Der Aufbau des Dialogs

| Bereich | Inhalt |
|---|---|
| **Links** | Die Liste der ausgewählten Dateien |
| **Rechts** | Die Tabelle aus *Originalname* und *Neuer Name* — mit einer eigenen Spalte für die erkannten Felder |
| **Unten** | Die Regeldefinition: Platzhalter, Ersetzung, Zähler |

Die Spalte mit den erkannten Feldern ist der eigentliche Zweck des Dialogs: Sie
sehen vor dem Klick, ob die Erkennung funktioniert hat. Bei einer Serie mit
uneinheitlichen Dateinamen ist genau das der Punkt, an dem man sonst zwanzig Dateien
falsch benennt.

---

## 3. Wie eine Regel ausgewertet wird

Die drei Bestandteile der Regel werden **in dieser Reihenfolge** ausgewertet:

### 3.1 Platzhalter

Platzhalter sind Textbausteine, die beim Umbenennen durch einen Wert ersetzt werden.

| Platzhalter | Ersetzt durch |
|---|---|
| `[name]` | Basisname der Datei ohne Endung |
| `[ext]` | Dateiendung |
| `[parent]` | Name des übergeordneten Ordners |
| `[n]` | laufender Zähler |
| `[date]` | Datum |
| `[time]` | Uhrzeit |
| `[size]` | Dateigröße |
| `[hash]` | Kurz-Prüfsumme |
| `[rand]` | Zufallswert |
| `[case]` / `[lower]` / `[upper]` / `[title]` | der Text in Großbuchstaben, kleinbuchstaben, in Groß- und Kleinschreibung oder in Titelschreibung |

### 3.2 Ersetzen

Suchtext und Ersetzungstext, optional als **regulärer Ausdruck** zuschaltbar. Ohne
Regex wird der Text wörtlich gesucht.

### 3.3 Zähler

Startwert, Schrittweite, Stellenzahl mit führenden Nullen und die Reihenfolge, in der
die Dateien den Zähler bekommen — zum Beispiel in der Reihenfolge der Markierung, im
Dateinamen oder nach Datum.

---

## 4. Vorschau und Konflikte

Die **Live-Vorschau** zeigt den neuen Namen jeder markierten Datei, während Sie die
Regel bearbeiten.

Berechnet der Dialog einen Namen, den es im Zielverzeichnis schon gibt, wird der
Eintrag **farblich markiert** und in eine eigene **Konfliktsprung-Liste** aufgenommen:

| Taste | Wirkung |
|---|---|
| `F3` | Zum nächsten Konflikt springen |
| `Shift+F3` | Zum vorherigen Konflikt springen |

So sehen Sie alle Konflikte in einem Durchgang, ohne die Liste verlassen zu müssen.

---

## 5. Vorlagen

Regeln lassen sich als **benannte Vorlagen** speichern und wieder anwenden. Jede
Vorlage ist eine TOML-Datei im Konfigurationsverzeichnis und kann von Ihnen angepasst
werden.

| Vorlage | Zielformat |
|---|---|
| **Serienfolge** | `{serie} - S{staffel:02}E{episode:02} - {titel}.{format}` |
| **Kamera** | `{n}_{serie}_{titel}.{format}` — der Zähler wird dreistellig geführt (`{n:03}`) |
| **Podcast** | `{date}_{titel}.{format}` — das Datum als `{date:YYYY-MM-DD}` |
| **Rechnung** | `{year}_{number}_{description}.{format}` — Jahr, Nummer, Beschreibung |
| **Musik** | `{artist}_{title}_{track}.{format}` — Interpret, Titel, Tracknummer (`{track:02}`) |

Die Vorlagen liegen im Konfigurationsverzeichnis unter `templates/`. Eine eigene
Vorlage legen Sie an, indem Sie im Dialog eine Regel speichern und ihr einen Namen
geben.

---

## 6. Serienfolgen — das Muster im Detail

Das häufigste Anwendungsfall: Episoden einer Serie nach dem Muster
`Serienname - S01E01 - Titel der Folge.Format` benennen.

### Das Beispiel

Sie haben den Ordner `The Expanse` mit den Dateien

```
Duldung.avi
Leviathan.avi
Homegrown.avi
```

Der Dialog erkennt in `Duldung.avi` kein Muster `SxxEyy`. Er fragt deshalb nach
Staffel und Episodennummer und **merkt sich die Eingabe** — die nächste Datei wird
mit denselben Werten vorbelegt. Sie tragen also nur einmal Staffel `1` und Episode
`1` ein, bestätigen die Vorbelegung bei `Leviathan.avi` und erhöhen die Episode auf
`2`.

Der **Serienname** stammt aus dem übergeordneten Verzeichnis: Der Ordner heißt
`The Expanse`, also ist der Serienname `The Expanse`. Das ist der Standard — er wird
aber immer angezeigt und ist überschreibbar, denn bei Serien in Sammelordnern stimmt
er sonst nicht.

Das Ergebnis:

```
Duldung.avi     →  The Expanse - S01E01 - Duldung.avi
Leviathan.avi   →  The Expanse - S01E02 - Leviathan.avi
Homegrown.avi   →  The Expanse - S01E03 - Homegrown.avi
```

### Zwei- versus dreistellig

`S01E01` und nicht `S1E1`: Staffel- und Episodennummer werden zweistellig
nullgefüllt. Liegt in der Auswahl eine Folge mit dreistelliger Nummer, hebt der
Dialog die Stellenbreite automatisch auf drei an — sonst wären die Dateien eines
Laufs nicht einheitlich benannt:

```
Staffel 1, Episode 100  →  The Expanse - S01E100 - … .avi
```

Wird die Staffel ebenfalls dreistellig, folgt sie demselben Prinzip.

### Bereits benannte Dateien

Enthält ein Dateiname bereits ein Muster `SxxEyy` — etwa `The.Expanse.S01E03.avi` —
übernimmt der Dialog Staffel und Episode als Vorbelegung und bietet den restlichen
Namen als Episodentitel an. Das spart bei Folgen, die schon sortiert abgelegt sind,
das erneute Eintippen.

### Unzulässige Zeichen

Der Episodentitel wird **unverändert** übernommen. Ein Doppelpunkt, ein Schrägstrich
oder ein Fragezeichen sind unter Windows unzulässig und führen dort zu einem Fehler.

Der Dialog weist darauf hin, **bevor** die Umbenennung ausgeführt wird, und schlägt
einen Ersatz vor:

```
The Expanse - S01E01 - Chapter 1: The Big One.avi
                              ^  Doppelpunkt — unzulässig unter Windows
Vorschlag:  The Expanse - S01E01 - Chapter 1 - The Big One.avi
```

Nichts wird stillschweigend geändert — Sie entscheiden.

---

## 7. Wenn etwas nicht stimmt

| Symptom | Ursache und Abhilfe |
|---|---|
| Die Episode wird nicht hochgezählt | Die Regel enthält keinen Zähler. Fügen Sie unter „Zähler" einen Startwert und eine Schrittweite hinzu. |
| Der Serienname ist der Ordnername, nicht die Serie | Der Serienname stammt standardmäßig aus dem übergeordneten Verzeichnis. Im Dialog ist er überschreibbar. |
| Die Vorschau bleibt leer | In der Regel fehlt der Platzhalter, der den Namen erzeugt — meist `{name}` oder `{titel}`. |
| Ein Name wird rot angezeigt | Der Name existiert bereits im Zielverzeichnis. `F3` springt zum nächsten Konflikt, `Shift+F3` zum vorherigen. |
| Die Erkennung greift nicht | Das Muster `SxxEyy` wird case-insensitiv gesucht. Ein Name wie `Staffel 1, Folge 2` enthält das Muster nicht und wird abgefragt. |

---

## Weiterlesen

- [Erste Schritte](getting-started.de.md) — starten, navigieren, eine Datei kopieren
- [Tastenkürzel-Referenz](../KEYMAP.md) — alle Aktionen, alle Kontexte
- [Glossar](../glossary.md) — Platzhalter, Konfliktsprung, virtuelles Verzeichnis
