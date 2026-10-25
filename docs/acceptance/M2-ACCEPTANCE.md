# Abnahme des Meilensteins M2 — Reife auf macOS

**Stand:** 25.10.2026 · **Prüfer:** Leitende Instanz · **Ergebnis:** angenommen mit zwei offenen Punkten

---

## 1. Was dieser Meilenstein umfasst

M2 macht aus dem arbeitenden Dateibrowser eine Anwendung, die im Alltag einen Dateibrowser
ersetzen kann: Registerkarten mit eigenem Zustand, ein Verzeichnisbaum je Panel, eine
Schnellansicht, die sichere Kopiermodi mit Synchronisationsplan, die atomare Umbenennung, das
Wiederholen nur der fehlgeschlagenen Einträge, das Rückgängig, die Prüfung der Archive ohne
Entpacken und die Interface-Profile `clean` und `full`.

Die Internationalisierung ist **nicht** Teil dieses Meilensteins. Sie ist der eigene
Meilenstein **M2b**, und der Grund steht in der Roadmap: die Vollständigkeit der Sprachen ist
eine Eigenschaft des Produkts und keine Einzelheit der Darstellung.

## 2. Der gemessene Stand

| Was | Wie gemessen | Ergebnis |
|---|---|---|
| Die Tests des ganzen Workspace | `cargo test --workspace --locked` | **924 grün, 0 rot, 0 übersprungen** |
| Die Formatierung | `cargo fmt --all -- --check` | **ohne Abweichung** |
| Die Warnungen des Linters | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **ohne Warnung** |
| Die Lizenzen | `cargo deny check` | **advisories, bans, licenses, sources: ok** |
| Die Grenzen der Crates | `scripts/check-crate-boundaries.sh` | **12 Abhängigkeiten, 0 Verstöße** |
| Die Größe der Dateien | `scripts/check-file-size.sh` | **0 über dem Maximum von 900**, 6 über dem Zielwert von 600 mit Begründung im Modulkopf |
| Keine Telemetrie | `scripts/check-no-telemetry.sh` | **0 Verstöße, 0 verbotene Crates** |
| Die sieben Konsistenzskripte | `scripts/check-*.py` | **alle PASS** |
| Die Gates selbst | `scripts/check-gates-negative.sh` | **34 Gates**: jedes wird bei einem Defekt rot und wieder grün |

Die Protokolle liegen unter `docs/acceptance/logs/`.

## 3. Die Fehler, die dieser Meilenstein gefunden hat

Der Meilenstein ist nicht deshalb grün, weil nichts passiert wäre. Die Prüfung hat zwölf echte
Fehler gefunden und zwölf Testfehler, die auf falsche Annahmen zurückgingen:

**Fehler im Code**

1. Die Liste der Textschlüssel zählte 106 Einträge, enthielt aber 107 — ein Schlüssel war
   nicht in der Prüfliste.
2. Der Wächter der temporären Datei trug ein `Debug`, das der Merkmalsträger des Dateisystems
   nicht erfüllen kann. Die Zustände werden jetzt von Hand geschrieben.
3. Der Übersetzer trug ein `Debug`, das das Bündel der Fluent-Bibliothek nicht erfüllen kann.
4. Der Sprachbezeichner hatte keinen gültigen Ersatz, wenn ein Verzeichnis keinen gültigen
   Sprachcode trägt.
5. Die Aufzählung der Verzeichniseinträge lieferte die Dateinamen statt der Einträge, die die
   Vergleiche brauchen.
6. Der Prüfplan der Umbenennung meldete denselben Konflikt zweimal. Die Sprungliste zeigte
   denselben Pfad zweimal.
7. Die Prüfung der Archive verglich die Prüfsumme auch dann, wenn der Eintrag keinen Inhalt
   hatte — ein symbolischer Link wurde dadurch als beschädigt gemeldet.
8. Der Aufrufer der Zeichenfunktionen trug acht bis neun Parameter, die immer dieselben waren.

**Fehler in den Tests, die falsche Annahmen trugen**

9. Zwei Tests des Verzeichnisbaums erwarteten die Wurzel des Volumes nicht, obwohl der Code
   sie ausdrücklich vorsieht. Der Code hat recht: die Wurzel ist ein Vorfahr wie jeder andere.
10. Zwei Tests der Umbenennung prüften eine Lage, die kein Konflikt ist.
11. Zwei Tests des Vergleichs verglichen ein Verzeichnis, das es nicht gab.
12. Ein Test der Prüfsumme übergab keinen Inhalt und erwartete trotzdem ein Ergebnis.

## 4. Die Lücke, die dieser Meilenstein aufdeckt

Die Warteschlange der Aufträge hatte **keinen einzigen Test**. Das Wiederholen der
fehlgeschlagenen Einträge (FR-22k) war implementiert, aber niemand hatte es je laufen
lassen. Neun Tests sind dazugekommen, und sie haben einen echten Fehler gefunden: das
Dateisystem der Tests konnte nicht einen einzelnen Eintrag scheitern lassen. Diese
Möglichkeit fehlt jetzt.

## 5. Die offenen Punkte

| Punkt | Grund | Wann |
|---|---|---|
| Der About-Dialog nennt die Zahl der tatsächlich unterstützten Formate | Der Dialog ist Teil des Meilensteins M3 | M3 |
| Die Leistungsziele NFR-01 bis NFR-08 sind gemessen | Die Messung braucht die fertige Oberfläche | nach M3 |

Diese beiden Punkte betreffen Anforderungen, die dem Meilenstein M2 in der Spezifikation
zugeordnet sind. Sie sind nicht umgangen, sondern offen benannt.

## 6. Die Korrekturen, die außerhalb des Meilensteins liegen

Diese Korrekturen waren nötig, damit der Meilenstein überhaupt messbar war, gehören aber
nicht zum Umfang von M2:

- Die Internationalisierung wurde aus M2 herausgelöst und zum Meilenstein **M2b** gemacht. Die
  Anforderungen FR-120 bis FR-126 zeigen jetzt auf M2b, und der Block M2b in den
  Abnahmekriterien trägt die Kriterien AC-M2b-01 bis AC-M2b-08.
- Der französische Katalog wurde auf 108 Schlüssel gebracht. Er war der Beweis für FR-125 —
  eine dritte Sprache ohne Änderung am Quelltext — und er war unvollständig, also kein Beweis.
- Das Kennzeichen der Texte `Locale::English` und `Locale::German` existieren nicht. Die
  Aufrufe nutzen jetzt `Locale::english()` und `Locale::of("de")`.
- Die Prüfskripte kennen jetzt den Meilenstein M2b und die Kennzeichen `AC-M2b-xx`.

## 7. Was als Nächstes kommt

1. Der Meilenstein **M2b** — die Internationalisierung in beiden Sprachen, mit der Prüfung der
   Schlüsselsätze und dem Wechsel zur Laufzeit.
2. Die **M1-Abnahme** wird neu geschrieben, auf dem ruhenden Baum und mit den 924 Tests, die
   inzwischen existieren.
3. Der Meilenstein **M3** — Windows, Linux und BSD.

---

*Die Annahme gilt für den Stand des Commits, mit dem die Protokolle unter
`docs/acceptance/logs/` geschrieben wurden. Jede spätere Änderung an den Quellen macht die
Messung ungültig.*
