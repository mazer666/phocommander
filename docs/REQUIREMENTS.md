# Phocommander — Anforderungsspezifikation 1.1

**Status:** Entwurf zur Abstimmung · **Datum:** 24.10.2026 · **Änderung 1.1:** Eingabemodelle, Netzwerk- und Cloud-Zugriffe, Produktmaximen · **Sprache des Projekts:** Englisch (UI zweisprachig DE/EN)
**Repository-Ziel:** `github.com/mazer666/phocommander` (public) · **Lizenz:** `MIT OR Apache-2.0`

---

## Zweck und Geltungsbereich

Phocommander ist ein Datei-Browser-Ersatz mit eingebautem Betrachter für macOS, Android, iPadOS, Windows, Linux und BSD. Das Dokument legt Produktumfang, Architektur, Sicherheits- und Qualitätsanforderungen fest und ist die verbindliche Grundlage für alle weiteren Arbeitspakete.

Der Geltungsbereich umfasst die Kern-Desktop-Funktionalität, die gemeinsame Rust-Bibliothek für alle sechs Zielplattformen sowie die plattformspezifischen Verpackungen. Nicht umfasst sind Cloud-Speicher-Anbindungen Dritter, ein Makro-Skriptsprachen-Interpreter mit VBScript-Kompatibilität, Brennfunktionen für optische Medien und ein AddIn-Binärinterface nach dem Vorbild von SpeedCommander — diese Funktionen sind in der Zielarchitektur vorgesehen, aber nicht Teil des ersten Auslieferungsumfangs.

### Clean-Room-Erklärung

Phocommander ist eine eigenständige Implementierung. Grundlage sind ausschließlich öffentlich dokumentierte Beobachtungen zum Funktionsumfang klassischer Zwei-Fenster-Dateimanager der Norton-Commander-Linie sowie der Quasi-Standard-Tastenbelegung, die sich seit den 1980er-Jahren als De-facto-Standard etabliert hat [4][36]. Weder Code, noch Bilder, noch Icons, noch Texte, noch Layoutvorgaben des Originalherstellers werden übernommen, gespiegelt oder nachgebaut.

Kopiert wird ausschließlich die *Idee* einer Funktion: zwei synchron sichtbare Verzeichnisfenster, ein eingebetteter Betrachter, ein Mehrfach-Umbenennungsdialog mit Platzhaltern. Die konkrete Umsetzung, das Bedienkonzept, die Oberflächengestaltung und die Codebasis sind eigenständig. Der Begriff „Clean Room" bezeichnet in der Softwareentwicklung ein Vorgehen, bei dem die Spezifikation ohne Kenntnis der Originalimplementierung erstellt wird — genau so wird hier verfahren [2][4].


---

## Produktvision

Phocommander richtet sich an Menschen, die große Dateimengen bewusst ordnen, vergleichen, umbenennen und sichten — im Terminal, in der IDE, in der Bildverwaltung, im Backup. Die Zielgruppe arbeitet den ganzen Tag mit Dateien und empfindet den Finder oder den Explorer als zu ablenkend, zu langsam oder zu unklar.

Der Nutzen liegt in drei Punkten, die gemeinsam den Produktkern ausmachen:

**Sichtbarkeit beider Seiten.** Quelle und Ziel sind gleichzeitig im Bild. Dadurch ist vor dem Kopieren klar, wohin kopiert wird, und Verzeichnisse lassen sich direkt gegenüberstellen, ohne den Explorer zu verlassen.

**Tastatur als Hauptweg.** Die vollständige Bedienung funktioniert ohne Maus. Das ist kein Komfort-Feature, sondern der Grund, warum sich Zwei-Fenster-Manager über Jahrzehnte halten. Eine Eingabezeile und ein Filterfeld beschleunigen Sprung und Selektion um Größenordnungen gegenüber reinem Maus-Navigieren.

**Betrachter im Fenster.** Dateien werden angezeigt, ohne dass ein externes Programm startet. Das spart bei der Sichtung von Logdateien, Screenshots, Konfigurationen und Codefragmenten den Anwendungskontext-Wechsel und ist der Hauptgrund, weshalb SpeedCommander und Forklift beide einen eingebauten Betrachter führen [1][5].

### Abgrenzung zu SpeedCommander und Forklift

SpeedCommander ist das funktionale Referenzprodukt: Zwei-Fenster-Technik, 13 Archivformate, Betrachter für über 80 Formate, integrierte FTP/FTPS/SFTP-Anbindung, Ordnersynchronisation, Mehrfach-Umbenennung, Dateisuche, integrierte Kommandozeile, Makros und AddIn-Schnittstelle [1][2][3]. Die Oberfläche wirkt, wie der Nutzer selbst anmerkt, trocken — sie ist dicht, funktional und auf reine Tastaturbedienung ausgelegt.

Forklift ist das Qualitätsreferenzprodukt auf macOS: native Menüs, Finder-ähnliche Anmutung, Quick Look, Tags, Vorwärts/Rückwärts-Navigation, Time-Machine-Wiederherstellung [5][38]. Es gilt als das funktionalste Dateimanager-Werkzeug für macOS, verlangt aber eine Lizenz und deckt iOS/iPadOS nicht ab.

Phocommander übernimmt von SpeedCommander das Bedienkonzept und von Forklift die Sorgfalt bei Oberfläche, Integration und Fehlermeldungen. Ziel ist nicht, eines der beiden Produkte zu kopieren, sondern deren jeweilige Stärken in einem freien, plattformübergreifenden Werkzeug zusammenzuführen.

| Merkmal | SpeedCommander [2][3] | Forklift [5] | Phocommander (Ziel) |
|---|---|---|---|
| Plattformen | Windows 7–11 | macOS | macOS, Android, iPadOS, Windows, Linux, BSD |
| Fensterprinzip | Zwei-Fenster, oben/unten | Zwei-Fenster, auch einzeln | Zwei-Fenster, oben/unten, Registerkarten |
| Betrachter | 80+ Formate | Quick Look, PDF, Text | Text, Bild, PDF, Hex, Archiv, Metadaten (gestuft) |
| Archivformate | 13 | ZIP u. a. | ZIP, TAR (+Kompression), 7z, GZip, BZip2, XZ, Zstd |
| Mehrfach-Umbenennen | Platzhalter, Konfliktsprung F3 | Filter/Parameter, Presets | Platzhalter, Regex, Vorschau, Konfliktsprung |
| Fernzugriff | FTP, FTPS, SFTP | FTP, SFTP, SMB, WebDAV, Cloud | SFTP, FTP (später) |
| Bedienung | rein tastaturorientiert | Tastatur + Maus | rein tastaturorientiert, Maus optional |
| Lizenz | Shareware (22.40) | kostenpflichtig | MIT OR Apache-2.0 |

Die Gegenüberstellung zeigt die Lücke, die Phocommander schließen will: die Bedienidee und die Tastaturorientierung des einen Produkts, die Integrationsqualität des anderen — und dazu eine freie Lizenz und sechs Plattformen, die keines der beiden erreicht.

---

---

## Produktmaximen

Drei Devisen bestimmen das Produkt. Sie stehen hier, weil sie keine Absichtserklärung sind, sondern **prüfbare Anforderungen** — jede spätere Entscheidung wird gegen sie gemessen, und das CI-Gate überwacht sie.

### Kostenlos

Keine Lizenzgebühr, kein Abo, keine In-App-Käufe, keine geschlossenen Funktionen. Jede Funktion, die in einer Version existiert, existiert in allen Versionen identisch — es gibt keinen „Pro"-Zweig. Keine Wartungsgebühr für Sicherheitsupdates. Kein Build-Zeit-Check gegen eine Lizenzserver-Instanz. Der Quellcode baut die Anwendung ohne Netzwerkzugang.

Prüfbar durch: eine CI-Prüfung, die bestätigt, dass kein Crate der Telemetrie- oder Analyse-Kategorien referenziert wird, und ein Test, der belegt, dass der Prozessstart keine Netzwerkverbindung öffnet.

### Clutter-frei

Keine Werbebanner, keine Empfehlungsspalten, keine Nachrichtendienste, kein Konto-Zwang, kein aufdrängender Onboarding-Tour. Der Startbildschirm ist die Arbeitsfläche, nicht eine Landingpage. Keine App-interne Werbung, keine gesponserten Treffer in Such- oder Verbindungslisten. Keine Einstellungen, die es nicht braucht.

Prüfbar durch: ein Screenshot-Abgleich der Startansicht je Kontext, und eine Prüfung, dass die Verbindungsliste ausschließlich vom Nutzer selbst angelegte Einträge enthält.

### Werbefrei

Keine Werbung, kein Tracking, keine Analyse, keine Telemetrie, keine Nutzungsprofile, keine Weitergabe an Dritte, keine Affiliate-Links, keine gesponserten Platzierungen. Lokale Funktionen funktionieren ohne Registrierung und ohne Login. Der Quellcode zeigt es: Es gibt keine Netzwerkanfrage, die der Nutzer nicht selbst ausgelöst hat.

Prüfbar durch: die Zusage in `PRIVACY.md`, den Null-Telemetrie-Nachweis im Sicherheitsdokument, und die CI-Prüfung auf Telemetrie-Abhängigkeiten.

---

## Eingabemodelle und Bedienkontexte

Phocommander wird auf sechs Plattformen und in bis zu fünf Bedienkontexten bedient. Der zentrale Satz dieser Spezifikation lautet: **Bedienung ist nicht an Geräte, sondern an Aktionen gebunden.**

### Das Aktionsmodell

Jede bedienbare Funktion ist eine benannte Aktion mit stabiler Aktions-ID (`panel.copy`, `panel.view`, `selection.toggle`). Die Eingabeschicht übersetzt physische Eingaben in Aktions-IDs. Auf Desktop liefert `winit` Tastatur-, Maus- und Touch-Ereignisse, Gamepad-Ereignisse jedoch nur auf Redox — auf Linux weder unter X11 noch unter Wayland [48]. Für den `dpad`- und den `controller`-Kontext heißt das: Die Eingabe kommt **nicht** aus `winit`, sondern aus der jeweiligen Plattform-Shell. Die `controller`-Anbindung auf Desktop ist deshalb eine eigene Aufgabe mit der Variante, Gamepads über eine plattformnative Bibliothek anzubinden, nicht über `winit`. Die Oberflächenschicht kennt ausschließlich Aktions-IDs und weiß nicht, ob eine Aktion per Taste, Klick, Wischgeste, D-Pad-Taste oder Controller-Trigger ausgelöst wurde.

Diese Trennung ist keine Abstraktion um der Schönheit willen. Sie ist die Voraussetzung dafür, dass dieselbe Anwendung auf einem MacBook mit Tastatur, auf einem iPad mit Fingern, auf einem Fire TV mit Fernbedienung und in einer Quest mit Controller ohne vier getrennte Oberflächen funktioniert.

Der Aktionskatalog liegt in `config/keymap.toml` und wird von der Konfigurationsdatei referenziert. Die Datei `docs/KEYMAP.md` dokumentiert jede Aktion mit Vorgabe, Kontext und Plattformvariante.

### Bedienkontexte

Jeder Kontext beschreibt, wie ein bestimmtes Eingabegerät die Aktions-IDs erreicht. Die Vorgabe wird je Kontext getrennt gespeichert, damit ein Nutzer auf dem Laptop dieselbe Anwendung auf dem Fernseher mit anderen Belegungen sieht.

| Kontext | Eingabegerät | Verfügbare Signale | Layoutprinzip |
|---|---|---|---|
| `keyboard` | Tastatur | Tasten, Modifier, Eingabetext | Vollständige Aktionsbelegung, Norton-Commander-Kurztasten |
| `pointer` | Maus | Zeiger, Klick, Doppelklick, Rechtsklick, Ziehen, Scrollrad | Zwei Panels nebeneinander, Menüleiste, Iconleiste, Hover |
| `touch` | Finger | Tippen, Long-Press, Wischen, Zwei-Finger-Scroll, Pinch | Gestapelte Panels, große Ziele, Sheet-Dialoge, kein Hover |
| `dpad` | TV-Fernbedienung | D-Pad, OK, Zurück, Home, Play/Pause | Ein Panel sichtbar, 10-Fuß-Skalierung, Fokusrahmen |
| `controller` | Gamepad, VR-Controller | Trigger, Griff, Thumbstick, A/B/X/Y, D-Pad | Räumliche Anordnung, Strahlzeiger, keine Modifier |

### Die Aktionspalette als Ersatz für F-Tasten

Das ist der zentrale Konflikt zwischen dem Bedienkonzept und der Fernbedienung. Die Norton-Commander-Kurztasten `F2` bis `F10` tragen die Hauptfunktionen — umbenennen, ansehen, kopieren, verschieben, löschen. Auf einer Fire-TV- oder Google-TV-Fernbedienung existieren diese Tasten nicht. Nur Richtungstasten, OK, Zurück und Home stehen zur Verfügung. Auch auf einem VR-Controller gibt es keine F-Tasten.

