//
//  The C interface of the Rust core.
//
//  WHY: the shell of iPadOS is written in Swift, and the rules of the
//  product live in the Rust core. The library of Rust speaks a plain C
//  interface, so that the shell needs no generator of bindings.
//
//  The contract of the two functions is the same on every system that
//  loads the bridge: the answer of the listing is a string of lines, every
//  line holds six values separated by a tabulator, and the memory of the
//  answer is given back with `phocommander_bridge_free`.
//

#ifndef PHOCOMMANDER_BRIDGE_H
#define PHOCOMMANDER_BRIDGE_H

/// The rows of a folder, in the order of the panel.
///
/// @param documents The bytes of the documents of the folder: the
///   identifier, the name, whether it is a folder, and whether the user
///   may write to it — one line per document, separated by a tabulator.
/// @param count The number of the bytes.
/// @return The answer, which the caller gives back with
///   `phocommander_bridge_free`.
char *phocommander_bridge_listing(const unsigned char *documents, unsigned long count);

/// Gives the memory of an answer back.
///
/// @param answer The answer of `phocommander_bridge_listing`.
void phocommander_bridge_free(char *answer);

#endif /* PHOCOMMANDER_BRIDGE_H */
