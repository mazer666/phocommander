# The words of the work surface — English.

job-kind-copy = Copy
job-kind-create-directory = Create directory
job-kind-delete = Delete
job-kind-move = Move
job-kind-rename = Rename
job-kind-sync = Synchronise
job-state-cancelled = cancelled
job-state-done = done
job-state-failed = failed
job-state-queued = queued
job-state-running = running
job-state-verifying = verifying
job-state-waiting = waiting
menu-application = Application
menu-file = File
menu-panel = Panel
menu-view = View
panel-hint-navigation = Direction keys move the focus - Enter opens - Esc goes back
panel-job-list-empty = No job is running
panel-job-list-title = Jobs
panel-job-progress = { $count } of { $total } items
panel-layout-side-by-side = Side by side
panel-layout-stacked = One below the other
panel-quick-view-binary = This file is not text
panel-quick-view-empty = No file is marked
panel-quick-view-title = Quick view
panel-tab-close = Close
panel-title-left = Left panel
panel-title-right = Right panel
panel-column-name = Name
panel-column-size = Size
panel-column-date = Date
panel-column-type = Type
panel-tree-title = Directories
panel-type-file = File
panel-type-directory = Directory
panel-type-link = Link
panel-type-other = Other
panel-empty = The directory is empty
panel-status-ready = Ready
panel-status-entries = { $count_number ->
        [one] 1 entry
       *[other] { $count } entries
    }
panel-status-selected = { $count_number ->
        [one] 1 selected
       *[other] { $count } selected
    }
panel-status-filtered = { $count } of { $total } entries
panel-filter-label = Filter
panel-filter-placeholder = name pattern
panel-filter-matches = { $count_number ->
        [one] 1 match
       *[other] { $count } matches
    }
panel-filter-grammar = * any characters · ? exactly one · ~ ignore case
panel-tab = Tab { $number }
panel-view-details = Details
panel-view-compact = Compact
panel-focus = Focused panel
panel-undo-notice = Undo removes the copies only; the previous state is not restored.

# The messages of the errors of the surface.

error-binding-empty = The binding is empty.
error-binding-key-twice = The binding names the key { $key } twice.
error-binding-no-key = The binding names no key.
error-binding-empty-palette-entry = The entry of the action palette carries no name.
error-binding-unknown-action = The action { $identifier } is not part of the catalogue.
error-binding-unknown-key = The binding { $binding } names the unknown key { $key }.
error-transfer-same-panel = The { $side } panel cannot be the source and the destination of a transfer at the same time.

# The message of a path that is longer than the platform allows.

path-too-long = The path { $path } is too long: { $amount } { $unit }, the limit is { $limit }.
path-too-long-advice = Move the file into a shorter directory, or rename the directory that holds it.
path-too-long-unit-characters = characters
path-too-long-unit-bytes = bytes

# The messages of the failures of the jobs.

io-read = The file could not be read: { $item }
io-write = The file could not be written: { $item }
io-permission = The system refused the access: { $item }
job-checksum-mismatch = The copy differs from the source: { $item }
path-reserved = The name is not usable on every platform: { $item }
job-cancelled = The job was cancelled: { $item }
job-not-confirmed = The job was not confirmed: { $item }
io-not-found = The path does not exist: { $item }
job-already-exists = The path exists already: { $item }
job-unsupported-mode = The mode of the job is not supported: { $item }
job-rename-conflict = The name is taken already: { $item }
