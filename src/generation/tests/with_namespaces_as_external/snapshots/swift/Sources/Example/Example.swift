import Other

public struct Child {
    public var external: Other.OtherParent

    public init(external: Other.OtherParent) {
        self.external = external
    }
}

indirect public enum Parent {
    case child(Child)
}
