package dev.phocommander.app

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

/**
 * The surface of the application.
 *
 * WHY: the panels stand below each other on a phone, because a finger
 * reaches one panel and not two at the same time. The rule of the product
 * is the same on every platform — the context decides where the decision
 * of the user cannot exist — and the Rust core knows it (FR-106).
 *
 * Every target that a finger reaches is forty-four points high. The rule
 * is the rule of the specification, and it is the same in the Rust core
 * and in the shell.
 */
@Composable
fun PhocommanderSurface(
    bridge: DocumentBridge,
    source: SafDocumentSource,
    onRequestDirectory: () -> Unit,
    onExplainPermission: () -> Unit,
) {
    var roots by remember { mutableStateOf(source.roots()) }
    var current by remember { mutableStateOf(roots.firstOrNull()) }
    var rows by remember { mutableStateOf(emptyList<ListingRow>()) }
    var openDocument by remember { mutableStateOf<String?>(null) }

    // WHY: the list of the panel is read from the source and ordered by
    //       the core. The shell draws what it gets and does not decide the
    //       order itself.
    fun refresh() {
        val uri = current ?: return
        rows = bridge.listing(source.documentsOf(uri))
    }

    MaterialTheme {
        Surface(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.fillMaxWidth()) {
                // The bar of the actions: the rows are forty-four points
                // high, and a button is a target of the same size.
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 44.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Button(
                        onClick = {
                            onRequestDirectory()
                            roots = source.roots()
                            current = roots.firstOrNull()
                            refresh()
                        },
                        modifier = Modifier.heightIn(min = 44.dp),
                    ) {
                        Text("Directory")
                    }
                    Button(
                        onClick = onExplainPermission,
                        modifier = Modifier.heightIn(min = 44.dp),
                    ) {
                        Text("Permission")
                    }
                }
                HorizontalDivider()

                // The list of the panel.
                LazyColumn(modifier = Modifier.fillMaxWidth()) {
                    items(rows) { row ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 44.dp)
                                .clickable {
                                    if (row.isFolder) {
                                        current = row.id
                                        refresh()
                                    } else {
                                        openDocument = row.name
                                    }
                                }
                                .padding(horizontal = 12.dp, vertical = 10.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = row.name,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }

                // The viewer: it opens below the list and takes the room it
                // needs (FR-50).
                val name = openDocument
                if (name != null) {
                    HorizontalDivider()
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(12.dp),
                    ) {
                        Text(name, style = MaterialTheme.typography.titleMedium)
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "",
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(120.dp)
                                .background(Color(0xFF202020)),
                        )
                    }
                }
            }
        }
    }
}
