// swift-tools-version: 6.1
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Acubench",
    platforms: [
        .iOS(.v15),
        .macOS(.v10_15)
    ],
    products: [
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "Acubench",
            targets: ["Acubench"]),
    ],
    targets: [
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .binaryTarget(
            name: "AcubenchFFI",
            path: "AcubenchFFI.xcframework"
        ),
        .target(
            name: "Acubench",
            dependencies: ["AcubenchFFI"],
            resources: [
                .process("Resources")
            ]
        ),
        .testTarget(
            name: "AcubenchTests",
            dependencies: ["Acubench"],
        ),
//        .plugin(
//            name: "RustBuilder",
//            capability: .buildTool()
//        )
    ]
)
