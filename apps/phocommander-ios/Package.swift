// swift-tools-version:5.9
// The Swift package of the iPadOS application of Phocommander.
//
// WHY: the shell of iPadOS is written in Swift, and the rules of the
//       product live in the Rust core. The library of Rust is built by
//       Cargo and linked into the application of the package.
//
// The build of the library needs the Rust toolchain of the machine that
// builds; the package itself needs nothing but Swift.

import PackageDescription

let package = Package(
    name: "Phocommander",
    platforms: [
        .iOS(.v17),
    ],
    products: [
        .library(name: "Phocommander", targets: ["Phocommander"]),
    ],
    targets: [
        // The header of the C interface of the Rust core. The library
        // itself is built by Cargo and linked into the application of the
        // Xcode project; the header tells the shell what it may call.
        .target(
            name: "PhocommanderBridge",
            path: "Sources/PhocommanderBridge"
        ),
        .target(
            name: "Phocommander",
            dependencies: ["PhocommanderBridge"],
            path: "Sources/Phocommander"
        ),
    ]
)
