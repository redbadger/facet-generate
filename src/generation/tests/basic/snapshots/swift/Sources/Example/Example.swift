import Serde

public struct Child: Hashable {
    @Indirect public var name: String

    public init(name: String) {
        self.name = name
    }
}

indirect public enum Parent: Hashable {
    case child(Child)
}
