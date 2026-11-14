import SwiftUI
import UniformTypeIdentifiers

/// The application of iPadOS.
///
/// WHY: the iPadOS version is not a file system browser. It is a viewer
/// and an organiser for the root directories that the user has explicitly
/// released, and the surface stays useful in the split view: the touch
/// selects, the keyboard operates.
///
/// The permission is the decision of the system, and the application asks
/// for it once — through the document picker — and then holds it in the
/// bookmarks.
@main
struct PhocommanderApp: App {

    /// The state of the application.
    @StateObject private var model = SurfaceModel()

    var body: some Scene {
        WindowGroup {
            SplitView {
                // The panel that has the focus.
                FileList(model: model, side: .left)
                FileList(model: model, side: .right)
            }
            .navigationSplitViewStyle(.balanced)
        }
    }
}

/// The model of the surface.
final class SurfaceModel: ObservableObject {

    /// The bridge to the Rust core.
    let bridge = DocumentBridge()

    /// The directories that the user released.
    let bookmarks = Bookmarks()

    /// The rows of the panel that has the focus.
    @Published var rows: [ListingRow] = []

    /// The document that the viewer shows.
    @Published var openDocument: String?

    /// Whether the permission of the directory is still in force.
    ///
    /// WHY: the user can revoke the permission at any time. The application
    ///   asks before it reads, and it says that the permission is gone
    ///   instead of failing silently.
    @Published var permissionRevoked = false

    /// Reads the documents of a released directory.
    func refresh() {
        guard let url = bookmarks.roots.first else {
            rows = []
            return
        }
        guard bookmarks.access(of: url) == .granted else {
            permissionRevoked = true
            rows = []
            return
        }
        permissionRevoked = false
        let records = documents(of: url)
        rows = bridge.listing(documents: records)
    }

    /// The documents of a directory.
    ///
    /// WHY: the permission of the system covers the content of the directory
    ///   recursively, and the shell reads the content with the coordinator
    ///   of the file system.
    private func documents(of url: URL) -> [DocumentRecord] {
        let keys: [URLResourceKey] = [.isDirectoryKey, .nameKey]
        guard let contents = try? FileManager.default.contentsOfDirectory(
            at: url,
            includingPropertiesForKeys: keys,
            options: [.skipsHiddenFiles]
        ) else {
            return []
        }
        let mayWrite = bookmarks.mayWrite(url)
        return contents.map { child in
            let isFolder = (try? child.resourceValues(forKeys: [.isDirectoryKey]))?
                .isDirectory ?? false
            return DocumentRecord(
                id: child.path,
                name: child.lastPathComponent,
                isFolder: isFolder,
                mayWrite: mayWrite
            )
        }
    }
}

/// One of the two panels.
struct FileList: View {

    /// The model of the surface.
    @ObservedObject var model: SurfaceModel

    /// The side of the panel.
    let side: Side

    /// The side of a panel.
    enum Side {
        case left
        case right
    }

    var body: some View {
        List(model.rows, id: \.id) { row in
            HStack {
                Text(row.name).lineLimit(1)
                Spacer()
                if !row.mayWrite {
                    // WHY: the limitation is shown in the interface and not
                    //   only in the documentation. A document that the user
                    //   may only read says so.
                    Image(systemName: "lock")
                        .foregroundStyle(.secondary)
                }
            }
            // WHY: the target of a finger is forty-four points high, and
            //   the rule is the same on every platform.
            .frame(minHeight: 44)
            .onTapGesture {
                if row.isFolder {
                    model.refresh()
                } else {
                    model.openDocument = row.name
                }
            }
        }
        .overlay {
            if model.permissionRevoked {
                // WHY: the user took the permission away. The application
                //   says so and offers the way to give it back.
                VStack {
                    Text("The permission for this directory is gone.")
                    Button("Choose a directory") {
                        model.refresh()
                    }
                }
            }
        }
    }
}
