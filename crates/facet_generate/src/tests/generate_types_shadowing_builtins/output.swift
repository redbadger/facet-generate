
indirect public enum BoolResult {
    case ok(Bool)
    case err(String)
}

public struct Delete {
    public var key: String

    public init(key: String) {
        self.key = key
    }
}

public struct Exists {
    public var key: String

    public init(key: String) {
        self.key = key
    }
}

public struct Get {
    public var key: String

    public init(key: String) {
        self.key = key
    }
}

public struct Keys {
    public var items: [String]
    public var nextCursor: UInt64

    public init(items: [String], nextCursor: UInt64) {
        self.items = items
        self.nextCursor = nextCursor
    }
}

indirect public enum KeysResult {
    case ok(Keys)
    case err(String)
}

public struct ListKeys {
    public var prefix: String
    public var cursor: UInt64

    public init(prefix: String, cursor: UInt64) {
        self.prefix = prefix
        self.cursor = cursor
    }
}

public struct Set {
    public var key: String
    public var value: [UInt8]

    public init(key: String, value: [UInt8]) {
        self.key = key
        self.value = value
    }
}

public struct Store {
    public var tags: Swift.Set<String>
    public var entries: [String: String]
    public var blob: [UInt8]
    public var pair: (Int32, String)

    public init(tags: Swift.Set<String>, entries: [String: String], blob: [UInt8], pair: (Int32, String)) {
        self.tags = tags
        self.entries = entries
        self.blob = blob
        self.pair = pair
    }
}

indirect public enum ValueResult {
    case ok([UInt8]?)
    case err(String)
}