Die Lösung ist die **Aktionspalette**: eine durchsuchbare Liste aller Aktionen, die im aktuellen Kontext verfügbar sind. Im `dpad`-Kontext öffnet sie sich über die Menü- oder Play/Pause-Taste und ist vollständig mit Richtungstasten und OK bedienbar. Sie ersetzt funktional die F-Tastenreihe und ist auf jedem Kontext verfügbar — auch auf dem Desktop als „Aktionen suchen".

Die Palette filtert dabei nach Kontext: Im `dpad`-Kontext erscheinen nur Aktionen, die ohne Maus und ohne Modifier sinnvoll ausführbar sind. Eine Aktion, die eine gedrückte Umschalt-Taste braucht, ist dort nicht erreichbar und wird nicht angezeigt.

### Fokus und Auswahl als getrennte Zustände

Auf Plattformen ohne Zeiger gibt es kein Hover. Der Fokus muss deshalb ein **sichtbarer, dauerhaft erkennbarer Zustand** sein, nicht ein Nebenprodukt der Mausposition.

Im `dpad`- und `controller`-Kontext werden Fokus und Auswahl getrennt geführt. Der Fokus bezeichnet die Zeile, auf die die nächste Aktion wirkt. Die Auswahl bezeichnet die markierten Dateien, die eine Aktion betrifft. Beim Navigieren wandert der Fokus, ohne die Auswahl zu verändern. Beim Anwählen wird die Auswahl verändert, ohne den Fokus zu verlieren.

Die Mehrfachauswahl braucht auf Fernbedienung einen expliziten Modus, weil die Tastenkombination aus Tastatur und Maus nicht existiert. Der `dpad`-Kontext bietet deshalb einen Auswahlmodus an, in dem OK die Markierung umschaltet und die Richtungstasten den Fokus bewegen. Der Modus wird sichtbar angezeigt und lässt sich jederzeit verlassen.

### Adaptive Layouts je Kontext

Die Layouts unterscheiden sich strukturell, nicht nur in Farbe und Schriftgröße.

Im `keyboard`- und `pointer`-Kontext stehen beide Panels gleichzeitig sichtbar. Quelle und Ziel sind auf einen Blick erfassbar, das ist der Kern des Bedienkonzepts.

Im `touch`-Kontext sind die Panels gestapelt: eine Dateiliste füllt den Bildschirm, das zweite Panel liegt darunter und wird per Wischgeste oder über eine Umschaltfläche erreicht. Zeilenhöhen sind mindestens 44 Punkte hoch. Dialoge erscheinen als Sheet von unten, nicht als zentrale Box. Hover-Effekte entfallen, weil es sie nicht gibt.

