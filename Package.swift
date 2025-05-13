// swift-tools-version: 6.1
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let exclude: [String] = [
    "../../android",
    "../../rust",
    "../../jitpack.yml",
]

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
            path: "ios/AcubenchFFI.xcframework"
        ),
        .target(
            name: "Acubench",
            dependencies: ["AcubenchFFI"],
            path: "ios/Sources/Acubench",
            exclude: exclude
        ),
        .testTarget(
            name: "AcubenchTests",
            dependencies: ["Acubench"],
            path: "ios/Tests/AcubenchTests",
            exclude: exclude
        ),
        // .plugin(
        //     name: "RustBuilder",
        //     capability: .buildTool(),
        //     path: "Plugins/RustBuilder",
        //     exclude: exclude
        // )
    ]
)
