
public struct Child {
    public var name: String

    public init(name: String) {
        self.name = name
    }
}

indirect public enum Parent {
    case child(Child)
}
