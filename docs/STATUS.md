# Der Stand des Projekts

**Letzte Aktualisierung:** 25.10.2026 · **Stand auf GitHub:** siehe `git log` — der letzte Commit trägt die Messung

Dieses Dokument ist das **Statusboard**. Es sagt, wo das Projekt steht, was gerade läuft und
was als Nächstes kommt. Die Roadmap sagt, *was* die Meilensteine enthalten; dieses Dokument
sagt, *wie weit* sie sind.

Die Quelle der Wahrheit für den Code ist das Repository selbst. Dieses Dokument beschreibt den
Stand eines Commits — mit dem Commit-Hash in der Kopfzeile.

---

## 1. Der Stand der Meilensteine

| Meilenstein | Inhalt | Stand | Nachweis |
|---|---|---|---|
| **M0** — Foundation | Workspace, CI, Konfiguration, Bedrohungsmodell, Dokumentation | **abgenommen** | `docs/acceptance/M0-ACCEPTANCE.md` |
| **M1** — The working file browser | Dateisystemmodell, Auftragswarteschlange, Umbenennung, Betrachter Stufe 1, Archive | **in Prüfung** — Engine und Dialog der Umbenennung sind fertig; offen sind das macOS-Protokoll und die Benchmarks | `docs/acceptance/M1-ACCEPTANCE.md` |
| **M2** — Maturity on macOS | Registerkarten, Profile, sichere Kopiermodi, Layouts je Kontext | **abgenommen** | `docs/acceptance/M2-ACCEPTANCE.md` |
| **M2b** — Internationalisierung | Englisch und Deutsch vollständig, lokalisierte Formatierung, Sprachwechsel zur Laufzeit | **in Arbeit** | `docs/ACCEPTANCE-CRITERIA.md`, Block M2b |
| **M3** — The other desktop platforms | Windows, Linux, BSD | offen | `docs/ROADMAP.md` |
| **M4** … **M9** | Mobil, iPadOS, Netzwerkquellen, Cloud, Fernseher, Rest | offen | `docs/ROADMAP.md` |

## 2. Was gerade läuft

| Was | Stand |
|---|---|
| **Die Abnahme von M1** | Die dritte Messung ist geschrieben. Drei Gründe bleiben: das manuelle Protokoll auf einem macOS-Gerät, die Benchmarks der Spezifikation und der Eigenschaftstest der Umbenennung |
| **Der Meilenstein M2b** | Die Kataloge sind synchron, das Gate gegen hartkodierte Texte steht, die Fehlermeldungen werden übersetzt, die Größen und Daten folgen der Sprache, und der Sprachwechsel erreicht die Oberfläche im nächsten Frame. Offen: der Auslöser des Wechsels — die Konfigurationsdatei wird einmal pro Sekunde gelesen, ein Menüeintrag wäre die dauerhafte Lösung |
| **Der Meilenstein M3** | Beginnt nach M2b — Windows, Linux und BSD |

## 3. Der gemessene Stand

| Was | Zahl |
|---|---|
| Tests | **1123 grün, 0 rot, 0 übersprungen** |
| Testbinäre | 54 |
| Gates, die sich selbst beweisen | **38** — jedes wird bei einem eingebauten Defekt rot und nach der Reparatur wieder grün. Zuletzt gemessen am 26.10.2026 nach den Ports von M3 und der Regel der Pfadlänge: **PASS**, 38 von 38 |
| Der Linter | `cargo clippy --all-targets -- -D warnings` — **keine Warnung** |
| Die Formatierung | `cargo fmt --all -- --check` — **ohne Abweichung** |
| Die Lizenzen | `cargo deny check` — advisories, bans, licenses, sources: **ok** |
| Die Konsistenz-Gates | 11 Stück, alle grün |
| Die Sprachkataloge | 3 Sprachen, 115 Schlüssel, gleicher Schlüsselsatz |

## 4. Was in diesem Meilenstein gefunden wurde

Der Meilenstein ist nicht grün, weil nichts passiert wäre. Die Prüfung hat **echte Fehler**
gefunden — im Code und in den Tests:

**Fehler im Code**

1. Die Liste der Textschlüssel zählte 106 Einträge und enthielt 107.
2. Der Wächter der temporären Datei trug ein `Debug`, das der Merkmalsträger nicht erfüllen kann.
3. Der Übersetzer trug ein `Debug`, das das Bündel der Fluent-Bibliothek nicht erfüllen kann.
4. Die Aufzählung der Verzeichniseinträge lieferte Dateinamen statt der Einträge, die die Vergleiche brauchen.
5. Der Prüfplan der Umbenennung meldete denselben Konflikt zweimal.
6. Die Prüfung der Archive verglich die Prüfsumme auch dann, wenn der Eintrag keinen Inhalt hatte — ein symbolischer Link wurde als beschädigt gemeldet.
7. Die leere Tastenbelegung meldete „unbekannte Taste" statt „leere Belegung".
8. Die Meldungen der Fehler standen in englischen Texten im Quelltext — die Statuszeile zeigte sie dem Benutzer an.
9. Die Zeichenfunktionen trugen acht bis neun Parameter, die immer dieselben waren.

