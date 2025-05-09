// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.
// Swift Package: AcurastBenchmark

import PackageDescription;

let package = Package(
    name: "AcurastBenchmark",
    platforms: [
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "AcurastBenchmark",
            targets: ["AcurastBenchmark"]
        )
    ],
    dependencies: [ ],
    targets: [
        .binaryTarget(name: "AcurastBenchmarkFFI", path: "./AcurastBenchmarkFFI.xcframework"),
        .target(
            name: "AcurastBenchmark",
            dependencies: [
                .target(name: "AcurastBenchmarkFFI")
            ],
            path: "Sources/AcurastBenchmark"
        ),
        .testTarget(
            name: "AcurastBenchmarkTests",
            dependencies: ["AcurastBenchmark"]
        )
    ]
)
