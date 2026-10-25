# Der Stand des Projekts

**Letzte Aktualisierung:** 25.10.2026 · **Stand auf GitHub:** `859e24d`

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
| **M1** — The working file browser | Dateisystemmodell, Auftragswarteschlange, Umbenennung, Betrachter Stufe 1, Archive | **in Prüfung** — zwei Lücken offen | `docs/acceptance/M1-ACCEPTANCE.md` |
| **M2** — Maturity on macOS | Registerkarten, Profile, sichere Kopiermodi, Layouts je Kontext | **abgenommen** | `docs/acceptance/M2-ACCEPTANCE.md` |
| **M2b** — Internationalisierung | Englisch und Deutsch vollständig, lokalisierte Formatierung, Sprachwechsel zur Laufzeit | **in Arbeit** | `docs/ACCEPTANCE-CRITERIA.md`, Block M2b |
| **M3** — The other desktop platforms | Windows, Linux, BSD | offen | `docs/ROADMAP.md` |
| **M4** … **M9** | Mobil, iPadOS, Netzwerkquellen, Cloud, Fernseher, Rest | offen | `docs/ROADMAP.md` |

## 2. Was gerade läuft

| Was | Stand |
|---|---|
| **Die Tests der Container-Adapter** (ZIP, TAR, 7z) | Der Core Engineer schreibt sie. Die Dateien `tar_tests.rs` und `test_support.rs` stehen, `zip.rs` und `sevenz.rs` sind angefasst. Es wird **nicht gebaut** — das Bauen macht die Leitende Instanz zentral, weil die Sandbox 3 GB RAM hat und parallele Builds sich gegenseitig abbrechen |
| **Die Abnahme von M1** | Wird neu geschrieben, sobald die Container-Tests stehen. Das Dokument liegt in `docs/acceptance/M1-ACCEPTANCE.md` und nennt die offenen Punkte |

## 3. Der gemessene Stand

| Was | Zahl |
|---|---|
| Tests | **941 grün, 0 rot, 0 übersprungen** |
| Testbinäre | 47 |
| Gates, die sich selbst beweisen | **38** — jedes wird bei einem eingebauten Defekt rot und nach der Reparatur wieder grün |
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
16. Die Container-Adapter der Archive hatten **keinen einzigen Test**.
17. Der Negativtest benutzte das zentrale Zielverzeichnis und verunreinigte die Binaries.

## 5. Die Regeln, die aus diesem Meilenstein folgen

Diese Regeln sind aus den Fehlern entstanden. Sie stehen hier, weil sie beim nächsten Mal
genauso leicht zu brechen sind:

| Regel | Warum |
|---|---|
| **Ein einziger Cargo-Prozess im Workspace.** Jeder Agent bekommt ein eigenes `CARGO_HOME` und `CARGO_TARGET_DIR` auf lokaler Platte. | Drei Gigabyte RAM. Zwei Builds gleichzeitig bringen beide um — und der Agent wartet dann auf eine Antwort, die nie kommt. |
| **Der Negativtest baut in ein Zielverzeichnis von sich.** | Ein Testbinary trägt den Pfad des Repositorys, in dem es gebaut wurde. Ein Binary aus der Kopie sucht die Konfiguration der Kopie — und die ist weg, wenn das Skript endet. |
| **Die Agenten bauen nicht.** Sie schreiben, die Leitende Instanz baut und prüft. | Ein Bau in einem fremden Verzeichnis frisst 1 bis 5 GB und blockiert das NAS-Volume, auf dem das Repository liegt. |
| **Nach jedem Meilenstein wird aufgeräumt.** | Am Ende dieses Tages lagen 20 GB Target-Verzeichnisse in `/tmp`. |
| **Ein Gate, das nie rot war, ist ein Wunsch.** | Jedes neue Gate gehört in `check-gates-negative.sh`, mit einem eingebauten Defekt. |

## 6. Was als Nächstes kommt

| # | Was | Warum |
|---|---|---|
| 1 | Die Container-Tests bauen und prüfen | Die letzte Lücke der M1-Abnahme |
| 2 | Die Abnahme von M1 neu schreiben | Sie misst den Stand vor der Reparatur und nennt die alten Gründe |
| 3 | Den Meilenstein M2b abschließen | Die Kataloge sind synchron, das Gate steht; offen sind die lokalisierte Formatierung in der Oberfläche und der Wechsel zur Laufzeit |
| 4 | Die Benchmarks von NFR-02, NFR-03 und NFR-09a | Sie brauchen das Release-Profil und einen ruhigen Baum |
| 5 | Den Meilenstein M3 beginnen — Windows, Linux, BSD | Der erste Meilenstein, der auf einer anderen Plattform läuft |

## 7. Die Fragen, die eine Entscheidung brauchen

| Frage | Stand |
|---|---|
| Wie läuft das manuelle Protokoll auf einem echten macOS-Gerät? | Die Sandbox hat ein Linux-System und keinen Bildschirm. Die Abnahme von M1 braucht dieses Protokoll. |
| Welche Zertifikate tragen die Binaries? | Windows braucht ein Code-Signing-Zertifikat, macOS einen Developer ID. Beides kostet Geld und braucht eine Entscheidung des Eigentümers. |
| Das Zielverzeichnis der Tests liegt auf einem NAS-Volume | Jeder Build zahlt die Latenz des Volumes. Eine lokale Ablage wäre schneller — wenn die Sandbox sie anbietet. |

---

*Der Stand dieses Dokuments gehört zum Commit in der Kopfzeile. Wer ihn fortschreibt, aktualisiert
die Kopfzeile mit.*
