package dev.phocommander.app

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.provider.Settings

/**
 * The explanation of the permission of the whole storage.
 *
 * WHY: the application works without the permission for the whole storage
 * of the device: the user releases the directories he wants in the dialog
 * of the system, and the application sees exactly those. The permission
 * for the whole storage is needed only for the directories that SAF does
 * not offer, and the user has to know that before he gives it.
 *
 * The explanation therefore says what the permission does, what it does
 * not do, and what happens without it.
 */
object PermissionExplanation {

    /**
     * The title of the explanation.
     */
    const val TITLE: String = "Why the permission is needed"

    /**
     * The text of the explanation.
     *
     * WHY: the text names the directories that the permission reaches and
     * the ones that it does not. A user who reads this knows what he gives
     * away, and the store entry of the application says the same.
     */
    const val TEXT: String =
        "Phocommander works without this permission. You choose the " +
            "directories in the dialog of the system, and the application " +
            "sees exactly those.\n\n" +
            "The permission for the whole storage is needed only for the " +
            "places that the dialog of the system does not offer — the " +
            "internal storage of the application, for example.\n\n" +
            "The application sends nothing, collects nothing and asks for " +
            "no account. The permission is used for reading and writing " +
            "files and nothing else."

    /**
     * Shows the explanation and offers the request.
     *
     * @param activity the activity that asks.
     */
    fun show(activity: Activity) {
        val dialog = android.app.AlertDialog.Builder(activity)
            .setTitle(TITLE)
            .setMessage(TEXT)
            .setPositiveButton(android.R.string.ok) { dialog, _ -> dialog.dismiss() }
            .setNeutralButton("Show me") { _, _ -> openTheSettings(activity) }
            .create()
        dialog.show()
    }

    /**
     * Opens the page of the system that holds the permission.
     *
     * WHY: the request of the permission itself belongs to the system and
     * not to the application: the system shows what the permission reaches
     * and asks the user.
     */
    private fun openTheSettings(activity: Activity) {
        val intent = Intent(
            Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
            Uri.parse("package:" + activity.packageName),
        )
        try {
            activity.startActivity(intent)
        } catch (_: android.content.ActivityNotFoundException) {
            // WHY: not every device carries the page. The application goes
            //       on with the SAF mode, which needs no permission.
        }
    }
}
