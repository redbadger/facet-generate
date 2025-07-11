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
            url: "https://github.com/example/other",
            from: "1.0.0"
        )
    ],
    targets: [
        .target(
            name: "Example",
            dependencies: ["Other"]
        ),
    ]
)
