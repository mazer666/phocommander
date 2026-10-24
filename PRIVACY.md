# Privacy Policy

Phocommander is a local program. It runs on your machine, shows you your files, and
does nothing else.

This document states which data is stored, where, how long it stays and how you
remove it — and, just as importantly, what never leaves your machine.

---

## The short version

| | |
|---|---|
| **Telemetry** | None. Not a counter, not a ping, not a crash report. |
| **Analytics** | None. |
| **Advertising** | None. No ad SDK, no sponsored placement, no affiliate link. |
| **Cloud** | None. Phocommander operates no server of its own. |
| **Usage profiles** | None. No identifier is created for you or about you. |
| **Registration** | Not needed. Every local function works without an account or a login. |
| **Third parties** | None. There is no data processor, no subprocessor, no SDK that phones home. |

The only network connection Phocommander ever opens is one **you** started, with a
**server you** named. There is no other path out of the process — and that is checked
mechanically: the CI gate fails if a crate from a telemetry or analytics category is
referenced, and a test proves that starting the process opens no network connection.

---

## What is stored

Phocommander writes only what you configure it to write:

| Data | Where it comes from | Why it exists |
|---|---|---|
| **Settings** — theme, row height, hidden-file visibility, panel layout, conflict policy, size format, worker-thread count | Your choices in the settings dialog | So the program looks and behaves the way you configured it, without asking again on every start |
| **Key bindings** | Your changes to the default bindings | So your bindings are the same on every device you use the program on |
| **Favourites** — folder and application favourites | Entries you create | So the places you return to are one keystroke away |
| **Recently opened directories** | Directories you opened, if session restore is enabled (`[general] restore_session`) | So the program opens where you left off instead of at a default location |

That is the complete list. Phocommander does **not** store:

- file contents, file names of files you did not open, or directory listings,
- a browsing history beyond the "recently opened directories" above,
- keystrokes, clipboard contents, or screen contents,
- device identifiers, advertising identifiers, or hardware fingerprints,
- crash reports sent anywhere — there is nowhere to send them to.

---

## Where it is stored

One central configuration file, in the location the operating system prescribes:

| Platform | Location |
|---|---|
| macOS | `~/Library/Application Support/Phocommander/` |
| Windows | `%APPDATA%\Phocommander\` |
| Linux, BSD | `$XDG_CONFIG_HOME/phocommander/` (by default `~/.config/phocommander/`) |
| Android, iPadOS | The application sandbox of the app |

The file is plain TOML and readable with any text editor. It carries a `schema_version`
field; when the format changes incompatibly, Phocommander migrates the file and keeps
the original next to it as `.bak`. The `.bak` is yours to delete.

Import and export of a configuration are possible so that you can move your settings
to another machine — that is an action you start, and it writes where you tell it to.

---

## How long it stays

Until you remove it. There is no expiry date, no automatic cleanup and no retention
period, because there is no server that could enforce one. Uninstalling the program
removes what the platform's uninstaller removes; anything the program wrote in your
configuration directory stays there unless you delete it.

---

## How to remove it

- **Everything at once:** delete the configuration directory listed above.
- **Individual parts:** edit the configuration file and remove the corresponding
  section — `[general]` for settings, `[keymap]` for bindings, the favourites file
  for favourites.
- **Only the recent directories:** set `[general] restore_session = false`, or clear
  the list in the settings.
- **A backup of the old format:** delete the `.bak` file next to the configuration.

After removing the files, no residue of your usage remains anywhere — there is no
other place where it could remain.

---

## Credentials for remote sources

If you connect to SFTP, FTP, SMB or WebDAV, the connection details are split in two:

| Part | Where it lives |
|---|---|
| Server address, port, user name, protocol | The configuration file — these are the parts you need to see and edit |
| **Password, private key, passphrase** | **The keychain of your operating system** — Keychain on macOS, Credential Manager on Windows, Secret Service on Linux and BSD |

The secret is never written into the configuration file, and it is never written into
a log file. The keychain is accessed through the `keyring` crate.

On platforms without a system keychain — Android and iPadOS without a Keychain
extension — Phocommander falls back to the application sandbox and **tells you** that
the data is then not protected by the operating system. Silently storing a password
in a world-readable file would be the one thing worse.

---

## Log files

Phocommander writes a log for troubleshooting. The log contains the protocol, the
time, the result and the duration of an operation. It contains **no user names, no
server names and no IP addresses**. If a field is needed for a bug report, it is
masked.

The log level is configurable (`[privacy] log_level`, default `warn`). Setting it to
`error` keeps only problems.

---

## What goes to a remote server

Only when you connect to one, and only what the operation needs.

**What is sent:**

- The requests that the operation consists of: listing a directory, reading a file,
  writing a file, deleting, creating a directory.
- The file names and paths inside the directory you asked for.
- The data of the file you transferred — in the direction you chose.

**What is not sent:**

- Anything about your local machine — no file lists, no recently opened directories, no
  favourites, no settings, no key bindings, no version information, no device
  identifier.
- No telemetry, no "usage ping", no heartbeat, no licence check.
- No other file than the one you operated on.
- No data to any third party: the connection goes directly between your program and
  the server you named. Phocommander operates no intermediary server.

The server sees an ordinary client. It cannot tell that it is talking to Phocommander
rather than to any other file transfer program — there is nothing to tell.

If you use a cloud provider via OAuth 2.0 or an S3-compatible target, the same rule
applies: the token exchange and the file access happen directly between the program
and the provider, the refresh token stays in the system keychain, and there is no
Phocommander server in between.

---

## Children

Phocommander is a file manager; it has no user accounts and collects no personal
data, so it has nothing to collect from or about a child.

---

## Changes to this policy

If the behaviour described here ever changes, this file changes in the same pull
request that changes the behaviour — and the changelog says so. A change that would
introduce telemetry, advertising or a server of our own is not a minor patch; it
contradicts the product rules and would need a new decision by the owner, recorded in
[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md).

---

## Where to go with questions

Privacy questions are not a support burden — they are design questions, and the
answer belongs in the specification. Open an issue for a question, and read
[`SECURITY.md`](SECURITY.md) if your question is about a vulnerability rather than
about data.
