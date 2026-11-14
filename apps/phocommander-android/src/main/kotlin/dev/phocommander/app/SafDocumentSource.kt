package dev.phocommander.app

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import androidx.activity.result.contract.ActivityResultContracts

/**
 * The source that reads the documents of the directories the user released.
 *
 * WHY: the Storage Access Framework grants access to the directories that
 * the user selects, and the grant survives a restart of the application.
 * The source therefore keeps the identifiers of the released directories and
 * asks the system for their content every time the panel is drawn.
 *
 * The permission for the whole storage of the device is a second way into
 * the same storage. The application works without it, and asks for it only
 * when the user asks for a directory that SAF does not offer.
 */
class SafDocumentSource(private val activity: Activity) {

    /** The directories that the user released, with their permission. */
    private val released = mutableMapOf<String, Boolean>()

    /**
     * The dialog of the system, in which the user selects a directory.
     *
     * WHY: the dialog of the system is the only place where the user may
     * release a directory. The application never asks for a permission on
     * its own, and it never walks the storage of the device without one.
     */
    private val picker = activity.registerForActivityResult(
        ActivityResultContracts.OpenDocumentTree(),
    ) { uri: Uri? ->
        if (uri == null) {
            return@registerForActivityResult
        }
        released[uri.toString()] = canWrite(uri)
        activity.contentResolver.takePersistableUriPermission(
            uri,
            Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION,
        )
    }

    /**
     * Asks the user for a directory.
     */
    fun requestDirectory() {
        picker.launch(null)
    }

    /**
     * The directories that the user released.
     */
    fun roots(): List<String> = released.keys.toList()

    /**
     * The documents of a directory.
     *
     * @param uri the identifier of the directory.
     */
    fun documentsOf(uri: String): List<DocumentRecord> {
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(
            Uri.parse(uri),
            DocumentsContract.Document.COLUMN_DOCUMENT_ID,
        )
        val records = mutableListOf<DocumentRecord>()
        activity.contentResolver.query(
            children,
            arrayOf(
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_MIME_TYPE,
            ),
            null,
            null,
            null,
        )?.use { cursor ->
            val idColumn = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val nameColumn = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val mimeColumn = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)
            while (cursor.moveToNext()) {
                val id = cursor.getString(idColumn) ?: continue
                val name = cursor.getString(nameColumn) ?: continue
                val mime = cursor.getString(mimeColumn) ?: ""
                val isFolder = mime == DocumentsContract.Document.MIME_TYPE_DIR
                records.add(
                    DocumentRecord(
                        id = id,
                        name = name,
                        isFolder = isFolder,
                        mayWrite = released[uri] == true,
                    ),
                )
            }
        }
        return records
    }

    /**
     * Whether the grant of the user allows writing.
     *
     * WHY: the permission of the system comes with the grant. A directory
     * that the user released for reading only stays read-only, and the
     * core refuses the operations that write.
     */
    private fun canWrite(uri: Uri): Boolean {
        val granted = activity.contentResolver.persistedUriPermissions
        return granted.any {
            it.uri == uri && it.isWritePermission
        }
    }
}
