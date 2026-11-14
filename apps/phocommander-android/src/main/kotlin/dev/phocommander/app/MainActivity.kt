package dev.phocommander.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.remember

/**
 * The activity of the Android application of Phocommander.
 *
 * WHY: the shell of Android holds the window and the events of the system,
 * and the rules of the product live in the Rust core. The activity therefore
 * does three things and nothing else: it asks the system for the
 * directories the user released, it hands the documents to the core, and it
 * draws what the core answers.
 *
 * The permission for the whole storage of the device is not asked for here.
 * It is asked for only when the user asks for a directory that SAF does
 * not offer, and the request happens in [SafDocumentSource].
 */
class MainActivity : ComponentActivity() {

    /** The source that reads the documents of the released directories. */
    private lateinit var source: SafDocumentSource

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        source = SafDocumentSource(this)
        setContent {
            // WHY: the bridge is created once, because the library of Rust
            //       is loaded once and it holds the rules of the documents.
            val bridge = remember { DocumentBridge() }
            PhocommanderSurface(
                bridge = bridge,
                source = source,
                onRequestDirectory = { source.requestDirectory() },
                onExplainPermission = { PermissionExplanation.show(this) },
            )
        }
    }
}
