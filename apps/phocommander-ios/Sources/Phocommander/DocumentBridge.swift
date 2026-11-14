import Foundation
import PhocommanderBridge

/// The bridge between the Swift shell and the Rust core.
///
/// WHY: the rules of the documents — the order of the list, the operations
/// that a document allows, the reason for a refusal — live in the Rust
/// core, because they are the rules of the product and not of the system.
/// The shell hands the documents over and draws what the core answers.
///
/// The library of Rust speaks a plain C interface, so that the shell needs
/// no generator of bindings: the strings go in and out, and the numbers are
/// numbers.
final class DocumentBridge {

    /// The library of Rust, loaded once.
    ///
    /// WHY: the library is loaded when the application starts and stays
    /// loaded until it ends. Loading it again for every list would cost
    /// time for nothing.
    private let loaded: Bool

    init() {
        loaded = (dlopen(
            Bundle.main.bundlePath.appending("/libphocommander_bridge.dylib"),
            RTLD_NOW
        ) != nil)
    }

    /// The rows of a folder, in the order of the panel.
    ///
    /// - Parameter documents: the documents of the folder — the identifier,
    ///   the name, whether it is a folder, and whether the user may write
    ///   to it.
    func listing(documents: [DocumentRecord]) -> [ListingRow] {
        guard loaded else { return [] }
        let flat = documents
            .map { [$0.id, $0.name, $0.isFolder ? "1" : "0", $0.mayWrite ? "1" : "0"]
                .joined(separator: "\t") }
            .joined(separator: "\n")
        let bytes = Array(flat.utf8)
        guard let answer = phocommander_bridge_listing(bytes, bytes.count) else {
            return []
        }
        defer { phocommander_bridge_free(answer) }
        let text = String(cString: answer)
        return text.split(separator: "\n", omittingEmptySubsequences: false)
            .map { row in ListingRow(row) }
    }

    /// The reason why a document refuses an operation.
    func reason(mayWrite: Bool) -> String? {
        mayWrite ? nil : "policy-read-only"
    }
}

/// One document as the shell hands it over.
struct DocumentRecord {
    let id: String
    let name: String
    let isFolder: Bool
    let mayWrite: Bool
}

/// One row of the list of a panel.
struct ListingRow {
    let id: String
    let name: String
    let isFolder: Bool
    let mayWrite: Bool
    let mayDelete: Bool
    let mayRename: Bool

    /// Reads a row of the answer of the core.
    init(_ row: Substring) {
        let values = row.split(separator: "\t", omittingEmptySubsequences: false)
        id = values.first.map(String.init) ?? ""
        name = values.count > 1 ? String(values[1]) : ""
        isFolder = values.count > 2 && values[2] == "1"
        mayWrite = values.count > 3 && values[3] == "1"
        mayDelete = values.count > 4 && values[4] == "1"
        mayRename = values.count > 5 && values[5] == "1"
    }
}
