# Beyond the Reference — Ideas that support the goals

**Status:** Living document · **Owner:** Product Manager · **Date:** 24 October 2026

This document collects ideas that go beyond the functional scope of the reference product. The
reference defines the floor of the product, not its ceiling. Everything listed here serves one of
the three product principles — free, clutter-free, advertising-free — or one of the quality
dimensions: technical capability, performance, usability, quality of life, product scope, security
and privacy.

An idea enters this document when it is worth more than it costs. It leaves the document when it
is implemented, rejected with a reason, or deliberately postponed to a later milestone. The
document exists so that an idea is not lost in a discussion and not silently dropped.

Each entry states the idea, the goal it serves, the cost, and the milestone it would fit into.

---

## 1. Navigation

### 1.1 Jump to a path from anywhere

A single field that accepts a path, a partial name or a remembered location, and takes the user
there. On a remote or a released root it offers the roots the user actually has, not a file
dialog with a tree that has to be walked.

*Serves:* usability, quality of life. *Cost:* small — the command line of M2 already resolves a
path; this makes it discoverable. *Milestone:* M2.

### 1.2 Remember the last directory of every source

Per source, not per application: the local disk, each network share and each released root keep
their own last position. Switching between a local folder and a server therefore lands where the
user was in that source, not where they were in the other one.

*Serves:* usability. *Cost:* small. *Milestone:* M6.

### 1.3 A history of the visited directories

A navigable list of the directories that were visited, with the time. It is a local history in
the configuration directory and contains nothing but paths the user opened themselves.

*Serves:* usability. *Cost:* small, and it raises a privacy question that is answered by the same
rule as everything else: nothing leaves the machine. *Milestone:* M2.

---

## 2. Working with many files

### 2.1 Compare two panels without a separate command

A permanent, quiet comparison: when both panels show directories of the same kind, the differing
entries are marked in the file list. The comparison runs in the background and updates when the
panel content changes. The explicit comparison command of FR-79 remains, for the case where the
user wants the full report.

*Serves:* usability, quality of life, product scope. *Cost:* moderate — it needs the comparison
function that M2 builds anyway, wired into the list rendering. *Milestone:* M2.

### 2.2 Select by pattern, with a preview of the result

The selection by pattern of FR-100 shows how many entries match before the selection is applied,
and the pattern language is documented in the help. A user who guesses the pattern wrong sees
that they guessed it wrong, instead of selecting nothing and wondering why.

*Serves:* usability. *Cost:* small. *Milestone:* M1.

### 2.3 A checksum column in the file list

The column shows the short checksum of the marked entry, computed in the background. It is the
same checksum that the verified copy mode uses, so the column and the verification agree.

*Serves:* technical capability, quality of life. *Cost:* moderate — the computation must not
block the list. *Milestone:* M2.

---

## 3. Safety

### 3.1 A preview of what a synchronisation will delete

The plan of the synchronisation modes already lists the deletions. The idea is to present them
more prominently: the entries that will be deleted are listed separately from the entries that
will be copied, and the deletion section requires a deliberate confirmation of its own.

*Serves:* safety, usability. *Cost:* small, given the plan exists. *Milestone:* M2.

### 3.2 The trash instead of the secure delete as the default

Already the decision of the specification. What the idea adds is the explicit behaviour when the
trash is not available — on a read-only source, or a released iPadOS root, the file cannot be
moved to a trash. The interface must say that instead of deleting it directly.

*Serves:* safety, usability. *Cost:* small. *Milestone:* M1.

### 3.3 A dry run for every destructive operation

A copy, a move, a rename and a synchronisation can be executed as a dry run: the operation
computes everything it would do and shows the result without touching a single file. The
`sync-preview` mode of the specification is the generalisation of this idea to the
synchronisation; the idea extends it to the ordinary operations.

*Serves:* safety. *Cost:* moderate. *Milestone:* M2.

---

## 4. Performance

### 4.1 A directory cache with a validity check

The listing of a directory is cached, and the cache is invalidated by the file system watcher
rather than by a timestamp comparison. The result: a panel that is revisited shows its content
immediately, and the cache cannot serve stale content because the watcher is the authority.

*Serves:* performance, quality of life. *Cost:* moderate — the watcher has to be reliable, and a
missed event would show stale content. *Milestone:* M2.

### 4.2 Reading the directory in parallel with the metadata

