import A

public struct Child {
    public var y: UInt8

    public init(y: UInt8) {
        self.y = y
    }
}

public struct Parent {
    public var first: A.Child
    public var second: Child

    public init(first: A.Child, second: Child) {
        self.first = first
        self.second = second
    }
}