**Fehler in den Tests, die falsche Annahmen trugen**

10. Zwei Tests des Verzeichnisbaums erwarteten die Wurzel des Volumes nicht, obwohl der Code sie vorsieht.
11. Zwei Tests der Umbenennung prüften eine Lage, die kein Konflikt ist.
12. Zwei Tests des Vergleichs verglichen ein Verzeichnis, das es nicht gab.
13. Ein Test der Prüfsumme übergab keinen Inhalt und erwartete trotzdem ein Ergebnis.
14. Der Integrationstest der Verzeichniseinträge war toter Code — er war nie im Manifest.

**Lücken, die die Prüfung aufgedeckt hat**

15. Die Warteschlange der Aufträge hatte **keinen einzigen Test** — das Wiederholen der fehlgeschlagenen Einträge war implementiert, aber nie gelaufen.
16. Die Container-Adapter der Archive hatten **keinen einzigen Test** — jetzt haben sie 32.
17. Die Erkennung des Formats kannte **kein komprimiertes TAR** — die vier Komprimierungen des Adapters waren über die öffentliche API unerreichbar.
18. Die Datei `keymap_view.rs` war auf 999 Zeilen gewachsen und damit über dem Maximum von 900. Sie ist jetzt geteilt: die Eingabe in `keymap_view/stroke.rs`, die Belegung in `keymap_view.rs`.
19. Der Negativtest benutzte das zentrale Zielverzeichnis und verunreinigte die Binaries.
20. `AGENTS.md` nannte 400 und 550 Zeilen, wo die Spezifikation (NFR-14) und das Gate 600 und 900 sagen. Die Datei sagt selbst, dass die Spezifikation gewinnt.
21. Drei Gate-Skripte hatten auf GitHub das Ausführungsbit verloren. Der Weg, der den Inhalt schreibt, trägt den Modus nicht — deshalb gibt es jetzt `scripts/restore-executable-bits.sh`.
22. **Die Umbenennung von M1 war nur zur Hälfte vorhanden.** Die *Ausführung* existiert und ist getestet — atomar, mit Konflikterkennung und dem Zurücknehmen eines gescheiterten Umbenennens. Die *Bildung* des neuen Namens aus einer Vorlage fehlte: die Platzhalter, das Suchen und Ersetzen, der Zähler, die Determinismus, die Vorschau, die Konfliktliste, die gespeicherten Vorlagen und die Nullauffüllung.
26. Der **abgeleitete Zufallswert** wurde mit sechzehn Zeichen geschrieben statt mit acht: die Breite eines Formats ist ein *Minimum*, kein Maximum. Ein 64-Bit-Hash passt nicht in acht Stellen, ohne dass er gekürzt wird.
23. Das Modul `format` des i18n-Crates war **nie kompiliert** — es war nicht deklariert. Der erste Build des Crates wäre rot gewesen.
23. Fluent registriert seine eingebauten Funktionen **nicht** von selbst. `NUMBER($count)` war deshalb eine unbekannte Funktion, und die Nachricht fiel auf den Schlüssel zurück.
24. Fluent 0.16 schreibt eine Zahl **ohne** die Trennzeichen der Sprache — das Feld `use_grouping` existiert, wird aber nicht benutzt. Die Zahlen der Oberfläche tragen die Trennzeichen jetzt, weil der i18n-Crate sie selbst mit den Regeln von CLDR formatiert.
25. Ein Argument der Oberfläche hatte **keinen Namen**.
27. **`cargo clippy` war stundenlang kaputt** und sah aus wie ein sauberer Code. Der Compiler liest seinen Quelltext beim Aufruf mit „-" vom Standard-Input, und das trug die Ausgabe des vorherigen Kommandos. Der Wrapper `scripts/cargo.sh` leert es und zeigt cargo auf den echten Treiber des Toolchains.
28. **Der Negativtest hat den eigenen Fehler der Leitenden Instanz gefunden.** Nach der Umstellung auf `Iterator::find` war der Quelltext nicht mehr nach `cargo fmt` formatiert — und `cargo fmt --check` sah das nicht, weil der Aufruf über das verseuchte Standard-Input fehlschlug und der Fehler als „failed to run rustc" daherkam. Der Gate-Test fand es, weil er `cargo fmt` **in einer Kopie** aufruft, in der das Standard-Input leer ist.
29. **Das Standard-Input der Shell trägt die Ausgabe des vorherigen Kommandos.** `rustc -` liest es als Rust-Quelltext und stirbt mit „unterminated character literal". Das war die Ursache der „Operation aborted"-Meldungen — **keine** Speicherprobleme, sondern ein verseuchtes stdin. Jeder cargo-Aufruf bekommt jetzt `< /dev/null`. Der Katalog konnte die Werte nicht zuordnen, und ein Argument, das zwei Zahlen trägt, wäre nicht zuordenbar gewesen.