Where a file system allows it, the names and the metadata are read in one pass instead of two.
On a network share this halves the number of round trips, which is the dominant cost there.

*Serves:* performance. *Cost:* moderate, and it is only possible where the protocol supports it.
*Milestone:* M6.

### 4.3 A measurement mode in the interface

A switch that shows how long each operation took: the listing, the sort, the rendering. The user
can see where the time goes without a profiler. This is the performance discipline made visible.

*Serves:* performance, technical capability. *Cost:* small. *Milestone:* M2.

---

## 5. Interfaces

### 5.1 A layout that remembers itself per window size

The panel arrangement, the column set and the position of the splitter are stored per window
geometry. A wide window gets the two panels side by side, a narrow one stacks them, and the user
does not have to rearrange the interface every time the window changes.

*Serves:* usability, quality of life. *Cost:* small. *Milestone:* M2.

### 5.2 The action palette as a search field over the commands

The palette filters as the user types, and the filter matches the command name in the interface
language. A user who does not know the name of a command can describe what they want.

*Serves:* usability. *Cost:* small, given the palette exists. *Milestone:* M8.

### 5.3 A keyboard-only tour of the interface

A short, dismissible sequence that walks through the ten most important commands, once, on the
first start, and never again. It is not an onboarding dialog that blocks the interface — the
interface is usable while it runs.

*Serves:* usability. *Cost:* small. *Milestone:* M2.

---

## 6. Architecture

### 6.1 One crate per protocol, not one crate for all of them

The specification places all remote sources in one crate. Splitting them into one crate per
protocol would make the dependency footprint of a build smaller and would let a licence
problem stay local to one protocol. The cost is more crates to maintain.

*Serves:* technical capability, security. *Cost:* moderate. *Milestone:* M6, if the dependency
footprint of the remote crate turns out to be a problem in practice.

### 6.2 A command interface for the headless use

The same commands that the interface uses, callable without a window. It makes the product
scriptable, makes the end-to-end tests of the later milestones possible without a user interface,
and it is the reason a macro system of M9 would not need a second implementation.

*Serves:* technical capability, product scope, quality of life. *Cost:* moderate. *Milestone:* M9.

### 6.3 A plugin interface for the viewer formats

The viewer formats are registered through a trait. A plugin interface would go further and allow
a format to be loaded at run time from a file. The risk is that a plugin is untrusted code in
the process, which sits badly next to the security rules of the specification.

*Serves:* product scope. *Cost:* high, and it weakens the security position. *Milestone:* not
planned; the decision is recorded so that it is not reconsidered without the trade-off.

---

## 7. Documentation and community

### 7.1 A glossary that is generated from the code

The identifiers of the actions, the error codes and the configuration keys would be extracted
from the source into the glossary, so that the documentation cannot drift away from the code.

*Serves:* quality of life for contributors. *Cost:* small, once the extraction script exists.
*Milestone:* M2.

### 7.2 A recipe collection

Short, task-oriented recipes: how to rename a season, how to compare two folders, how to set up a
synchronisation, how to connect to a NAS. Each is a page of the user guide and each is tested
against a real version.

*Serves:* usability, product scope. *Cost:* moderate. *Milestone:* M2 and onwards.

### 7.3 A translation platform

The Fluent catalogues are designed for it, and a translation platform would let contributors
translate the interface without touching the repository. The cost is an external dependency and
a moderation workflow, which sits badly next to the principle that the product operates without a
service of its own.

*Serves:* product scope, community. *Cost:* moderate, with a privacy consideration.
*Milestone:* not planned; the catalogues stay in the repository so that the project has no
external dependency.

---

## 8. Ideas that were considered and rejected

| Idea | Why it was rejected |
|---|---|
| A plugin system that loads code at run time | Untrusted code in the process contradicts the security rules. The trait-based extension points give the same extensibility without it |
| An automatic cloud sync in the background | A background service that polls a provider contradicts the promise of zero telemetry and of no service of our own |
| A translation platform with an account | The project would depend on a service and collect data about its contributors |
| A built-in file preview from a remote source without a limit | A remote source can announce a size it does not have; the response has to be bounded |
| A full immersive VR port | A file manager lives on a keyboard and a fine pointer, and a panel in space has neither |
| An interface that rearranges itself automatically based on usage | The product has no usage data, because it has no telemetry. An automatic rearrangement would also be exactly the kind of surprise the clutter-free principle forbids |
