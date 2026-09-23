// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "Example",
    products: [
        .library(
            name: "Example",
            targets: ["Example"]
        )
    ],
    targets: [
        .target(
            name: "A",
            dependencies: ["B", "Serde"]
        ),
        .target(
            name: "B",
            dependencies: ["Serde"]
        ),
        .target(
            name: "Example",
            dependencies: ["A", "B", "Serde"]
        ),
        .target(
            name: "Serde",
            dependencies: []
        ),
    ]
)
