import Foundation

/// The directories that the user released, and the permission that goes
/// with them.
///
/// WHY: iPadOS is fully sandboxed, and a lasting permission is stored as a
/// security-scoped bookmark. The bookmark survives a restart of the
/// application, and the user can revoke it at any time in the settings. The
/// application therefore has to answer three things: which directories it
/// may read, whether it may write to them, and what it does when the user
/// has taken the permission away.
final class Bookmarks {

    /// The name of the file that holds the bookmarks.
    private static let fileName = "bookmarks.plist"

    /// The bookmarks of the released directories.
    private(set) var bookmarks: [Data: URL] = [:]

    /// Whether the user may write to the directory.
    private var writeable: Set<Data> = []

    /// The directories that the user released.
    var roots: [URL] {
        Array(bookmarks.values)
    }

    /// The state of the permission of a directory.
    ///
    /// WHY: a revoked permission is not an error of the program but a
    ///       decision of the user. The shell shows the directory as it is
    ///       and says that the permission is gone — it does not crash and
    ///       it does not pretend that the directory is still there.
    enum Access {
        /// The permission is in force.
        case granted

        /// The user took the permission away.
        case revoked
    }

    /// Reads the bookmarks of the application.
    init() {
        let url = Self.fileURL()
        guard let data = try? Data(contentsOf: url),
              let stored = try? PropertyListSerialization.propertyList(
                from: data,
                options: [],
                format: nil
              ) as? [String: Data]
        else {
            return
        }
        for (key, value) in stored {
            var stale = false
            if let url = try? URL(
                resolvingBookmarkData: value,
                options: .withSecurityScope,
                relativeTo: nil,
                bookmarkDataIsStale: &stale
            ) {
                bookmarks[value] = url
                if stale {
                    // WHY: a bookmark that the system has marked as stale
                    //       still works, but it must be written again.
                    continue
                }
            }
        }
    }

    /// Adds the directory that the user released.
    func add(_ url: URL) {
        guard let data = try? url.bookmarkData(
            options: .withSecurityScope,
            includingResourceValuesForKeys: nil,
            relativeTo: nil
        ) else {
            return
        }
        bookmarks[data] = url
        writeable.insert(data)
        save()
    }

    /// Whether the permission of the directory is still in force.
    ///
    /// WHY: the system tells the application that the permission is gone
    ///       only when the application asks for it. The shell asks before
    ///       it reads a directory, and it treats the answer as it is.
    func access(of url: URL) -> Access {
        let started = url.startAccessingSecurityScopedResource()
        if started {
            url.stopAccessingSecurityScopedResource()
            return .granted
        }
        return .revoked
    }

    /// Whether the application may write to the directory.
    func mayWrite(_ url: URL) -> Bool {
        bookmarks.first { $0.value == url }
            .map { writeable.contains($0.key) } ?? false
    }

    /// Writes the bookmarks into the directory of the application.
    private func save() {
        var stored: [String: Data] = [:]
        for (key, value) in bookmarks {
            stored[value.path] = key
        }
        guard let data = try? PropertyListSerialization.data(
            fromPropertyList: stored,
            format: .binary,
            options: 0
        ) else {
            return
        }
        try? data.write(to: Self.fileURL())
    }

    /// The file that holds the bookmarks.
    private static func fileURL() -> URL {
        let directory = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first ?? URL(fileURLWithPath: NSTemporaryDirectory())
        try? FileManager.default.createDirectory(
            at: directory,
            withIntermediateDirectories: true
        )
        return directory.appendingPathComponent(fileName)
    }
}