## 5. Die Regeln, die aus diesem Meilenstein folgen

Diese Regeln sind aus den Fehlern entstanden. Sie stehen hier, weil sie beim nächsten Mal
genauso leicht zu brechen sind:

| Regel | Warum |
|---|---|
| **Ein einziger Cargo-Prozess im Workspace.** Jeder Agent bekommt ein eigenes `CARGO_HOME` und `CARGO_TARGET_DIR` auf lokaler Platte. | Drei Gigabyte RAM. Zwei Builds gleichzeitig bringen beide um — und der Agent wartet dann auf eine Antwort, die nie kommt. |
| **Der Negativtest baut in ein Zielverzeichnis von sich.** | Ein Testbinary trägt den Pfad des Repositorys, in dem es gebaut wurde. Ein Binary aus der Kopie sucht die Konfiguration der Kopie — und die ist weg, wenn das Skript endet. |
| **Die Agenten bauen nicht.** Sie schreiben, die Leitende Instanz baut und prüft. | Ein Bau in einem fremden Verzeichnis frisst 1 bis 5 GB und blockiert das NAS-Volume, auf dem das Repository liegt. |
| **Nach jedem Meilenstein wird aufgeräumt.** | Am Ende dieses Tages lagen 20 GB Target-Verzeichnisse in `/tmp`. |
| **Nach jedem `git reset` das Ausführungsbit wiederherstellen.** | `./scripts/restore-executable-bits.sh` — der Weg, der den Inhalt schreibt, trägt den Modus nicht. |
| **Jeder cargo-Aufruf bekommt `< /dev/null`.** | Das Standard-Input der Shell trägt die Ausgabe des vorherigen Kommandos, und `rustc -` liest sie als Rust-Quelltext. |
| **Ein Gate, das nie rot war, ist ein Wunsch.** | Jedes neue Gate gehört in `check-gates-negative.sh`, mit einem eingebauten Defekt. |

## 6. Was als Nächstes kommt

| # | Was | Warum |
|---|---|---|
| 1 | Das manuelle Protokoll von M1 auf einem macOS-Gerät | Die Sandbox hat ein Linux-System und keinen Bildschirm. Die Abnahme braucht dieses Protokoll, sonst bleibt M1 offen |
| 2 | Die Benchmarks von NFR-02, NFR-03 und NFR-09a | Sie brauchen das Release-Profil und einen ruhigen Baum |
| 3 | Den Auslöser des Sprachwechsels festlegen | Der Desktop-Build liest die Konfigurationsdatei einmal pro Sekunde. Ein Eintrag in der nativen Menüleiste wäre die dauerhafte Lösung — `muda` kommt mit M3 |
| 4 | Den Meilenstein M3 vorbereiten — Windows, Linux, BSD | Der erste Meilenstein, der auf einer anderen Plattform läuft. Zuerst die Portabilitätsfragen: der Trenner der Pfade, die langen Pfade unter Windows, die Tastenbelegung je Plattform und die Matrix der CI |

## 7. Die Fragen, die eine Entscheidung brauchen

| Frage | Stand |
|---|---|
| Wie läuft das manuelle Protokoll auf einem echten macOS-Gerät? | Die Sandbox hat ein Linux-System und keinen Bildschirm. Die Abnahme von M1 braucht dieses Protokoll. |
| Welche Zertifikate tragen die Binaries? | Windows braucht ein Code-Signing-Zertifikat, macOS einen Developer ID. Beides kostet Geld und braucht eine Entscheidung des Eigentümers. |
| Das Zielverzeichnis der Tests liegt auf einem NAS-Volume | Jeder Build zahlt die Latenz des Volumes. Eine lokale Ablage wäre schneller — wenn die Sandbox sie anbietet. |
| Der Release-Plan war widersprüchlich | Linux mit Alpha 2 und Windows mit Beta 1 — obwohl beide Ports zu M3 gehören. Korrigiert am 25.10.2026: Alpha 1, Alpha 2 und Beta 1 liefern macOS aus, Beta 2 liefert alle fünf Desktop-Plattformen |
| Der Eigenschaftstest der Umbenennung (FR-33) | Die Spezifikation nennt ihn ausdrücklich. Er ist der letzte fehlende Test der Abnahme von M1. |
| Soll die Dateiendung ein Merkmal sein? | **Nein**, entschieden am 25.10.2026: die Endung bleibt ein Hinweis. Die Magic Bytes entscheiden. Eine Datei, die `.zip` heißt und kein ZIP ist, soll als das erkannt werden, was sie ist. |
| Verschlüsselte Container (FR-68) | Nicht angefasst. Der Meilenstein M1 umfasst die unverschlüsselten Formate; die verschlüsselten kommen mit dem Umgang mit den Passwörtern. |

---

*Der Stand dieses Dokuments gehört zum Commit in der Kopfzeile. Wer ihn fortschreibt, aktualisiert
die Kopfzeile mit.*
