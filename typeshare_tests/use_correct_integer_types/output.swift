import Foundation

/// This is a comment.
struct Foo: Codable {
    var a: Int8
    var b: Int16
    var c: Int32
    var e: UInt8
    var f: UInt16
    var g: UInt32

    init(a: Int8, b: Int16, c: Int32, e: UInt8, f: UInt16, g: UInt32) {
        self.a = a
        self.b = b
        self.c = c
        self.e = e
        self.f = f
        self.g = g
    }
}
