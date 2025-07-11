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
            url: "https://github.com/serde-rs/serde",
            from: "1.0.137"
        )
    ],
    targets: [
        .target(
            name: "Example",
            dependencies: ["Serde"]
        ),
    ]
)
