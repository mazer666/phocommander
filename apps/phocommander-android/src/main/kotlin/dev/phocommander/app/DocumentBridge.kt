package dev.phocommander.app

/**
 * The bridge between the Kotlin shell and the Rust core.
 *
 * WHY: the rules of the documents — the order of the list, the operations
 * that a document allows, the reason for a refusal — live in the Rust core,
 * because they are the rules of the product and not of the platform. The
 * shell hands the documents over and draws what the core answers.
 *
 * The library of Rust speaks a plain C interface, so that the shell does
 * not need a generator for the bindings: the strings go in and out, and the
 * numbers are numbers.
 */
class DocumentBridge {

    /**
     * The library of Rust, loaded once.
     *
     * WHY: the library is loaded when the application starts and stays
     * loaded until it ends. Loading it again for every list would cost
     * time for nothing.
     */
    private val library: Any? by lazy {
        try {
            System.loadLibrary("phocommander_android")
        } catch (error: UnsatisfiedLinkError) {
            // WHY: the library is missing when the application was built
            //       without the NDK. The shell says so and carries on with
            //       an empty list — a program that shows nothing is better
            //       than a program that crashes on the first frame.
            null
        }
    }

    /**
     * The rows of a folder, in the order of the panel.
     *
     * @param documents the documents of the folder: the identifier, the
     *   name, whether it is a folder, and whether the user may write to it.
     */
    fun listing(documents: List<DocumentRecord>): List<ListingRow> {
        val loaded = library ?: return emptyList()
        return runCatching {
            val flat = documents.joinToString("\n") {
                listOf(it.id, it.name, it.isFolder.toString(), it.mayWrite.toString())
                    .joinToString("\t")
            }
            val answer = nativeListing(loaded, flat)
            answer.map { row ->
                ListingRow(
                    id = row[0],
                    name = row[1],
                    isFolder = row[2] == "1",
                    mayWrite = row[3] == "1",
                    mayDelete = row[4] == "1",
                    mayRename = row[5] == "1",
                )
            }
        }.getOrElse { emptyList() }
    }

    /**
     * The reason why a document refuses an operation.
     *
     * @param mayWrite whether the user may write to the document.
     * @return the key of the catalogue, or nothing when the operation is
     *   allowed.
     */
    fun reason(mayWrite: Boolean): String? =
        if (mayWrite) null else "policy-read-only"

    /**
     * The entry point of the library of Rust.
     *
     * WHY: the signature is the C interface of the library: a pointer to
     * the bytes of the input, their number, and a pointer to the answer.
     * The shell reads the answer as a string of lines.
     */
    @Suppress("unused")
    private external fun nativeListing(library: Any, documents: String): Array<String>
}

/**
 * One document as the shell hands it over.
 */
data class DocumentRecord(
    val id: String,
    val name: String,
    val isFolder: Boolean,
    val mayWrite: Boolean,
)

/**
 * One row of the list of a panel.
 */
data class ListingRow(
    val id: String,
    val name: String,
    val isFolder: Boolean,
    val mayWrite: Boolean,
    val mayDelete: Boolean,
    val mayRename: Boolean,
)
