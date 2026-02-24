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
            name: "Example",
            dependencies: ["Other", "Serde"]
        ),
        .target(
            name: "Other",
            dependencies: ["Serde"]
        ),
    ]
)
