
indirect public enum KeywordEnum {
    case `default`
    case `case`
    case `switch`(String)
    case `where`(`in`: Int32, `default`: String)
}

/// A struct whose every field is a keyword in at least one target language.
/// Each language escapes only its own reserved words: `import` is escaped in
/// Swift and TypeScript but is a soft keyword in Kotlin, and `type` is
/// contextual everywhere, so both come through bare where they are legal.
public struct KeywordFields {
    public var `default`: String
    public var `in`: Int32
    public var `class`: Bool
    public var object: String
    public var `static`: Bool
    public var `let`: String
    public var when: Int32
    public var `is`: Bool
    public var fun: String
    public var `operator`: String
    public var `import`: String
    public var type: String
    public var function: String?
    /// A tuple field: the Swift plugin derives `whereField0` / `whereField1`
    /// locals from this name, which must stay unescaped.
    public var `where`: (Int32, String)

    public init(`default`: String, `in`: Int32, `class`: Bool, object: String, `static`: Bool, `let`: String, when: Int32, `is`: Bool, fun: String, `operator`: String, `import`: String, type: String, function: String?, `where`: (Int32, String)) {
        self.`default` = `default`
        self.`in` = `in`
        self.`class` = `class`
        self.object = object
        self.`static` = `static`
        self.`let` = `let`
        self.when = when
        self.`is` = `is`
        self.fun = fun
        self.`operator` = `operator`
        self.`import` = `import`
        self.type = type
        self.function = function
        self.`where` = `where`
    }
}

/// Newtype struct — its member is named `value`, a Kotlin soft keyword that
/// must not be escaped.
public struct KeywordNewType {
    public var value: String

    public init(value: String) {
        self.value = value
    }
}

/// Tuple struct — its members are named `field0`, `field1`, which are never
/// keywords.
public struct KeywordTuple {
    public var field0: String
    public var field1: Int32

    public init(field0: String, field1: Int32) {
        self.field0 = field0
        self.field1 = field1
    }
}
