# Die Wörter der Arbeitsfläche — Deutsch.

job-kind-copy = Kopieren
job-kind-create-directory = Verzeichnis anlegen
job-kind-delete = Löschen
job-kind-move = Verschieben
job-kind-rename = Umbenennen
job-kind-sync = Abgleichen
job-state-cancelled = abgebrochen
job-state-done = fertig
job-state-failed = fehlgeschlagen
job-state-queued = in der Warteschlange
job-state-running = läuft
job-state-verifying = prüft
job-state-waiting = wartet
menu-application = Programm
menu-file = Datei
menu-panel = Panel
menu-view = Ansicht
panel-hint-navigation = Pfeiltasten bewegen - Enter öffnet - Esc geht zurück
panel-job-list-empty = Es läuft kein Auftrag
panel-job-list-title = Aufträge
panel-job-progress = { $count } von { $total } Einträgen
panel-job-rate = { $rate }/s
panel-job-eta-seconds = noch { $seconds } s
panel-job-eta-minutes = noch { $minutes } min
panel-layout-side-by-side = Nebeneinander
panel-layout-stacked = Untereinander
panel-quick-view-binary = Diese Datei ist kein Text
panel-quick-view-empty = Es ist keine Datei markiert
panel-quick-view-title = Schnellansicht
panel-tab-close = Schließen
panel-title-left = Linkes Panel
panel-title-right = Rechtes Panel
panel-column-name = Name
panel-column-size = Größe
panel-column-date = Datum
panel-column-type = Typ
panel-tree-title = Verzeichnisse
panel-type-file = Datei
panel-type-directory = Verzeichnis
panel-type-link = Verknüpfung
panel-type-other = Sonstiges
panel-empty = Das Verzeichnis ist leer
panel-status-ready = Bereit
panel-status-entries = { $count_number ->
        [one] 1 Eintrag
       *[other] { $count } Einträge
    }
panel-status-selected = { $count_number ->
        [one] 1 ausgewählt
       *[other] { $count } ausgewählt
    }
panel-status-filtered = { $count } von { $total } Einträgen
panel-filter-label = Filter
panel-filter-placeholder = Namensmuster
panel-filter-matches = { $count_number ->
        [one] 1 Treffer
       *[other] { $count } Treffer
    }
panel-filter-grammar = * beliebig viele Zeichen · ? genau ein Zeichen · ~ ohne Groß-/Kleinschreibung
panel-tab = Reiter { $number }
panel-view-details = Details
panel-view-compact = Kompakt
panel-focus = Fokussiertes Panel
panel-undo-notice = Rückgängig entfernt nur die Kopien; der vorherige Zustand wird nicht wiederhergestellt.

# The messages of the errors of the surface.

error-binding-empty = Die Belegung ist leer.
error-binding-key-twice = Die Belegung nennt die Taste { $key } zweimal.
error-binding-no-key = Die Belegung nennt keine Taste.
error-binding-empty-palette-entry = Der Eintrag der Aktionspalette trägt keinen Namen.
error-binding-unknown-action = Die Aktion { $identifier } gehört nicht zum Katalog.
error-binding-unknown-key = Die Belegung { $binding } nennt die unbekannte Taste { $key }.
error-transfer-same-panel = Das Panel { $side } kann nicht gleichzeitig Quelle und Ziel einer Übertragung sein.

# The message of a path that is longer than the platform allows.

path-too-long = Der Pfad { $path } ist zu lang: { $amount } { $unit }, die Grenze ist { $limit }.
path-too-long-advice = Verschiebe die Datei in ein kürzeres Verzeichnis, oder benenne das Verzeichnis um, das sie enthält.
path-too-long-unit-characters = Zeichen
path-too-long-unit-bytes = Bytes

# The messages of the failures of the jobs.

io-read = Die Datei konnte nicht gelesen werden: { $item }
io-write = Die Datei konnte nicht geschrieben werden: { $item }
io-permission = Das System hat den Zugriff verweigert: { $item }
job-checksum-mismatch = Die Kopie weicht von der Quelle ab: { $item }
path-reserved = Der Name ist auf nicht jeder Plattform brauchbar: { $item }
job-cancelled = Der Auftrag wurde abgebrochen: { $item }
job-not-confirmed = Der Auftrag wurde nicht bestätigt: { $item }
io-not-found = Der Pfad gibt es nicht: { $item }
job-already-exists = Der Pfad gibt es schon: { $item }
job-unsupported-mode = Die Art des Auftrags wird nicht unterstützt: { $item }
job-rename-conflict = Der Name ist schon vergeben: { $item }
