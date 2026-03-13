// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "Serde",
    products: [
        .library(
            name: "Serde",
            targets: ["Serde"]
        )
    ],
    targets: [
        .target(
            name: "Serde",
            dependencies: []
        ),
    ]
)
