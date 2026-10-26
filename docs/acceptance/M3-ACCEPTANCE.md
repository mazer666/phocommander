# Abnahme des Meilensteins M3 — Windows, Linux und BSD

**Stand:** 26.10.2026 · **Prüfer:** Leitende Instanz · **Ergebnis:** angenommen mit zwei offenen Punkten

---

## 1. Was dieser Meilenstein umfasst

M3 bringt dieselbe Anwendung auf die anderen Desktop-Plattformen, aus derselben Codebasis:
der **Windows-Port** mit dem Manifest für die langen Pfade und einer verständlichen Meldung bei
Pfadlänge, der **Linux-Port** mit dem Wayland-Pfad und dem Rückfall auf X11, die
**FreeBSD- und NetBSD-Bauten** in der CI-Matrix und die **Paketierung** für die drei
Plattformen.

## 2. Der gemessene Stand

| Was | Wie gemessen | Ergebnis |
|---|---|---|
| Die Tests des Workspace | `cargo test --workspace --locked` | **1277 grün, 0 rot, 0 übersprungen** |
| Die Formatierung | `cargo fmt --all -- --check` | **ohne Abweichung** |
| Die Warnungen des Linters | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **ohne Warnung** |
| Die Lizenzen | `cargo deny check` | **advisories, bans, licenses, sources: ok** |
| Die Grenzen der Crates | `scripts/check-crate-boundaries.sh` | **13 Abhängigkeiten, 0 Verstöße** |
| Keine Telemetrie | `scripts/check-no-telemetry.sh` | **0 Verstöße** |
| Die sieben Konsistenz-Gates | `scripts/check-*.py` | **alle PASS** |
| Die Gates beweisen sich selbst | `scripts/check-gates-negative.sh` | **38 Gates**, jedes wird rot und wieder grün |

## 3. Was dieser Meilenstein gefunden hat

**Ein Fund, der die Sicherheitsarchitektur betraft.** Die gehärtete Extraktion benutzte das Crate
`cap-std`, und **das Crate braucht einen Nightly-Compiler**: es verlangt vom Compiler
Eigenschaften, die der stabile Compiler nicht erlaubt — in *jeder* Version. Die alten Versionen
helfen nicht, sie ziehen ein `rustix`, das ein dem Compiler längst entferntes Attribut benutzt.

Die Entscheidung ADR-0004 nannte `rustix` bereits als Alternative, und sie bietet denselben
Zugriff — den Deskriptor des Verzeichnisses, das relative Öffnen, das Verweigern der
symbolischen Links — **ohne eine einzige Zeile `unsafe`** in diesem Repository. Die Umstellung
ist geschehen, und **die Negativtests der Härtungsregeln H-02 bis H-06 halten gegen sie**.

**Zwei Fehler in der Kette der Meldungen.** Der Katalogschlüssel eines Fehlers und der
Fehlercode sind zwei verschiedene Dinge: der Code trägt einen Punkt (`path.too_long`), weil er
einen Fall in einer Falltabelle benennt, der Schlüssel trägt einen Bindestrich
(`path-too-long`), weil das Format des Katalogs keinen Punkt erlaubt. Und ein Schlüssel, der
zwei Bedeutungen trug, wurde vom Katalog abgelehnt — und damit war der **ganze Katalog
unbrauchbar**, woran fast alle i18n-Tests scheiterten.

**Ein Fund, der die Oberfläche betraft.** Die Auftragsliste hatte **keine Quelle**: die
Oberfläche kannte das Trait `JobSource`, aber die Desktop-App hat es nie umgesetzt. Die Liste
blieb leer, und der Benutzer sah nie, dass ein Auftrag fehlschlug.

## 4. Die zwei offenen Punkte

| Punkt | Grund | Wann |
|---|---|---|
| **Der erste Windows-Bau** | Das Manifest `longPathAware` und die einbettende Resource-Datei sind mit `rust-lld` bewiesen, nicht mit dem Linker von Windows. `link.exe` nimmt dieselbe `.res`-Datei, aber der erste Bau des Eigentümers ist der Beweis | beim ersten Windows-Bau |
| **Das manuelle Testprotokoll für das plattformspezifische Verhalten** | Die Sandbox hat ein Linux-System und keinen Bildschirm, kein Windows und kein BSD. Das Protokoll braucht diese Maschinen | mit den Geräten des Eigentümers |

Die übrigen Punkte der Roadmap sind erfüllt: das Manifest, die Meldung der Pfadlänge, die
Entscheidung Wayland gegen X11, die Bauten von FreeBSD und NetBSD und die Paketierung.

## 5. Die Entscheidungen, die dieser Meilenstein getroffen hat

| Entscheidung | Warum |
|---|---|
| **Die gehärtete Extraktion nutzt `rustix` statt `cap-std`** | `cap-std` braucht einen Nightly-Compiler. Ein Nightly-Compiler ist für ein Projekt, das sein Sicherheitsargument in safe Rust schreibt, keine Option. ADR-0004 nannte `rustix` bereits |
| **Die Dateiendung entscheidet das Format eines Containers nicht** | Die Magic Bytes entscheiden. Eine Datei, die `.zip` heißt und kein ZIP ist, soll als das erkannt werden, was sie ist |
| **Ein Skript wird über die API mit dem Modus gepusht, nicht über den Connector** | Der Connector trägt den Modus nicht, und ein Gate-Skript ohne Ausführungsbit läuft nicht |

## 6. Was als Nächstes kommt

1. Der **Eigenschaftstest der Umbenennung (FR-33)** — er ist inzwischen geschrieben und auf seine
   Wirksamkeit geprüft: mit einem echten Zufall statt des abgeleiteten Werts fallen beide
   Eigenschaften um.
2. Die **Benchmarks NFR-02 und NFR-03** im Release-Profil.
3. Die **Abnahme von M1** abschließen, sobald das manuelle Protokoll auf einem macOS-Gerät
   gelaufen ist.

---

*Die Annahme gilt für den Stand des Commits, mit dem die Protokolle geschrieben wurden. Jede
spätere Änderung an den Quellen macht die Messung ungültig.*
