import Foundation

struct NotVisibleInKotlin: Codable {
    var inner: UInt32

    init(inner: UInt32) {
        self.inner = inner
    }
}

struct NotVisibleInTypescript: Codable {
    var inner: UInt32

    init(inner: UInt32) {
        self.inner = inner
    }
}

enum EnumWithVariantsPerLanguage: String, Codable {
    case notVisibleInKotlin = "NotVisibleInKotlin"
    case notVisibleInTypescript = "NotVisibleInTypescript"
}