Im `dpad`-Kontext ist **ein Panel sichtbar**. Zwei gleichzeitige Panels wären auf einem Fernseher unlesbar. Das zweite Panel erscheint als Overlay, wenn eine Aktion ein Ziel braucht. Schriftgrößen sind für einen Betrachtungsabstand von etwa drei Metern ausgelegt, Fokusrahmen deutlich kräftiger als auf dem Desktop. Die Bedienhinweise („OK öffnet, Zurück geht zurück") sind sichtbar, weil eine Fernbedienung keine Tastenbeschriftung hat.

Im `controller`-Kontext werden die Panels räumlich angeordnet, nicht in einem Fenster. Die Bedienung erfolgt über einen Strahlzeiger mit Trigger-Auswahl; der Fokus liegt auf dem Strahlzeiger, nicht auf der Zeile.

### Meta Quest als Android-Konfiguration

Horizon OS ist ein Android-Betriebssystem. Eine Horizon-OS-Anwendung ist ein normales Android-Programm, das als **flaches Panel** im Raum steht; Touch-Gesten wie Tippen, Wischen, Scrollen und Pinchen werden vom System auf Controller- und Hand-Tracking-Eingaben abgebildet [44][45].

Daraus folgt die Empfehlung: **Kein immersiver VR-Port.** Die Android-Version wird um einen `controller`-Bedienkontext erweitert, und Quest bedient die Anwendung als Panel. Das spart ein vollständiges OpenXR- und Vulkan-Frontend, das für einen Dateimanager keinen Mehrwert liefert. Die OpenXR-Bibliothek `openxr` steht unter `MIT OR Apache-2.0` [46] und wäre verfügbar — sie wird für eine mögliche spätere räumliche Variante vorgehalten, aber nicht für M0 bis M9 eingeplant.

Die Einschränkung ist ehrlich zu benennen: Im Panel-Modus ist die Bedienung auf Gesten und Controller-Aktionen beschränkt. Ein Dateimanager in VR lebt von Tastatur und Feinzeiger — beides hat ein Panel im Raum nicht. Quest ist deshalb als **Bedienkontext auf der Android-Version** eingeplant und nicht als eigenständiges Ziel.

### Fire TV und Google TV

Auf Fernsehern ist das D-Pad die primäre Navigation [40]. Google Play prüft TV-Anwendungen darauf, ob die **gesamte Funktionalität mit einem Fünf-Wege-D-Pad und der Zurück-Taste** erreichbar ist [41]. Weitere Vorgaben:

- Die Anwendung darf nicht auf eine Menü-Taste der Fernbedienung angewiesen sein [41].
- Zurück navigiert zurück und führt aus der Anwendung heraus, ohne Endlosschleifen [40].
- `android.hardware.touchscreen` wird als nicht erforderlich deklariert, `CATEGORY_LEANBACK_LAUNCHER` gesetzt, ein 320×180-Pixel-Banner hinterlegt.
- Der Zeiger ist auf keinen Fall das primäre Bedienmittel.

Für Phocommander bedeutet das: Der `dpad`-Kontext ist eine **vollwertige Bedienart**, keine Notlösung. Jede Funktion, die im `keyboard`-Kontext erreichbar ist, muss im `dpad`-Kontext über die Aktionspalette erreichbar sein. Diese Regel wird bei der Abnahme mit einem Prüfprotokoll belegt: die Liste aller Aktionen wird gegen ihre Erreichbarkeit im `dpad`-Kontext geprüft.

---

## Netzwerk- und Cloud-Zugriffe

### Quellen statt Funktionen

Netzwerkzugriff ist kein eigener Funktionsbereich, sondern eine weitere **Quelle** in derselben Abstraktion. Eine SMB-Freigabe, ein WebDAV-Ordner, ein SFTP-Server und ein S3-Bucket liefern denselben „Ordnerinhalt" wie das lokale Dateisystem. Die Bedienung ist für alle Quellen identisch — kopieren, verschieben, umbenennen, löschen, ansehen, durchsuchen.

Das ist die Konsequenz aus der Architekturentscheidung in der Plattform-Matrix: Die Dateisystemabstraktion kennt keine Quellenart. Jede Quelle implementiert dieselbe Schnittstelle, und der Kern bleibt frei von Protokolldetails.

### Protokollunterstützung

| Protokoll | Rust-Bibliothek | Lizenz | Anmerkung |
|---|---|---|---|
| SFTP | `russh` 0.64 | Apache-2.0 | Reines Rust, keine C-Abhängigkeit |
| FTP, FTPS | `suppaftp` 12.2 | MIT OR Apache-2.0 | Reines Rust |
| SMB 2/3 | `smb2` 0.35 | MIT OR Apache-2.0 | Reines Rust, pipelined I/O |
| WebDAV | eigener Client auf `reqwest` | Apache-2.0 / MIT | `fast-dav-rs` ist LGPL-3.0 und fällt damit weg |
| S3-kompatibel | `rusty-s3` 0.10 | BSD-2-Clause | AWS, Backblaze B2, Wasabi, MinIO |
| OAuth 2.0 | `oauth2` 5.0 | MIT OR Apache-2.0 | Für Cloud-Anmeldung |

Alle genannten Versionen und Lizenzen sind gegen das crates.io-Register geprüft [47].

Der WebDAV-Client wird **selbst geschrieben** statt `fast-dav-rs` zu verwenden. Die Bibliothek ist funktional passend und LGPL-3.0-lizenziert [26]. LGPL erlaubt die Verwendung, verpflichtet aber bei statisch gelinkten Binaries zur Offenlegung oder Relinkbarkeit — das steht im Widerspruch zur Doppellizenz und zur Devise „kostenlos ohne Kompromisse". Der(WebDAV-)Zugriff braucht ohnehin nur `PROPFIND`, `GET`, `PUT`, `DELETE` und `MKCOL` [39], also ist der eigene Client klein und ohne Megabyte-Drittanbieter-Abhängigkeit gebaut.

`pavao` (MIT) wäre für SMB eine Alternative, bindet aber `libsmbclient` ein und damit eine Systembibliothek. Das erschwert die Quer-Kompilierung für Android und iOS. `smb2` ist reines Rust und damit die bessere Wahl.

### SMB-Sicherheitsregeln

SMB ist das Protokoll mit der höchsten Angriffsfläche in dieser Liste. Die Regeln sind verbindlich:

- **SMB1 wird nicht implementiert.** Das Protokoll ist deprecated, kennt keine moderne Signatur und war die Grundlage von EternalBlue; Microsoft liefert es seit Windows 10 1903 und Windows Server 2019 nicht mehr aus [43].
- **NTLMv1 wird nicht verwendet.** Microsoft hat die NTLMv1-Netzwerkauthentifizierung in Windows 11 24H2 und Windows Server 2025 entfernt und plant, NTLM insgesamt standardmäßig abzuschalten [42]. Wo NTLM nötig wird, ist es NTLMv2.
- **SMB-Signing wird verlangt, wenn der Server es unterstützt.** Ohne Signatur sind Relay-Angriffe möglich, bei denen ein Angreifer die Authentifizierung eines ahnungslosen Nutzers weiterreicht; Windows 11 24H2 verlangt die Signatur bei frischen Installationen bereits [43].
- **Transportverschlüsselung (SMB 3) wird bevorzugt, Zugangsdaten liegen im Schlüsselbund.** Die Verschlüsselung schützt die Vertraulichkeit der Daten auf dem Weg, und Microsoft rät, sie zusätzlich zur Signatur einzusetzen, wo alle Geräte sie unterstützen [43]. Zugangsdaten stehen nie in der Konfigurationsdatei.

Ein Verbindungsversuch gegen einen Server, der nur SMB1 oder nur NTLMv1 anbietet, schlägt mit einer verständlichen Meldung fehl. Er wird nicht „probiert und vielleicht klappt".

### Cloud-Zugriff

Cloud-Zugriff bedeutet OAuth 2.0 gegen den Anbieter und direkte API-Aufrufe über HTTPS. **Phocommander betreibt keinen eigenen Vermittlungsserver.** Jede Anmeldung läuft direkt zwischen der Anwendung und dem Anbieter; das Refresh-Token liegt im Schlüsselbund.

Diese Entscheidung folgt aus der Devise „kostenlos, clutter-frei, werbefrei": Ein eigener Server wäre ein Betrieb, eine Kostenstelle, eine Datenverarbeitungsstelle und eine Datenschutzfrage — also das Gegenteil dessen, was das Produkt verspricht. Auch die Datenschutz-Zusage bleibt so vollständig: Es fließen nur die Daten, die der Nutzer selbst anfordert.

S3-kompatible Ziele decken den großen Teil der privaten Cloud-Nutzung ab, weil Nextcloud, ownCloud, Seafile und viele NAS-Systeme WebDAV anbieten und die kommerziellen Object-Storage-Dienste S3 sprechen. Beide Wege sind abgedeckt, ohne einen Anbieter zu bevorzugen.

### Verbindungsdaten und Protokollierung

Zugangsdaten liegen im Schlüsselbund des Betriebssystems (Keychain, Credential Manager, Secret Service). Auf Plattformen ohne Schlüsselbund fällt die Anwendung auf die App-interne Sandbox zurück und weist darauf hin, dass die Daten dann nicht vom Betriebssystem geschützt sind.

Verbindungen werden in einer vom Nutzer angelegten Liste verwaltet. Verbindungsnamen, Serveradressen und Benutzernamen gelten als personenbezogene Daten und erscheinen nicht im Protokoll. Das Protokoll nennt Protokoll, Zeitpunkt, Ergebnis und Dauer, niemals den Server oder den Benutzer.

---

## Funktionsumfang

### Zwei-Fenster-Arbeitsfläche

Zwei unabhängige Verzeichnisfenster stehen nebeneinander (Standard) oder untereinander. Beide zeigen gleichzeitig Pfad, Dateiliste und Spalten (Name, Größe, Datum, Typ). Die Anordnung ist umschaltbar, ohne Neustart.

Jedes Fenster besitzt eine Registerkartenleiste. Jede Registerkarte hält einen eigenen Verzeichnispfad, eine eigene Sortierung und eigene Anzeigeeinstellungen. Registerkarten lassen sich zwischen den beiden Fenstern verschieben.

Ein Verzeichnisbaum ist je Fenster zuschaltbar. Beim Verlassen eines Ordners bleibt die Ansichtart erhalten — derselbe Verzeichnistyp wird immer gleich dargestellt [3].

Ein dritter Bereich ist optional zuschaltbar: eine Schnellansicht, die den Inhalt der markierten Datei anzeigt, ohne sie zu öffnen. SpeedCommander führt diese Funktion als „SpeedView" [2].

**Zustandsmodell der Arbeitsfläche.** Die Arbeitsfläche besteht aus genau zwei Panel-Objekten. Jedes Panel besitzt: aktive Registerkarte, Pfad, Cursorposition, Sortierung, Auswahlmenge, Ansichtmodus. Operationen wirken immer auf das *aktive* Panel; die Übergabe ans andere Panel erfolgt explizit (Tastatur `Ctrl+Enter`, oder per Drag & Drop). Eine Fehlbedienung kann so keine Datei versehentlich in das falsche Verzeichnis kopieren.

### Dateioperationen

Kopieren, Verschieben, Umbenennen, Löschen und Erstellen von Verzeichnissen sind die Grundoperationen. Jede Operation läuft über eine Auftragswarteschlange im Hintergrund, sodass die Oberfläche bedienbar bleibt — der Nutzer kann während einer laufenden Kopie weiterarbeiten. SpeedCommander bietet dieselbe Unterbrechbarkeit für Archivoperationen [3].

Verhalten bei Konflikten ist explizit und konfigurierbar: überschreiben, überspringen, beide behalten (umbenennen), abfragen, oder den Vorgang abbrechen. Überschreiben ist **nicht** die Voreinstellung.

Ein Rückgängig-Bereich für Operationen ist vorgesehen, aber klar als begrenzt gekennzeichnet: Rückgängig für Kopiervorgänge bedeutet „Kopie löschen“, nicht „Zielzustand wiederherstellen“. Bei Dateioperationen außerhalb des Papierkorbs gibt es keine Transaktions- garantie.

### Mehrfaches Umbenennen

Der Mehrfach-Umbenennungsdialog ist die Funktion, die der Nutzer ausdrücklich hervorhebt. Er arbeitet in drei Zonen: links die Liste der ausgewählten Dateien, rechts eine Tabelle aus *Originalname* und *Neuer Name*, unten die Regeldefinition.

Die Regelauswertung kombiniert, in dieser Reihenfolge:

1. **Textbausteine (Platzhalter)** — `[name]` Basisname, `[ext]` Endung, `[parent]` übergeordneter Ordner, `[n]` laufender Zähler, `[date]` Datum, `[time]` Uhrzeit, `[size]` Dateigröße, `[hash]` Kurz-Prüfsumme, `[rand]` Zufallswert, `[case]` / `[lower]` / `[upper]` / `[title]` Groß-/Kleinschreibung.
2. **Ersetzen** — Suchtext und Ersetzungstext, optional als regulärer Ausdruck. SpeedCommander bietet reguläre Ausdrücke in der Pro-Version an [1]; für Phocommander ist Regex optional zuschaltbar.
3. **Zähler** — Startwert, Schrittweite, Stellenzahl mit führenden Nullen, Sortierung der Zählerfolge.

Der Dialog zeigt eine Live-Vorschau des Ergebnisses. Konflikte — der berechnete Name existiert bereits — werden farblich markiert und in einer eigenen Konfliktsprung-Liste geführt, die mit `F3` (nächster) und `Shift+F3` (vorheriger) durchlaufbar ist. Dieses Muster stammt aus SpeedCommander 18.00.9200 und hat sich bewährt [2].

Regeln lassen sich als benannte Vorlagen speichern und wieder anwenden.

**Das Serienfolgen-Muster.** Ein eigener Anwendungsfall treibt die Platzhalterauswahl: das Umbenennen von Serienfolgen nach dem Muster `Serienname - S01E01 - Titel der Folge.Format`.

Der Dialog erkennt in einem vorhandenen Dateinamen zunächst die Staffel- und Episodennummer. Ein Dateiname wie `Duldung.avi` oder `The.Expanse.S01E03.avi` wird auf das Muster `SxxEyy` geprüft. Findet er eines, werden Staffel und Episode als Vorbelegung übernommen und der restliche Name als Episodentitel angeboten. Findet er keines, fragt der Dialog nach der Staffel und der Episodennummer ab, bevor er etwas vorbelegt — und merkt sich die letzte Eingabe als Vorbelegung für die nächste Datei, damit eine Serie in einem Durchgang benannt werden kann.

Der Serienname stammt in der Regel aus dem übergeordneten Verzeichnis: Legt der Nutzer die Staffeln in einem Ordner `The Expanse` ab, ist das der Serienname. Das ist der Standard, wird aber immer angezeigt und ist überschreibbar — bei Serien, die in Sammelordner liegen, stimmt er sonst nicht.

Die Zielformatierung lautet:

```
{serie} - S{staffel:02}E{episode:02} - {titel}.{format}
```

Dabei gilt: `{staffel:02}` und `{episode:02}` sind zweistellig nullgefüllt, damit `S01E01` und nicht `S1E1` entsteht. Bei Serien mit dreistelligen Episodennummern wird die Stellenbreite automatisch auf drei angehoben, sobald eine Folge mit dreistelliger Nummer in der Auswahl liegt — sonst wären die Dateien eines Laufs nicht einheitlich benannt.

Der Episodentitel wird unverändert übernommen. Ein Doppelpunkt, ein Schrägstrich oder ein Fragezeichen im Titel sind auf Windows unzulässig und führen dort zu einem Fehler. Der Dialog weist darauf hin **bevor** die Umbenennung ausgeführt wird, und schlägt einen ersatz vor, statt stillschweigend zu ändern — der Nutzer entscheidet.

Die Vorlage „Serienfolge" ist eine benannte Vorlage wie jede andere und damit Teil des Konfigurationsdatei-Mechanismus. Sie speichert das Zielformat, die Erkennungsregel und die Frage, ob der Serienname aus dem übergeordnern Verzeichnis übernommen werden soll.

Ein Vorkonfiguriertes Set von Vorlagen liegt der Anwendung bei: „Serienfolge" (wie oben beschrieben), „Kamera" (Reihenfolge_Serienfolge_Titel, für Videokameras), „Podcast" (Datum_Titel), „Rechnung" (Jahr_Nummer_Beschreibung) und „Musik" (Interpret_Titel_Track). Jede Vorlage ist eine TOML-Datei im Konfigurationsverzeichnis und vom Nutzer anpassbar.

Vor dem Umbenennen zeigt der Dialog eine Vorschau der neuen Namen aller ausgewählten Dateien nebeneinander, mit den erkannten Feldern als eigener Spalte. So sieht der Nutzer vor dem Klick, ob die Erkennung funktioniert hat — bei Serien mit uneinheitlichen Dateinamen ist genau das der Punkt, an dem man sonst zwanzig Dateien falsch umbenennt.

### Dateibetrachter

Der Betrachter öffnet in einem zusätzlichen Bereich, ohne die Anwendung zu verlassen. Er unterstützt eine geordnete Stufung:

| Stufe | Format | Verfahren | Terminierung |
|---|---|---|---|
| 1 | Text, Quellcode | Zeichensatz-Erkennung, UTF-8/16, Zeilenenden | M0 |
| 1 | Hexdump | Offset, Hex, ASCII, ODER-Gruppe | M0 |
| 1 | Bilder | PNG, JPEG, GIF, WebP, BMP, TIFF | M0 |
| 1 | Archivinhalt | Verzeichnis des Archivs, Einzeldatei extrahieren | M0 |
| 1 | Metadaten | Größe, Typ, Zeitstempel, Prüfsumme | M0 |
| 2 | PDF | Textlayer, Seitennavigation | M1 |
| 2 | Quellcode-Hervorhebung | Syntaxhervorhebung | M1 |
| 3 | Audio, Video | Metadaten, Standbild, Wiedergabe | M3 |
| 3 | Dokumente (Office) | Nurmetadaten, keine Volltextdarstellung | M3 |

SpeedCommander nennt „80+ Formate" [1]; dieses Ziel wird bewusst nicht übernommen. Eine ehrliche Angabe der tatsächlich unterstützten Formate im About-Dialog ist besser als eine überzogene Zahl. Vollständige Office-Dokumentdarstellung ohne nativen Renderer ist im reinen Rust-Umfeld nicht erreichbar und wird als „extern öffnen" ausgelagert.

Der Betrachter ist read-only. Ein separater Editor ist eine spätere Ausbaustufe.

### Archivbehandlung

Archive erscheinen in der Dateiliste als normale Einträge. Ein Doppelklick auf ein Archiv öffnet es als *virtuelles Verzeichnis*: die enthaltenen Dateien sind auflistbar, anzeigbar und in die andere Panel-Hälfte kopierbar, ohne dass das Archiv vorher entpackt werden muss. Genau dieses Verhalten ist der Grund, warum SpeedCommander und Forklift Archive wie Ordner behandeln [2][3].

Unterstützte Formate zum Start: **ZIP**, **TAR** (mit GZip, BZip2, XZ, Zstd), **7z**. Für 7z existiert eine reine Rust-Implementierung `sevenz-rust2` [26]; für ZIP das `zip`-Crate [26]; TAR über `tar` [26]. Lesen und Schreiben sind möglich, Archive werden verschlüsselt erzeugt (AES).

Formate, die nur mit propriären oder lizenzpflichtigen Bibliotheken machbar wären — RAR (proprietär), WIM, CAB, ARJ, LZH — bleiben außen vor. Ein RAR-Leser erfordert eine Lizenz der WinRAR-Eigentümer; diese wird nicht eingeholt.

Der Explorervergleich zeigt diese Einschränkung deutlich: das Original unterstützt 13 Archivformate inklusive RAR und SQX [2], Phocommander startet mit sieben Kompressionsverfahren in drei Containerformaten.

Alle Extraktionen laufen durch eine gehärtete Pfadauflösung (siehe *Sicherheit*).

### Suche und Synchronisation

Die Dateisuche durchsucht nach Name, Größe, Datum und Inhalt. Text-, Hex- und Binärsuche sind möglich, Unicode-Text wird korrekt behandelt. Suchoperatoren UND/ODER/NICHT stehen zur Verfügung, ebenso die Suche in Archiven [2].

Die Inhaltssuche läuft in einem Hintergrundprozess mit fortschreibbarer Trefferliste; ein Treffer kann direkt in ein Panel übernommen werden.

Der Verzeichnisvergleich stellt zwei Verzeichnisse gegenüber und meldet Unterschiede in Inhalt, Datum und Größe. Die Synchronisation überträgt Unterschiede in beide Richtungen und legt Profiloptionen fest, welche Verzeichnisse einbeziehen oder ausschließen. Vor der Ausführung wird ein Plan angezeigt, nicht nur ein Dialog mit „OK".

### Favoriten, Dateiablage und Tags

Ordnerfavoriten und Anwendungsfavoriten sind über die Symbolleiste oder ein Tastenkürzel aufrufbar [2]. Die Dateiablage sammelt häufig benötigte Dateien an einem Ort, unabhängig von ihrem Fundort.

Tags an Dateien und Ordnern sind eine spätere Ausbaustufe (M3); sie sind in SpeedCommander Pro-Funktion [1].

### Eingabezeile und Werkzeuge

Eine Eingabezeile am unteren Rand nimmt Pfade und Shell-Kommandos entgegen. Sie ist der direkte Weg zu einem Verzeichnis, ohne durch die Hierarchie zu klicken, und in SpeedCommander fest verankert [4].

Eine Kommandozeilenhistorie mit `Ctrl+Down` und ein Befehlsausgabefenster gehören zum Umfang.

Die Menüleiste folgt den Plattformkonventionen: auf macOS in der Systemmenüleiste, auf Windows und Linux im Fenster. Auf macOS liegen die Standardrollen (About, Services, Hide, Quit) an den erwarteten Stellen.

---

## Nicht-funktionale Anforderungen

### Sicherheit

Die Trust Boundary verläuft zwischen dem Prozess und dem Dateisystem. Eingaben aus Dateisystem und aus Archiven gelten grundsätzlich als bösartig.

**Pfad-Traversal bei der Extraktion (Zip-Slip).** Das `zip`-Crate bietet mit `enclosed_name()` einen Schutz, der absolute Pfade und `..`-Komponenten ablehnt [25]. Diese Methode allein genügt jedoch nicht: Vor dem Zip-Slip gab es `CVE-2025-29787` (`RUSTSEC-2025-0168`), bei dem Symlinks innerhalb des Archivs den Pfad für nachfolgende Einträge aus dem Zielverzeichnis herausführten [24]. Weitere Advisories betrafen das Anlegen von Symlinks mit angreiferkontrollierten Zielen außerhalb des Zielverzeichnisses (`RUSTSEC-2026-0368`), das Setzen von Unix-Berechtigungen durch Symlinks hindurch (`RUSTSEC-2026-0369`) und das Erhalten von SUID/SGID-Bits mit möglicher Privilegienerhöhung (`RUSTSEC-2026-0367`) [21][22][23].

Daraus folgt die verbindliche Regel: Phocommander verwendet **nicht** die `extract()`-Funktionen der Crates, sondern iteriert über die Einträge und schreibt sie selbst. Jeder Zielpfad wird vor dem Schreiben kanonisiert und gegen das Zielverzeichnis geprüft; Symlinks werden grundsätzlich nicht aus Archivenmaterial erzeugt. Die Mindestversion des `zip`-Crates ist 9.0.5, die die genannten Advisorys behebt [26].

**Dekompressionsbomben.** Für die Entpackvorschau und die Extraktion gelten ein Limit für die Gesamtgröße, ein Limit für die Anzahl der Einträge und ein Limit für die Größe des einzelnen Eintrags. Wird ein Limit überschritten, bricht die Operation ab, statt den Speicher zu belasten.

**Zeitfenster zwischen Prüfung und Verwendung (TOCTOU).** Die Rust-Standardbibliothek bietet keine atomaren Dateioperationen mit Pfad-Semantik. Wo die Prüfung eines Ziels und der Zugriff darauf auseinanderfallen können, wird der Zugriff über Deskriptor-basierte Aufrufe (`openat`-Semantik über `rustix` oder `cap-std`) ausgeführt, oder die Prüfung wird unmittelbar vor dem Zugriff wiederholt. Die verbleibende Race-Bedingung wird im Bedrohungsmodell benannt, nicht verschwiegen.

**Symlinks beim Kopieren und Löschen.** Beim rekursiven Kopieren werden Symlinks als Symlinks kopiert, nicht dereferenziert — andernfalls entstehen Kopien außerhalb des Zielbaums. Beim Löschen werden sie unlinkt, nicht rekursiv gelöscht.

**Sicheres Löschen.** SpeedCommander bewirbt das Löschen „nach Regierungsrichtlinien" [2][3]. Für Phocommander gilt stattdessen das aktuelle Regelwerk: **NIST SP 800-88 Rev. 2** mit den Stufen *Clear*, *Purge* und *Destroy* [19]. Konkret:

- *Clear* (logisches Überschreiben) ist die Standardstufe für das Löschen in Phocommander und wird als „sicheres Löschen" bezeichnet.
- Auf Flash-Medien ist Überschreiben **nicht** zuverlässig, weil Wear-Leveling und Over-Provisioning die Zellen umverteilen; NIST rät für SSDs ausdrücklich von Überschreibverfahren ab [19][20].
- Der historische DoD-5220.22-M-Drei-Durchgang ist veraltet: Das US-Verteidigungsministerium hat die Überschreibspezifikation bereits 2006 aus NISPOM entfernt, weil Mehrfachdurchgänge auf modernen Medien keinen zusätzlichen Schutz bieten [19][20].
- Der Papierkorb ist der Standardpfad. „Sicheres Löschen" ist eine bewusste Zusatzoption mit der Einschränkung, dass ihre Wirkung auf Flash-Medien nicht garantiert werden kann. Diese Einschränkung steht in der Oberfläche, nicht nur im Handbuch.

**Abhängigkeiten.** Die CI prüft jede Abhängigkeit mit `cargo-deny` (Lizenz- und Advisories-Prüfung) und `cargo-audit` (bekannte Schwachstellen). Eine festgelegte Werkzeugversion wird verwendet, weil `cargo-deny` dieselbe Konfiguration je nach Version unterschiedlich liest — im Projekt Vocument hat dieser Umstand bereits zu einem falschen Fehlalarm geführt. `cargo-deny` 0.20.2 und `cargo-audit` 0.22.2 sind die aktuellen Stände [26].

**Netzwerkquellen als Angriffsfläche.** Jede entfernte Quelle ist bösartig, genauso wie ein Archiv. Der Server liefert Eintragsnamen, Größenangaben, Zeitstempel und Inhalte — alles davon ist ungeprüft zu behandeln.

*Zertifikatsprüfung:* Bei HTTPS-basierten Quellen (WebDAV, S3, OAuth) wird das Serverzertifikat streng geprüft. Kein „bei Fehler fortfahren", kein Opt-out für die Prüfung. Ein selbstsigniertes Zertifikat kann der Nutzer bewusst hinterlegen, mit einer PIN-geschützten Ausnahme für genau diesen Host — und nur für diesen.

*Antwortgrößen:* Die Antwort eines Servers darf die erwartete Größe überschreiten, ohne dass der Speicher mitwächst. Der Client liest in einem festen Budget, bricht ab und meldet den Fehler. Ein Server, der eine Verzeichnisliste mit einem Gigabyte Antwort liefert, darf den Prozess nicht in den Speicherdruck treiben.

*Protokoll-Downsizing:* Ein Server, der eine ältere Protokollversion anbietet, wird nicht stillschweigend akzeptiert. SMB1 und NTLMv1 werden abgelehnt; WebDAV-Server ohne TLS werden abgelehnt, sofern HTTPS nicht erreichbar ist.

*Eingeschleuste Verzeichnisnamen:* Ein SMB- oder WebDAV-Server kann Namen mit Trennzeichen, Steuerzeichen oder absurd langen Pfaden liefern. Namen werden normalisiert, auf unzulässige Zeichen geprüft und in der Oberfläche gekürzt dargestellt, ohne den echten Namen zu verlieren.

*Zeitlimits:* Jede Netzwerkoperation hat ein Zeitlimit. Ein Server, der eine Antwort nicht abschließt, führt nach Ablauf zu einem Abbruch mit verständlicher Meldung, nicht zu einem hängenden Auftrag.

**Rechtsschutz.** Phocommander nutzt ausschließlich Bibliotheken mit permissiver Lizenz. Crates mit Copyleft-Wirkung (GPL, AGPL) sind ausgeschlossen, solange das Projekt unter `MIT OR Apache-2.0` steht. Das schließt `mupdf` (AGPL) und die `slint`-Laufzeit im GPL-Modus aus [10][11].

### Datenschutz

Phocommander ist ein lokales Werkzeug. Es gibt keine Telemetrie, keine Analytics, keine Nutzungszählung, keine Werbung und keine Cloud-Anbindung. Es werden keine Daten an Dritte übertragen, außer der Nutzer stellt eine Netzwerkverbindung selbst her (SFTP/FTP) — und diese ist dann eine bewusste Handlung.

Es werden keine Nutzungsprofile, keine Absturzberichte und keine Inhaltsindizes gespeichert. Die Anwendung schreibt nur, was der Nutzer selbst konfiguriert: Einstellungen, Tastenbelegung, Favoriten, zuletzt geöffnete Verzeichnisse.

Verbindungsdaten für SFTP/FTP werden nicht im Klartext in der Konfigurationsdatei abgelegt, sondern im Schlüsselbund des Betriebssystems (Keychain, Credential Manager, Secret Service) abgelegt. Das `keyring`-Crate kapselt diese Zugriffe [26]. Auf Plattformen ohne Schlüsselbund (Android/iPadOS ohne Keychain-Erweiterung) fällt Phocommander auf die App-interne Sandbox zurück und weist darauf hin, dass die Daten dann nicht vom Betriebssystem geschützt sind.

Protokolldaten enthalten keine Benutzernamen, keine Servernamen und keine IP-Adressen. Ist die Protokollierung für Fehlersuche nötig, werden personenbezogene Felder maskiert.

### Leistung

Die folgenden Zielwerte sind verbindliche Gates der CI, keine Absichtserklärung. Ein Meilenstein gilt erst als erreicht, wenn die Messung vorliegt.

| Vorgabe | Zielwert |
|---|---|
| Verzeichnisliste bis 10.000 Einträge | < 100 ms bis zur ersten Darstellung |
| Verzeichniswechsel, bereits geladen | < 16 ms (ein Bild) |
| Tastendruck bis zur sichtbaren Reaktion | < 50 ms |
| Startzeit bis zur bedienbaren Oberfläche | < 800 ms (macOS, warm) |
| Speicherbedarf im Leerlauf, 2 Panels offen | < 120 MB |
| Dateikopie, sequenziell | ≥ 80 % der sequenziellen Lese-/Schreibbandbreite |
| Suchlauf über 100.000 Dateien (Namen) | < 30 s |
| Archivauflistung, 5.000 Einträge | < 500 ms |

Hintergrundoperationen sind verpflichtend asynchron. Kein Vorgang, der länger als 50 ms dauern kann, blockiert die Oberfläche. Alle Operationen melden Fortschritt mit Ratenangabe und sind abbrechbar.

### Barrierefreiheit

Bedienung vollständig per Tastatur, mit sichtbarer Fokusmarkierung und deterministischer Fokusreihenfolge. Kontrastverhältnis mindestens 4,5:1 für Text und 3:1 für Fokusrahmen. `prefers-reduced-motion` wird respektiert. Screenreader-Unterstützung über AccessKit, das in egui und Slint standardmäßig aktiviert ist [11].

### Codequalität und Einsteigerfreundlichkeit

Der „Noob-Stil" ist eine verbindliche Projektkonvention, kein Geschmack: Jede öffentliche Funktion trägt einen Doc-Kommentar nach dem Muster **WAS / WARUM / BEISPIEL**. Jedes Modul beginnt mit einem Kopfkommentar, der seine Verantwortung und seine Grenze benennt. Jede Konstante hat einen Namen, der seinen Zweck trägt — keine Magic Numbers. Jede öffentliche Funktion hat mindestens einen Unit-Test.

Die überarbeitungsresistente Grundregel: Was ein Anfänger nicht nachvollziehen kann, wird umgeschrieben, nicht weggelassen. Tiefe Verschachtelung wird durch early returns vermieden. Der Code ist auf Englisch, die Dokumentation zweisprachig (DE/EN), die Oberflächentexte internationalisierbar mit Deutsch als Vorgabe.

Verbindliche Grenzwerte, aus dem Vocument-Vorbild übernommen und hier bestätigt: keine Quelldatei über 400 Zeilen (weich) beziehungsweise 550 Zeilen (hart); `unsafe_code`, `unwrap_used` und `expect_used` sind im Workspace verboten; `missing_docs` ist Warnstufe und wird für `pub`-API zur Fehlerstufe in den Kerncrates.

### Internationalisierung

Alle Oberflächentexte laufen durch einen Übersetzungsschlüssel, keine Zeichenketten stehen hart im Code. Deutsch und Englisch sind vollständig abgedeckt. Datums-, Zahlen- und Größenformate folgen der Systemsprache.

---

## Systemarchitektur

### Crate-Struktur

Der Workspace folgt dem im Photocraft-Projekt etablierten Muster — flache `crates/*`-Kisten, `apps/*` für die Binärdateien, `xtask` für Build-Hilfen [28]. Das hält Werkzeuge, Reviewer und Mitwirkende zwischen den Projekten konsistent.

```
phocommander/
├── Cargo.toml                 # Workspace, gemeinsame Dependency-Versionen
├── rust-toolchain.toml        # gepinnte Toolchain
├── deny.toml                  # cargo-deny-Konfiguration (Lizenzen + Advisories)
├── config/
│   ├── defaults.toml          # zentrale Konfigurationsdatei (Vorlage)
│   └── keymap.toml            # Tastenbelegung (Vorlage)
├── crates/
│   ├── phocommander-core/     # Dateisystemmodell, Pfade, Symbole, Fehler
│   ├── phocommander-config/   # Konfigurationsparser, Schema-Versionierung
│   ├── phocommander-keymap/   # Aktionskatalog, Auflösung, Bedienkontexte
│   ├── phocommander-ops/      # Kopieren, Verschieben, Löschen, Auftragswarteschlange
│   ├── phocommander-archive/  # Archivadapter, gehärtete Extraktion
│   ├── phocommander-viewer/   # Betrachterregistry und Formatadapter
│   ├── phocommander-remote/   # Netzwerkquellen: SFTP, FTP, SMB, WebDAV, S3
│   └── phocommander-ui/       # egui-Oberfläche, Panels, Theme
├── platform/
│   └── phocommander-platform/ # OS-Adapter: Menü, Dialoge, Papierkorb, Berechtigungen
├── apps/
│   ├── phocommander-desktop/  # macOS, Windows, Linux, BSD
│   ├── phocommander-android/  # Gradle-Wrapper, Rust-Bibliothekscrate
│   └── phocommander-ios/      # Xcode-Projekt, Rust-Bibliothekscrate
├── scripts/                   # Prüfskripte (Crate-Grenzen, Dateigrößen, Links)
├── config/                    # Vorgaben: defaults.toml, keymap.toml, Vorlagen/
└── docs/
    ├── REQUIREMENTS.md        # dieses Dokument
    ├── ARCHITECTURE.md
    ├── adr/                   # Architecture Decision Records
    ├── user-guide/            # DE + EN
    ├── security/THREAT_MODEL.md
    └── glossary/
```

**Abhängigkeitsrichtung.** `apps/*` → `platform/*` → `crates/*`. Innerhalb von `crates/*` darf keine Kiste in eine andere Kiste der gleichen Ebene ableiten. Diese Regel wird maschinell geprüft, nicht per Konvention: ein Prüfskript liest die `Cargo.toml`-Dateien und schlägt bei Verstoß mit Exit-Code 1 fehl. Ein Negativtest (Verstoß bauen → Gate rot → zurückbauen → Gate grün) belegt, dass das Gate wirkt.

Der Kern ist frei von UI-Abhängigkeiten. `phocommander-core` kennt keine Fenster, keine Crates des UI-Stack und keine plattformspezifischen APIs — dadurch ist er auf jeder Zielplattform testbar und der einzige Weg, die Testabdeckung überhaupt sinnvoll zu messen.

### Benutzerschnittstelle und Plattformanpassung

**Framework-Wahl: egui/eframe 0.37.** Die Entscheidung ist bewusst gegen vier Alternativen abgewogen:

*egui* liegt unter `MIT OR Apache-2.0` [7] und deckt alle sechs Zielplattformen ab — Windows, macOS und Linux stabil, Android über `android-activity`, iOS über `objc2`/UIKit, mit sicherer-area-Unterstützung [32][33]. Das Projekt Photocraft verwendet bereits dieselbe Framework-Generation, dieselbe `MIT OR Apache-2.0`-Kombination und Rust 2024 [28]; das Muster für ein natives Menü über ein eigenes Backend-Trait ist dort bereits erprobt und übertragbar [29].

*Slint* bietet native Menüs, eine eigene Markup-Sprache und gute Mobiltauglichkeit, steht aber unter `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` [8]. Die royalty-free-Lizenz ist kostenlos, verlangt aber einen sichtbaren Slint-Nachweis und untersagt eingebettete Nutzung; der GPL-Modus würde das gesamte Produkt auf GPL ziehen [9][10]. Für ein Projekt unter `MIT OR Apache-2.0` ist das eine dauerhafte Lizenzhürde.

*iced* ist reines Rust mit Elm-Architektur, hat aber weiterhin keine Mobilunterstützung — das Ticket ist seit April 2020 offen [12]. Es scheidet für Android und iPadOS aus.

*Tauri 2* unterstützt Mobil seit Oktober 2024 stabil [34][37], bringt aber eine Web-Frontend-Schicht mit HTML, CSS und JavaScript mit. Das verletzt die Vorgabe „in Rust" und erzeugt plattformabhängige Darstellungsunterschiede durch die unterschiedlichen Webview-Engines (WebKit, WebView2, WebKitGTK) [11][37].

*Natives UI je Plattform* (AppKit, UIKit, Win32, GTK) ergäbe echte Systemoptik, vervielfacht aber den Pflegeaufwand um den Faktor sechs und verlangt je Plattform eine eigene Sprache oder ein eigenes FFI-Team.

**Definition der Plattformanpassung.** egui zeichnet seine Widgets selbst; ein Look-and-Feel wie Forklifts echtes AppKit ist damit nicht erreichbar [11]. Der Ausgleich besteht aus vier Achsen, die gemeinsam das Plattformgefühl tragen:

- *Menü*: echtes natives Menü über `muda` [27] — auf macOS in der Systemmenüleiste, auf Windows als `HMENU` im Fenster, auf Linux und BSD über GTK. `muda` unterstützt Windows, macOS und Linux/BSD mit GTK 3 oder 4 [27]. Photocraft hat dafür bereits eine plattformfreie `Backend`-Abstraktion mit Sync-Schnittstelle [29].
- *Tastatur*: Plattformspezifische Modifikatoren. `Cmd` auf macOS und iPadOS, `Ctrl` auf Windows, Linux und BSD. Die *Aktionen* bleiben identisch, nur die Tastenbelegung variiert.
- *Dialoge*: native Dateidialoge über `rfd` [26], damit Erlaubnis-Dialoge und Dateityp-Filter wie im System aussehen und funktionieren.
- *Metrik und Theme*: plattformspezifische Zeilenhöhen, Schriftgrößen, Fokusrahmen, Farbpaletten (hell/dunkel) und Sidebar-Tiefe, damit die Oberfläche nicht wie eine Webanwendung wirkt.

Diese Kombination liefert ein dichtes, tastaturorientiertes Werkzeug mit systemspezifischen Konventionen. Sie liefert nicht die Pixelgenauigkeit eines nativen AppKit-Programms. Diese Einschränkung wird in der Spezifikation offen benannt, weil sie eine der offenen Designentscheidungen des Projekts ist (siehe *Offene Entscheidungen*, Frage E1).

**Modellierung der Oberfläche.** egui ist ein Immediate-Mode-Framework: der Widget-Baum wird in jedem Bild neu beschrieben. Für eine Dateiliste mit 100.000 Einträgen ist das zu teuer. Die UI-Schicht hält deshalb den Verzeichnisinhalt in einem eigenen Zustandsobjekt außerhalb des egui-Kontexts und zeichnet nur den sichtbaren Ausschnitt. Zeichnen und Zustand sind getrennt; das entspricht dem Muster, das im Photocraft-Projekt für große Ansichten etabliert ist.

### Tastatursteuerung

Die Belegung folgt der Norton-Commander-Tradition, die sich als Quasi-Standard etabliert hat [4][36]:

| Taste | Aktion | Taste | Aktion |
|---|---|---|---|
| `F2` | Umbenennen (auch Mehrfach) | `Ctrl+R` | Panel neu laden |
| `F3` | Betrachter öffnen | `Ctrl+U` | Panels tauschen |
| `F4` | Editor (später) | `Ctrl+Enter` | in Gegenpanel öffnen |
| `F5` | Kopieren | `Ctrl+1`–`Ctrl+9` | Registerkarte |
| `F6` | Verschieben | `Ctrl+Tab` | nächste Registerkarte |
| `F7` | Verzeichnis anlegen | `Ins` | Auswahl umschalten |
| `F8` | Löschen | `Ctrl+A` | alle auswählen |
| `F9` | Menü | `Ctrl+Num-` | Auswahl aufheben |
| `F10` | Beenden | `Ctrl+Num*` | Auswahl invertieren |
| `Tab` | Panel wechseln | `Leertaste` | Auswahl per Muster |
| `Alt+F1`/`Alt+F2` | Laufwerke | `Ctrl+PgUp` | übergeordnetes Verzeichnis |

Die Zuordnung ist konfigurierbar. Die Konfigurationsdatei enthält eine Sektion, in der jede Aktion an beliebige Tasten gebunden werden kann; die Voreinstellung pro Betriebssystem liegt als eigene Vorlage bei. `F3` im Betrachter sucht weiter, `Shift+F3` sucht rückwärts — durchgängig, auch in den Einzelfenstern des Mehrfach-Umbenennungsdialogs, wo `F3` und `Shift+F3` den Konfliktsprung steuern [2].

### Konfigurationsdatei

Es gibt genau eine zentrale Konfigurationsdatei im TOML-Format, unterstützt durch `toml` und `serde` [26]. Ihr Speicherort folgt den Betriebssystemkonventionen: `~/Library/Application Support/Phocommander/` auf macOS, `%APPDATA%\Phocommander\` auf Windows, `$XDG_CONFIG_HOME/phocommander/` auf Linux und BSD, App-Sandbox auf Android und iPadOS.

Aufbau:

```toml
# Schema-Version: wird bei inkompatiblen Änderungen erhöht.
# Bei older Version wird automatisch migriert; die Originaldatei
# bleibt als .bak erhalten.
schema_version = 1

[general]
# Sprache der Oberfläche: "de", "en" oder "system"
language = "system"
# Verhalten bei dem Start: zuletzt geöffnete Verzeichnisse wiederherstellen
restore_session = true

[appearance]
theme = "system"          # "system" | "light" | "dark"
row_height = 0            # 0 = plattformabhängiger Standard
show_hidden_files = false
show_system_files = false
file_size_format = "binary"   # "binary" | "decimal" | "iec"

[panels]
# Standardanordnung der beiden Fenster
layout = "horizontal"     # "horizontal" | "vertical"
# Standardverhalten bei einem Rechtsklick: Kontextmenü oder Auswahl umschalten
right_click_action = "context_menu"
# Überschreiben beim Kopieren niemals automatisch
overwrite_policy = "ask"  # "ask" | "never" | "always" | "keep_both"

[keymap]
# Überschreibt einzelne Zuordnungen der Plattformvorgabe.
# "primary" ist Cmd auf macOS/iPadOS, Ctrl unter Windows/Linux/BSD.
"panel.copy" = "F5"
"panel.move" = "F6"

[viewer]
# Maximale Dateigröße, die der Betrachter in den Speicher lädt
max_file_size = 10485760
# Textdateien mit mehr als N Bytes werden nicht geladen
max_text_size = 4194304

[archives]
# Obergrenze für entpackte Daten (Schutz vor Dekompressionsbomben)
max_extracted_bytes = 2147483648
max_entries = 100000

[privacy]
# Protokollebene. Phocommander protokolliert grundsätzlich nicht
# auf Benutzer- oder Rechnernamen.
log_level = "warn"

[advanced]
# Anzahl der Worker-Threads für Hintergrundoperationen.
# 0 = automatisch (Anzahl der verfügbaren Kerne)
worker_threads = 0

[input]
# Welcher Bedienkontext aktiv ist. "auto" erkennt das Eingabegerät
# zur Laufzeit: Tastatur oder Maus -> keyboard/pointer, Touch -> touch,
# Fernbedienung -> dpad, Controller -> controller.
context = "auto"
# Tastenbelegung je Kontext. Jede Aktion hat eine eigene Vorgabe,
# damit dieselbe Anwendung auf jedem Gerät richtig bedient wird.
[input.keyboard]
"panel.copy" = "F5"
"panel.move" = "F6"
[input.dpad]
# Auf der Fernbedienung ersetzt die Aktionspalette die F-Tastenreihe.
# "panel.copy" = "palette:file.copy"
"panel.copy" = "palette:file.copy"
"panel.move" = "palette:file.move"
"palette.open" = "Menu"
[input.controller]
# Trigger = Aktion ausloesen, Griff = Auswahl umschalten.
"panel.copy" = "palette:file.copy"
"selection.toggle" = "Grip"

[remote]
# Verhalten bei Verbindungsproblemen: "retry" versucht es erneut,
# "fail" bricht ab. Timeouts gelten pro Verbindungsaufbau.
connect_timeout_seconds = 15
# SMB: Signatur verlangen, wenn der Server sie unterstuetzt.
# "require" lehnt Server ohne Signatur ab, "prefer" verhandelt.
smb_signing = "prefer"
# SMB: Transportverschluesselung (SMB 3) bevorzugen.
smb_encryption = "prefer"
# Groesster erlaubter Download einzelner Dateien aus entfernten Quellen.
# 0 = unbegrenzt (nur bei bewusst vertrauenswürdigen Quellen).
max_remote_download_bytes = 0
```

Die Datei ist vollständig kommentiert, weil sie die erste Anlaufstelle für Nutzer und Mitwirkende ist. Sie enthält ein `schema_version`-Feld; bei inkompatiblen Änderungen migriert Phocommander die alte Datei und legt das Original als `.bak` daneben. Import und Export einer Konfiguration sind vorgesehen, damit Einstellungen zwischen Rechnern übertragen werden können.

### Auftragsverarbeitung

Alle verlangsamen Vorgänge laufen über eine Auftragswarteschlange. Ein Auftrag hat: Typ, Status, Fortschritt, Ratenangabe, Abbruchflagge, Fehlerliste. Die Warteschlange ist threadsicher; die UI liest einen Snapshot.

Mehrere Aufträge laufen parallel bis zu einem konfigurierbaren Limit, standardmäßig der Zahl der verfügbaren Kerne. Eine Panel-Hälfte blockiert nicht, während in der anderen gearbeitet wird. SpeedCommander bietet hier priorisierbare Hintergrund-Archivoperationen [2]; die Priorisierung ist für Phocommander eine geplante Ausbaustufe.

### Viewer- und Archivregistrierung

Betrachter und Archivadapter werden über ein Trait registriert. Ein neuer Adapter ist eine neue Implementierung plus eine Registrierungszeile — ohne Änderung am Kern. Damit ist der Funktionsumfang erweiterbar, ohne dass die Codebasis an einem Ort zusammenwächst.

---

## Plattform-Matrix

Die Zugriffsrechte unterscheiden sich je Plattform grundlegend. Das ist der größte Unterschied zwischen den sechs Zielsystemen und bestimmt den Zuschnitt der jeweiligen Version.

| Plattform | Zugriff auf das Dateisystem | Konsequenz für Phocommander |
|---|---|---|
| macOS | App-Sandbox erlaubt nur ausgewählte Dateien; „Full Disk Access" muss der Nutzer manuell in den Systemeinstellungen erteilen [35][15] | Dateimanager ohne weitreichende Rechte ist unbrauchbar. Ohne Sandbox, mit Notarisierung, mit einer Anleitung zur Freigabe |
| iPadOS | Vollständig sandboxed. Zugriff nur über `UIDocumentPickerViewController` und security-scoped URLs [13][14] | Kein Dateisystemdurchsuchen möglich. „Verzeichnisse" sind nur vom Nutzer ausgewählte Wurzeln |
| Android | Scoped Storage ab API 30; `MANAGE_EXTERNAL_STORAGE` ist auf File-Manager-Funktion beschränkt und braucht eine Deklaration im Play Console [16][17][18] | Ohne die Spezialberechtigung funktioniert nur der SAF-Zugriff. Als Dateimanager ist die Berechtigung fachlich zulässig [18] |
| Windows | Keine Sandbox. Pfadlänge 260 Zeichen ohne Opt-in, bis 32.767 mit `longPathAware` im Manifest [30][31] | Manifesteintrag `longPathAware` wird gesetzt. Fehlermeldungen bei Pfadlänge werden verständlich formuliert |
| Linux | Keine Sandbox. X11 und Wayland | Standardpfad über `winit` [6]. Wayland-Eigenheiten beachten |
| BSD | Keine Sandbox. FreeBSD und NetBSD sind bei `winit` Tier 2 (Bau garantiert, getestet nicht) [6] | Bau auf FreeBSD und NetBSD wird in der CI geprüft, Funktionstests erfolgen manuell |
| Fire TV, Google TV | Android mit Leanback. Kein Touchscreen, D-Pad als primäre Navigation [40] | Eigener `dpad`-Bedienkontext, Aktionspalette, 10-Fuß-Layout, ein Panel sichtbar |
| Meta Quest | Horizon OS ist Android [44]. Flat Panel, Gesten werden auf Controller abgebildet [45] | Kein eigener Port, sondern `controller`-Kontext auf der Android-Version |

Die Matrix zeigt die eigentliche Architekturentscheidung: Phocommander braucht eine Dateisystemabstraktion, die einen „Ordnerinhalt" liefert, ohne zu wissen, ob er aus dem lokalen Dateisystem, aus einem Archiv, aus einem über SFTP erreichten Verzeichnis oder aus einem vom Nutzer freigegebenen Wurzelverzeichnis auf iPadOS stammt. Ohne diese Abstraktion entstehen für jede Quelle eigene Sonderfälle, die den Kern auseinanderziehen.

### macOS

Die macOS-Version ist die erste Auslieferung und die Referenzimplementierung der Oberfläche.

Der Dateimanager braucht systemweiten Zugriff, den die App-Sandbox nicht hergibt — sie erlaubt nur Dateien, die der Nutzer ausgewählt hat [35]. Ein Sandboxed Dateimanager wäre ein Dateidialog, kein Dateimanager. Phocommander wird daher **ohne** App-Sandbox ausgeliefert, mit Notarisierung, Hardened Runtime und einer deutlichen Erststart-Seite, die die nötigen Berechtigungen erklärt. Der Nutzer entscheidet, ob er den Vollzugriff erteilt.

Nutzung von „Full Disk Access" wird **nicht** erzwungen und nicht umgangen: Apple lässt sich den Zugriff nicht per Entitlement einräumen, der Nutzer muss ihn in den Systemeinstellungen erteilen, und die App muss mit der Ablehnung umgehen können [35].

### iPadOS

iPadOS ist eine eingeschränkte Plattform, und jede Zusage muss daran gemessen werden. Der Prozess ist sandboxed; Zugriff auf Dateien außerhalb des Containers gibt es nur über den Dokumenten-Picker, der security-scoped URLs mit Rückgaberechten liefert [13][14]. Verzeichnisse können ab iOS 13 aus jedem verfügbaren File-Provider ausgewählt werden, und die Berechtigung gilt rekursiv für den Inhalt [13].

Daraus folgt der Zuschnitt: Die iPadOS-Version ist **kein** Dateisystem-Browser. Sie ist ein Betrachter und Organisator für Wurzelverzeichnisse, die der Nutzer ausdrücklich freigibt. Eine Linkstruktur zur dauerhaften Nutzung wird über security-scoped Bookmarks gespeichert; der Nutzer kann den Zugriff jederzeit in den Einstellungen entziehen, und die App muss den Fehler sauber behandeln [13].

Die Zwei-Fenster-Oberfläche bleibt auf dem iPad sinnvoll, allerdings in geteilter Ansicht statt als Vollbild-Fensterpaar. Touch bedient die Auswahl, Tastatur bleibt die Hauptsteuerung.

### Android

Android unterstützt Scoped Storage seit API 30; Apps mit Ziel-SDK 30 oder höher müssen damit rechnen [17]. Das Storage Access Framework ist der empfohlene und datenschutzfreundlichere Weg [18]. Für einen Dateimanager ist `MANAGE_EXTERNAL_STORAGE` fachlich zulässig — Google nennt File-Management ausdrücklich als erlaubten Anwendungsfall —, erfordert aber eine Erklärung der Kernfunktionalität, eine Erklärung im Manifest, die Prüfung durch die Play-Zentrale und eine klare Aufforderung an den Nutzer, die Berechtigung zu aktivieren [16][18].

Strategie: Die App funktioniert im SAF-Modus ohne die Spezialberechtigung und bietet die Vollzugriffsfunktion als klar gekennzeichnete, optionale Erweiterung an. Ohne Vollzugriff arbeitet sie im Dokumenten-Browser-Modus.

Die FUSE-Schicht in Android 11 verlangsamt Massenoperationen, bei voller Berechtigung greifen die Optimierungen des MediaProvider [17]. Ein eigener Kopiervorgang mit direkten Dateizugriffen ist deshalb vorzuziehen.

### Windows, Linux und BSD

Windows bringt die Pfadlängenfrage mit: 260 Zeichen im Standard, 32.767 mit aktiviertem `LongPathsEnabled` und `longPathAware` im Anwendungsmanifest [30][31]. Das Manifest wird gesetzt. Da der Explorer die Langpfad-Unterstützung nicht nutzt [31], kann Phocommander einen Zustand erreichen, in dem der Explorer die Datei nicht anzeigt, Phocommander aber schon — dieser Fall wird abgefangen und verständlich gemeldet.

Linux und BSD teilen sich den Codepfad. FreeBSD und NetBSD sind bei `winit` Tier 2: der Bau ist garantiert, tiefergehende Tests finden nicht statt [6]. Der CI prüft den Bau auf beiden, Funktionstests laufen manuell. Wayland wird bevorzugt, X11 als Rückfallebene.

---

## Test- und Qualitätsstrategie

### Testpyramide

**Unit-Tests** liegen bei jeder öffentlichen Funktion — eine verbindliche Regel aus dem Vocument-Vorbild. Sie decken die reine Logik ab: Pfadauflösung, Tastenauflösung, Umbenennungsregeln, Sortierung, Größenformatierung.

**Integrationstests** decken die Kistengrenzen ab: Kopiervorgang über ein temporäres Verzeichnis, Extraktion eines Archivs mit Trail-Einträgen, Laden einer Konfigurationsdatei mit unbekanntem Feld.

**Property-Tests** mit `proptest` [26] prüfen Invarianten, die man nicht aufzählen kann: dass die Pfadauflösung nie aus dem Zielverzeichnis herausführt, dass die Umbenennungsregel für beliebige Eingaben deterministisch ist, dass die Tastenauflösung bei Konflikten deterministisch entscheidet.

**Fuzz-Tests** prüfen die Parser: Archivheader, Bildheader, Zeichensatz-Erkennung. Ein Fuzz-Lauf gehört in die nächtliche CI.

**UI-Tests** über `egui_kittest` prüfen die Tastaturbedienung deterministisch, ohne pixelweise Screenshots.

**Leistungsmessungen** mit `criterion` [26] sind Teil des Gates; die Werte aus der Leistungsanforderung sind die Schwellen.

### Continuous Integration

Die CI läuft als Matrix über Linux, Windows und macOS. Zusätzlich prüft sie Quer-Kompilierung nach `aarch64-linux-android` und `aarch64-apple-ios`, sowie den Bau für FreeBSD. Jeder Lauf umfasst: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`, `cargo-deny`, `cargo-audit` und `cargo build --release`.

Die Werkzeugversionen werden gepinnt, weil sich dieselbe Konfiguration je nach Version unterschiedlich liest — `cargo-deny` interpretiert eine Allow-Liste je nach Version als `AGPL-3.0` oder `AGPL-3.0-or-later` und fällt sonst mit einer Konfigurationsfehlermeldung aus. Im Vocument-Projekt hat das dazu geführt, dass die Lizenzprüfung in der CI nie gelaufen ist und die Meldung stattdessen die Konfiguration beschuldigte [26].

Der Gate darf nicht grün sein, wenn eine Regel verletzt ist. Für jede Härtung — Pfad-Traversal, Dekompressionsbombe, Header-Limit — existiert ein negativer Test, der den Angriff tatsächlich ausführt und die Abwehr belegt.

### Abnahmekriterien

Jede Meilensteinabnahme folgt dem Vocument-Muster: Funktionalität, Technik, Leistung, Benutzerführung, Sicherheit und Datenschutz, Interoperabilität, bekannte Einschränkungen, Entscheidung. Jede Aussage braucht eine konkrete Evidenz — ein Testname, eine Kommandoausgabe. Ein Meilenstein gilt nur als erreicht, wenn Evidenz existiert.

---

## Repository- und Dokumentationsorganisation

### Ablagestruktur

Die oben dargestellte Crate-Struktur ist zugleich die Repository-Struktur. Der Workspace ist eine einzige Cargo-Workspace mit gemeinsamen Dependency-Versionen an einer Stelle, Versionierung über `rust-toolchain.toml` gepinnt.

Das Muster ist bewusst identisch zu Photocraft [28]: flache `crates/*`-Kisten, `apps/*` für die Binärdateien, ein `xtask`-Crate für Build-Hilfen. Das senkt die Einstiegshürde für Mitwirkende, die bereits in den Schwesterprojekten arbeiten.

### Dokumentationslandschaft

```
docs/
├── REQUIREMENTS.md          # dieses Dokument
├── ARCHITECTURE.md          # Crate-Grenzen, Datenflüsse, Erweiterungspunkte
├── WHERE_TO_CHANGE.md       # "Wo ändere ich was?" für Mitwirkende
├── adr/                     # Architecture Decision Records, nummeriert
├── user-guide/              # DE + EN, nach Aufgaben gegliedert
├── security/
│   ├── THREAT_MODEL.md
│   └── HARDENING.md
├── glossary.md
├── acceptance/              # Abnahmenachweise je Meilenstein (M<n>-ACCEPTANCE.md)
├── reviews/                 # Reviews und Prüfprotokolle
├── guides/                  # Anleitungen (Installation, Netzwerk, Bedienkontexte)
└── templates/               # Vorlagen für Abnahmen und Reviews
```

ADR folgen dem Muster Status, Datum, Kontext, Entscheidung, Konsequenzen, Alternativen. Die erste Serie umfasst die Framework-Wahl, die Crate-Grenzen, die Fehlerhierarchie und die Sicherheitsstrategie für Archivextraktion.

`AGENTS.md` beschreibt die Arbeitsregeln für KI-gestützte Beiträge: erst lesen, dann ändern; keine Behauptung ohne Messung; keine Stubs; Übergabe über Patch und `deliverable.md`.

### Beitragsregeln und Governance

`CONTRIBUTING.md` fasst die Regeln zusammen, die in den Schwesterprojekten bereits erprobt sind: keine Quelldatei über 400 Zeilen, Doc-Kommentar im WAS/WARUM/BEISPIEL-Muster, Unit-Test für jede öffentliche Funktion, kein Commit von Abhängigkeiten ohne Prüfung. `SECURITY.md` beschreibt das Bedrohungsmodell und den Meldeweg für Schwachstellen. `PRIVACY.md` hält fest, welche Daten gespeichert werden, wo, wie lange und wie sie entfernt werden — mit der ausdrücklichen Zusage null Telemetrie.

`CODE_OF_CONDUCT.md` folgt dem Contributor Covenant.

### Lizenzierung

Das Repository steht unter der Doppellizenz `MIT OR Apache-2.0`. Das bedeutet: Jeder Nutzer wählt zwischen MIT und Apache-2.0. MIT ist kurz und permissiv, Apache-2.0 enthält zusätzlich eine ausdrückliche Patentklausel — für ein Projekt mit möglicher späterer Patentanmeldung ist Apache-2.0 die bessere Wahl. Beide Dateien liegen im Repository-Root, `LICENSE-MIT` und `LICENSE-APACHE`. In jeder Quelldatei steht ein kurzer SPDX-Header.

Diese Doppellizenz entspricht der Konvention des Photocraft-Projekts [28] und der Rust-Community. Die `cargo-deny`-Konfiguration erlaubt ausschließlich permissive Lizenzen; Copyleft fällt durch.

---

## Roadmap

### Phasen M0 bis M9

Die Reihenfolge folgt der Priorität des Nutzers (macOS, Android, iPadOS, Windows, Linux, BSD) mit einer technischen Korrektur: **Windows, Linux und BSD teilen sich den vollständigen Desktop-Codepfad mit macOS.** Sie sind nach macOS nahezu kostenlos, während Android und iPadOS jeweils eigene Verpackung, eigene Berechtigungsmodelle und eigene Store-Anforderungen mitbringen [16][13]. Die teuersten Ziele sind damit nicht die letzten, sondern die mobilen.

| Phase | Inhalt | Ergebnis |
|---|---|---|
| **M0** | Workspace, CI, Konfigurationsdatei, Kerncrate, Tastenauflösung, Panel-Grundgerüst | Zwei Fenster, navigieren, auswählen. Bedienbar auf macOS |
| **M1** | Dateioperationen mit Auftragswarteschlange, Mehrfach-Umbenennen, Betrachter Stufe 1, Archiv Stufe 1 (ZIP/TAR/7z), sichere Extraktion | Funktionaler Dateibrowser auf macOS |
| **M2** | Panel-Härtung, Registerkarten, Filterfeld, Eingabezeile, Verzeichnisvergleich, Favoriten, Betrachter Stufe 2 (PDF, Syntaxhervorhebung), Finder-Integration | Reife macOS-Version |
| **M3** | Windows- und Linux-/BSD-Ports, Paketierung, Long-Path-Behandlung, Wayland | Drei Desktop-Plattformen |
| **M4** | Android: Gradle-Projekt, Rust-`cdylib`, SAF-Modus, optionaler Vollzugriff, Touch- und Tastatur-Bedienung | Android-Version |
| **M5** | iPadOS: Xcode-Projekt, Dokumenten-Picker, security-scoped Bookmarks, geteilte Ansicht | iPadOS-Version |
| **M6** | Netzwerkquellen: SFTP, FTP, SMB, WebDAV; Verbindungsverwaltung; Synchronisation; Verzeichnisvergleich produktiv; Suche mit Inhalt | Netzwerkfunktionen |
| **M7** | Cloud über OAuth 2.0 und S3-kompatible Ziele; Bedienkontext `controller` auf der Android-Version | Cloud- und Controller-Zugriff |
| **M8** | Fire TV und Google TV: Bedienkontext `dpad`, Aktionspalette, 10-Fuß-Layout, Leanback-Manifest, Store-Abnahme | Fernseher-Version |
| **M9** | Betrachter Stufe 3, Editor, Tags, Makros | Erweiterungen |

Meilenstein M0 und M1 sind der Kern. Ab M2 ist jede weitere Plattform eine Auslieferungsvariante desselben Kerns.

Die Netzwerkmeilensteine folgen bewusst **nach** den Desktop- und Mobilplattformen: Eine Quelle, die nicht funktioniert, ist reparierbar und sichtbar. Eine halbfertige Netzwerkschicht, die in M1 dazwischenhängt, gefährdet den Kern. Umgekehrt ist die Aktionpalette in M8 die Voraussetzung dafür, dass Fire TV die Anforderungen von Google Play besteht [41] — sie kommt nicht als Nachlieferung, sondern als eigener Meilenstein.


---

## Projektplan

Der Projektplan übersetzt die Roadmap in eine Reihenfolge, in der gearbeitet wird. Er ist verbindlich: Ein Meilenstein gilt erst als erreicht, wenn seine Abnahmekriterien erfüllt sind **und** die zugehörige Abnahme unter `docs/acceptance/` dokumentiert ist.

### Grundregeln des Ablaufs

**Der Kern kommt vor der Oberfläche.** Jeder Meilenstein liefert zuerst eine funktionierende, testbare Schicht im Rust-Kern. Die Oberfläche bindet sie an. Eine Oberflächenfunktion ohne Kern-Unterstützung wird nicht begonnen.

**Jeder Meilenstein endet mit einem gepushten Stand auf `main`.** Der Fortschritt ist damit jederzeit sichtbar und nachvollziehbar. Zwischenstände werden nicht gesammelt, sondern fortlaufend weitergegeben.

**Kein Meilenstein gilt als erreicht ohne Beleg.** Ein Meilenstein ohne grünen Testlauf und ohne Abnahmedokument ist nicht erreicht, egal wie überzeugend die Beschreibung klingt.

**Rückfragen werden gestellt, nicht geraten.** Wenn eine Entscheidung fehlt, wird sie beim Owner angefragt und die Antwort im jeweiligen Dokument festgehalten.

### Meilensteine

Die folgende Übersicht zeigt, welcher Meilenstein was liefert und worauf er aufbaut.

| Meilenstein | Inhalt | Ergebnis | Abhängigkeit |
|---|---|---|---|
| **M0** | Cargo-Workspace, Crate-Grundgerüst, CI-Grundlage, Konfigurationsdatei mit Vorgaben, Tastenbelegung, Bedrohungsmodell, Projektdokumentation, Lizenzprüfung | Das Repository baut, prüft und testet sich auf drei Betriebssystemen. Die Spezifikation ist die verbindliche Grundlage jedes weiteren Schritts | keine |
| **M1** | Dateisystemmodell, Auftragswarteschlange, Panel-Zustand, Tastenauflösung, Dateioperationen, Mehrfach-Umbenennen mit Vorlagen, Betrachter Stufe 1, Archivadapter Stufe 1, gehärtete Extraktion | Ein funktionaler Zwei-Fenster-Dateibrowser auf macOS. Kopieren, verschieben, umbenennen, ansehen, Archive öffnen | M0 |
| **M2** | Registerkarten, Filterfeld, Eingabezeile, Verzeichnisbaum, Verzeichnisvergleich, Favoriten, Betrachter Stufe 2, native Menüs, Kontextmenüs | Eine reife macOS-Version, die im Alltag einen Dateibrowser ersetzen kann | M1 |
| **M3** | Windows-, Linux- und BSD-Ports, Paketierung, Long-Path-Behandlung, Wayland, X11-Fallback, NetBSD- und FreeBSD-Bau | Drei weitere Desktop-Plattformen aus derselben Codebasis | M2 |
| **M4** | Android-Shell, Gradle-Projekt, Rust-`cdylib`, SAF-Modus, optionaler Vollzugriff, Touch-Kontext, adaptive Layouts | Eine Android-Version, die im Store veröffentlicht werden kann | M3 |
| **M5** | iPadOS-Shell, Xcode-Projekt, Dokumenten-Picker, security-scoped Bookmarks, geteilte Ansicht, Apple-Tastatur-Unterstützung | Eine iPadOS-Version für den App Store | M4 |
| **M6** | Netzwerkquellen: SFTP, FTP, SMB, WebDAV; Verbindungsverwaltung; Schlüsselbund-Integration; Angriffsflächen-Härtung; Synchronisation | Entfernte Quellen sind bedienbar wie lokale Verzeichnisse | M5 |
| **M7** | Cloud über OAuth 2.0 und S3-kompatible Ziele, Bedienkontext `controller` auf der Android-Version, Controller-Anbindung auf Desktop | Cloud-Zugriff ohne eigenen Server, Controller-Bedienung | M6 |
| **M8** | Bedienkontext `dpad`, Aktionspalette, 10-Fuß-Layout, Leanback-Manifest, Store-Abnahme, Prüfprotokoll der D-Pad-Erreichbarkeit | Eine Fire-TV- und Google-TV-Version, die die Google-Play-Prüfung besteht | M7 |
| **M9** | Betrachter Stufe 3, Texteditor, Tags, Makros | Die Erweiterungen des Funktionsumfangs | M8 |

Die Reihenfolge ist linear, weil jede Stufe auf der vorherigen aufsetzt. Ausnahmen sind nur dort vorgesehen, wo ein Teilbereich unabhängig ist: Die Dokumentation, das Bedrohungsmodell und die Lizenzprüfung laufen ab M0 parallel mit dem Code und werden in jedem Meilenstein nachgeführt.

### Risikoregister

Die acht folgenden Risiken sind die bekannten Unsicherheiten des Vorhabens. Jedes trägt eine Gegenmaßnahme und einen Verantwortlichen.

| Risiko | Eintritt | Auswirkung | Gegenmaßnahme | Owner |
|---|---|---|---|---|
| Die Dateisystemabstraktion erweist sich als zu eng | mittel | hoch | ADR-0002 entscheidet die Schnittstelle vor M1; die Protokollquellen aus M6 werden bereits in M1 als Testimplementierung durchgespielt | Architekt |
| Die Aktionspalette wird zu mächtig und unübersichtlich | mittel | mittel | Die Palette zeigt nur Kontext-Aktionen und filtert nach Verfügbarkeit; die Filterregel wird in M1 festgelegt und in M8 geprüft | UI-Engineer |
| Der Fernbedienungs-Kontext scheitert an der Play-Prüfung | mittel | hoch | M8 ist ein eigener Meilenstein mit eigenem Prüfprotokoll, nicht eine Nachlieferung zu M4; die Erreichbarkeitsliste wird ab M1 mitgeführt | Produkt |
| SMB-Bibliotheken tragen ein ungünstiges Lizenzmodell | niedrig | mittel | `pavao` ist nicht eingebunden; die `deny.toml` erzwingt die Lizenzprüfung, eine Copyleft-Abhängigkeit fällt im Gate durch | Sicherheit |
| Die Quer-Kompilierung für Android und iOS scheitert an Systembibliotheken | hoch | hoch | `smb2` und `russh` sind reines Rust; Bibliotheken mit C-Anteilen (`pavao`, `ssh2`) sind ausgeschlossen | Architekt |
| Die iPadOS-Sandbox begrenzt den Funktionsumfang stärker als erwartet | hoch | mittel | Die Einschränkung steht in der Spezifikation und wird im M5-Dokument ausgewiesen, nicht verschwiegen | Produkt |
| Die Codelänge-Regel bremst die Entwicklung | mittel | niedrig | Die Grenze von 400 Zeilen zwingt zur sauberen Zerlegung; das ist der Zweck, kein Hindernis | Architekt |
| Der Build bricht in der CI und nicht lokal | mittel | mittel | Der zentrale Build in der CI ist das Gate; lokal wird nur entwickelt | Qualität |

Das höchste Risiko liegt bei der Dateisystemabstraktion, und es ist das einzige mit der Bewertung „hoch" bei beiden Achsen. Deshalb wird sie vor dem ersten Code in einem ADR entschieden und in M1 mit einer Testimplementierung einer entfernten Quelle geprüft, statt erst in M6 aufzutauchen. Die beiden Risiken mit der höchsten Eintrittswahrscheinlichkeit betreffen die Quer-Kompilierung und die iPadOS-Sandbox — beide sind unvermeidbar und werden deshalb früh verifiziert statt später umgangen.

### Rücksprungregel

Stellt sich in einem Meilenstein heraus, dass eine Grundannahme nicht trägt, wird **nicht** am Symptom herumgebaut. Der betroffene Meilenstein wird angehalten, die Annahme in einem ADR oder in der Spezifikation korrigiert, und die Arbeit am veränderten Teil wiederholt. Das kostet Tage, spart aber ein Feature, das auf einer falschen Annahme ruht.

---

## Abnahmeregelungen

Jede Meilensteinabnahme folgt demselben Muster. Die Datei liegt unter `docs/acceptance/M<n>-ACCEPTANCE.md` und trägt die Abschnitte in dieser Reihenfolge.

### Aufbau einer Abnahme

**Funktionalität.** Welche Funktionen des Meilenstands sind vorhanden und wie verhalten sie sich? Jede Zeile nennt die Anforderungs-ID und ein beobachtetes Verhalten, keine Behauptung.

**Technik.** Welche Architekturentscheidungen sind umgesetzt, welche Crate-Grenzen wurden eingehalten, welche Werkzeuge liefen durch?

**Leistung.** Welche Zielwerte wurden gemessen, mit welchem Werkzeug, mit welchem Ergebnis? Nicht gemessene Zielwerte werden als „nicht gemessen" ausgewiesen — nicht als „erfüllt".

**Benutzerführung.** Wie sieht der Meilenstein aus und wie wird er bedient? Tastaturführung ist getrennt auszuweisen, weil sie der Hauptbedienweg ist.

**Sicherheit und Datenschutz.** Welche Härtungen sind wirksam, und **welcher negative Test belegt das**? Ein positiver Test beweist nichts über die Abwehr.

**Interoperabilität.** Welche fremden Systeme wurden geprüft — Archive mit fremden Werkzeugen, Netzlaufwerke mit Samba, WebDAV mit Nextcloud?

**Bekannte Einschränkungen.** Was funktioniert nicht, und warum? Diese Liste ist Pflicht und darf nicht leer sein, solange es etwas gibt.

**Entscheidung.** Abgenommen, mit Auflagen, oder abgelehnt. Die Entscheidung nennt die Person und das Datum.

### Belegregeln

Ein Beleg ist eine Kommandoausgabe, ein Testname mit grünem Ergebnis oder ein Abzug aus einer Abnahmedokumentation. „Funktioniert" ist kein Beleg. „Sollte passen" ist kein Beleg. Eine Coverage-Prozentzahl ohne Messung ist kein Beleg.

Tests, die nicht ausgeführt werden konnten, werden als **NOT RUN mit exaktem Grund** ausgewiesen. Ein NOT RUN mit Begründung ist zulässig und ehrlich. Ein vorgetäuschter PASS ist unzulässig.

### Die Abnahme-Gates

Diese Gates bilden das Gerüst, an dem jede Änderung gemessen wird.

| Gate | Wann | Was geprüft wird | Was bei Rot passiert |
|---|---|---|---|
| **Formatierung** | jede Änderung | `cargo fmt --check` | Der Build bricht ab |
| **Statische Analyse** | jede Änderung | `cargo clippy --all-targets -- -D warnings` | Der Build bricht ab |
| **Lizenzprüfung** | jede Änderung | `cargo deny check` | Der Build bricht ab |
| **Schwachstellenprüfung** | jede Änderung | `cargo audit` | Der Build bricht ab |
| **Crate-Grenzen** | jede Änderung | `scripts/check-crate-boundaries.sh` | Der Build bricht ab |
| **Testlauf** | jede Änderung | `cargo test --workspace` | Der Build bricht ab |
| **Plattformbau** | vor Meilensteinabschluss | Bau auf Linux, macOS, Windows, FreeBSD, Android, iOS | Der Meilenstein gilt nicht als erreicht |
| **Leistung** | vor Meilensteinabschluss | Die Zielwerte der Spezifikation, gemessen mit `criterion` | Der Meilenstein gilt nicht als erreicht |
| **Sicherheitsnachweis** | vor Meilensteinabschluss | Die negativen Tests für jede Härtung | Der Meilenstein gilt nicht als erreicht |
| **Bedienerreichbarkeit** | vor M8 | Jede Aktion ist im `dpad`-Kontext erreichbar | M8 gilt nicht als erreicht |

Kein Gate wird abgeschwächt, um grün zu werden. Ein Lint wird nicht mit `#[allow]` wegkommentiert, eine Prüfung wird nicht aus der CI entfernt, ein Test wird nicht deaktiviert.

### Nachweisverzeichnis

Unter `docs/acceptance/` liegen die Abnahmedokumente, unter `docs/reviews/` die Reviews und unter `docs/security/` das Bedrohungsmodell. Die Dateien sind nummeriert und datiert, damit der Verlauf des Projekts nachvollziehbar bleibt.

---

## Teamorganisation

Die Arbeit wird mit einem Team von KI-Agenten organisiert, die klare Rollen und messbare Endbedingungen haben. Der Roster umfasst:

| Rolle | Verantwortung | Endbedingung |
|---|---|---|
| Product Manager | Anforderungs-IDs, Umfangsentscheidungen, Abnahmekriterien | Anforderungsdokument ohne Widersprüche, Abnahmekriterien messbar |
| Rust Architect | Crate-Grenzen, öffentliche API, ADRs, Workspace-Integrität | Grenzprüfung grün, alle ADRs geschrieben, `clippy` und `fmt` sauber |
| Core Engineer | Dateisystemmodell, Auftragswarteschlange, Umbenennung, Archiv | Unit-Tests für jede öffentliche Funktion, keine Stubs |
| UI Engineer | Panels, Tastaturbedienung, Theme, native Menüs | Tastaturführung ohne Maus, Kontrast geprüft |
| Security & Privacy | Bedrohungsmodell, Extraktionshärtung, Berechtigungen | Negative Tests belegen jede Härtung, `SECURITY.md` und `PRIVACY.md` stehen |
| QA Engineer | Testabdeckung, CI-Gate, Abnahmeevidenz | Gate wirksam nachgewiesen, Abnahmenachweise mit echter Ausgabe |
| Documentation Lead | Nutzerdokumentation, Glossar, `AGENTS.md` | Doku ohne tote Links, Glossar vollständig |

Ein Team-Durchlauf arbeitet in Zyklen aus Erzeugung und unabhängiger Prüfung: Der Erzeuger liefert, der Prüfer verifiziert eigenständig durch erneutes Ausführen der Kommandos, nicht durch Lesen der Zusammenfassung. Erst bei bestandener Prüfung akzeptiert der Owner den Zyklus.

---

## Getroffene Entscheidungen

Die folgenden Entscheidungen sind vom Owner am 24.10.2026 getroffen. Sie sind verbindlich. Jede Abweichung braucht eine neue Entscheidung, die hier mit Datum und Begründung nachgetragen wird.

**E1 — Strenge der Plattformanpassung: Option A.** egui als gemeinsame Oberfläche, ergänzt um native Menüleiste über `muda`, native Dialoge über `rfd`, Plattform-Shortcuts und Plattform-Metriken. Ein Codepfad, sechs Plattformen.

**E2 — Reichweite der Archivunterstützung: ZIP, TAR mit fünf Kompressionen, 7z.** RAR und die exotischen Formate bleiben außen vor. RAR erfordert eine Lizenz der WinRAR-Eigentümer, die nicht eingeholt wird.

**E3 — Strenge der Sicherheitshärtung: streng.** Kein `extract()` aus den Crates, eigene kanonisierte Pfadauflösung, Symlinks aus Archivmaterial werden nicht erzeugt, Größenlimits gegen Dekompressionsbomben, `cargo-deny` und `cargo-audit` als harte CI-Gates.

**E4 — App-Sandbox auf macOS: nein.** Auslieferung ohne App-Sandbox, mit Notarisierung, Hardened Runtime und einer Erststart-Seite, die „Full Disk Access" erklärt. Die App erzwingt die Berechtigung nicht und umgeht sie nicht.

**E5 — Viewer-Umfang zum Start: Stufe 1 vollständig.** Text, Hex, Bilder, Archivinhalt und Metadaten ab M1, PDF und Syntaxhervorhebung ab M2, Audio und Video ab M9. Eine ehrliche Liste mit zehn funktionierenden Formaten ist die Positionierung.

**E6 — Reihenfolge der Netzwerkmeilensteine: SMB und WebDAV zuerst.** Beide werden im eigenen Heimnetz genutzt, WebDAV deckt zugleich Nextcloud, ownCloud und die meisten NAS-Systeme ab. SFTP und FTP folgen in M6 gemeinsam mit dem Verbindungsmanagement.

**E7 — SMB-Härtung: streng.** SMB1 und NTLMv1 werden nicht implementiert. Signatur und Transportverschlüsselung werden nach Möglichkeit verlangt. Ein Server ohne Unterstützung für Signatur oder Verschlüsselung wird abgelehnt statt stillschweigend akzeptiert.

**E8 — Quest-Unterstützung: Bedienkontext `controller` auf der Android-Version.** Kein immersiver VR-Port. Die Einschränkung — im Panel-Modus gibt es keine Tastatur — wird in der Dokumentation ausgewiesen.

**E9 — Fernbedienungs-Bedienung: Vollversion, keine Zusatzfunktion.** Jede Aktion ist im `dpad`-Kontext erreichbar, weil Google Play die gesamte Funktionalität mit Fünf-Wege-D-Pad prüft. Eine Einschränkung auf eine Grundversion wäre auch eine kostenpflichtige Aufweichung der Devise „kostenlos".

### Offene Punkte für die Umsetzung

Die folgenden Punkte sind während der Umsetzung zu klären. Sie blockieren den Start nicht, brauchen aber eine Antwort, bevor die betroffene Zeile Code geschrieben wird.

**Bezugsgröße der Serienfolgen-Erkennung.** Ob der Serienname aus dem übergeordneten Verzeichnis übernommen oder bei jeder Serie neu eingegeben wird, ist eine Frage der Gewohnheit. Vorschlag: Der Verzeichnisname ist die Vorgabe, er wird aber immer angezeigt und ist überschreibbar. Die Entscheidung fällt bei der ersten Erprobung mit echten Serien.

**Umfang der Aktionpalette.** Ob die Palette nur Aktionen zeigt, die im aktuellen Kontext sinnvoll sind, oder alle Aktionen mit einem Hinweis auf nicht erreichbare. Vorschlag: nur erreichbare. Eine Palette, die ständig nicht funktionierende Einträge zeigt, ist genau das Clutter, das das Produkt vermeiden soll.

**Grenze der Ergebnisgröße bei Netzwerkquellen.** Der Vorschlag in der Spezifikation ist ein festes Lesebudget. Der genaue Wert richtet sich nach der kleinsten sinnvollen Zielplattform und wird bei M6 gemessen, nicht geraten.

## Quellenverzeichnis

[1] SpeedCommander — Produktseite, Funktionsübersicht: https://www.speedproject.de/
[2] SpeedCommander — Funktionen im Detail: https://www.speedproject.de/sc/funktionen/
[3] SpeedCommander (englisch) — Feature-Beschreibung: https://www.speedproject.com/sc/
[4] SpeedCommander — Wikipedia: https://de.wikipedia.org/wiki/SpeedCommander
[5] Forklift 3 Review: https://smartwebsetup.com/forklift-3-review/
[6] winit — Plattformunterstützung: https://rust-windowing.github.io/winit/winit/index.html
[7] egui auf crates.io: https://crates.io/crates/egui
[8] slint auf crates.io: https://crates.io/crates/slint
[9] Slint — FAQ zur Lizenzierung: https://github.com/slint-ui/slint/blob/master/FAQ.md
[10] Slint — Lizenzdatei: https://github.com/slint-ui/slint/blob/master/LICENSE.md
[11] Rust-GUI-Ökosystem — Entscheidungsgrundlage: https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/20-how-to-build.md
[12] iced — Mobilticket #302, Status 2026: https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/01-iced.md
[13] Apple — Zugriff auf Verzeichnisse (UIDocumentPickerViewController): https://developer.apple.com/documentation/uikit/providing-access-to-directories
[14] Apple — UIDocumentPickerViewController: https://developer.apple.com/documentation/uikit/uidocumentpickerviewcontroller
[15] Apple — Sicherheitsmodell der Laufzeitumgebung: https://support.apple.com/en-gb/guide/security/sec15bfe098e/web
[16] Android — Manage all files: https://developer.android.com/training/data-storage/manage-all-files
[17] Android — Scoped Storage: https://source.android.com/docs/core/storage/scoped
[18] Google Play — Richtlinie zu `MANAGE_EXTERNAL_STORAGE`: https://support.google.com/googleplay/android-developer/answer/10467955?hl=en
[19] NIST SP 800-88 Rev. 2 — Guidelines for Media Sanitization: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-88r2.pdf
[20] Berkeley — Secure Deletion Guideline: https://security.berkeley.edu/secure-deletion-guideline
[21] RUSTSEC-2026-0369 — zip: Unix-Berechtigungen durch Symlinks: https://rustsec.org/advisories/RUSTSEC-2026-0369.html
[22] RUSTSEC-2026-0367 — zip: Erhalt von SUID/SGID-Bits: https://rustsec.org/advisories/RUSTSEC-2026-0367.html
[23] RUSTSEC-2026-0368 — zip: Symlinks außerhalb des Zielverzeichnisses: https://rustsec.org/advisories/RUSTSEC-2026-0368.html
[24] RUSTSEC-2025-0168 — zip: fehlerhafte Pfadkanonisierung: https://rustsec.org/advisories/RUSTSEC-2025-0168.html
[25] `zip`-Crate — `ZipFile::enclosed_name`: https://docs.rs/zip/latest/zip/read/struct.ZipFile.html
[26] crates.io — Paketregister (Versionen und Lizenzen der genannten Crates): https://crates.io/
[27] muda — native Menüs für Windows, macOS, Linux/BSD: https://crates.io/crates/muda
[28] Photocraft — Workspace-Manifest: https://github.com/storytold/photocraft/blob/main/Cargo.toml
[29] Photocraft — Abstraktion des nativen Menüs: https://github.com/storytold/photocraft/blob/v0.12.0/crates/ui-egui/src/native_menu.rs
[30] Microsoft — Maximum Path Length Limitation: https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation
[31] Microsoft — Dateipfadlängen je Plattform: https://support.microsoft.com/en-us/onedrive/what-are-file-path-length-limits
[32] egui — plattformspezifische Details: https://deepwiki.com/emilk/egui/6.3-platform-specific-considerations
[33] egui — Übersicht und Plattformtabelle: https://deepwiki.com/emilk/egui
[34] Tauri — Release-Notizen 2.x: https://v2.tauri.app/release/tauri/
[35] Apple — Dateizugriff aus der macOS-App-Sandbox: https://developer.apple.com/tutorials/data/documentation/security/accessing-files-from-the-macos-app-sandbox.md
[36] Double Commander — Tastenbelegung als Quasi-Standard-Referenz: https://github.com/networm/doublecmd/blob/main/shortcuts.scf
[37] Rust-GUI-Ökosystem — Tauri im Detail: https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/04-tauri.md
[38] Die besten macOS-Dateimanager 2026: https://www.compsmag.com/best/mac-file-managers/
[39] Nextcloud — WebDAV-Basic-APIs (PROPFIND, PUT, MKCOL): https://docs.nextcloud.com/server/latest/developer_manual/client_apis/WebDAV/basic.html
[40] Android TV — Navigation mit D-Pad: https://developer.android.com/training/tv/get-started/navigation
[41] Android TV — TV-App-Qualitätsvorgaben (TV-DP, TV-DM, TV-DB, TV-ML, TV-MT): https://developer.android.com/develop/adaptive-apps/quality-guidelines/tv-app-quality
[42] TrueNAS — SMB-Dienst, NTLMv1-Abschaltung durch Microsoft: https://www.truenas.com/docs/scale/26/systemsettings/services/smbservice/
[43] Microsoft — SMB-Abfangschutz, SMB1-Abschaltung, Signaturpflicht: https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-interception-defense
[44] Meta — Horizon OS, Grundlagen (Android-basiert): https://developers.meta.com/vr/essentials/quick-start/
[45] Meta — Plattformen und Werkzeuge (Flat Panels, Gestenabbildung): https://developers.meta.com/vr/discover/platforms/
[46] openxr — Rust-Bindings für OpenXR: https://crates.io/crates/openxr
[47] crates.io — Paketregister (Versionen und Lizenzen der genannten Crates): https://crates.io/
[48] winit — Plattform- und Eingabeunterstützung: https://docs.rs/crate/winit/latest/source/FEATURES.md
