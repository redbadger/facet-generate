// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "Example",
    products: [
        .library(
            name: "Example",
            targets: ["Example"])
    ],
    targets: [
		.target(name: "Example", dependencies: ["Serde"]),
		.target(name: "Serde", dependencies: []),
	]
)
