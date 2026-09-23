// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "Example",
    products: [
        .library(
            name: "Example",
            targets: ["Kv"]
        )
    ],
    targets: [
        .target(
            name: "Example",
            dependencies: ["Serde"]
        ),
        .target(
            name: "Kv",
            dependencies: ["Example", "Serde"]
        ),
        .target(
            name: "Serde",
            dependencies: []
        ),
    ]
)
