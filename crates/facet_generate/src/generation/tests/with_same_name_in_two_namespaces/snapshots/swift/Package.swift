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
            dependencies: []
        ),
        .target(
            name: "B",
            dependencies: ["A"]
        ),
        .target(
            name: "Example",
            dependencies: ["A", "B"]
        ),
    ]
)
