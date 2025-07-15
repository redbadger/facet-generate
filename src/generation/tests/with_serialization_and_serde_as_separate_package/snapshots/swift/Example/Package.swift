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
    dependencies: [
        .package(
            path: "../Serde"
        )
    ],
    targets: [
        .target(
            name: "Example",
            dependencies: ["Serde"]
        ),
    ]
)
