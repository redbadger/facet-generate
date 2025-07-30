import Other
import Serde

public struct Child: Hashable {
    @Indirect public var external: Other.OtherParent

    public init(external: Other.OtherParent) {
        self.external = external
    }
}

indirect public enum Parent: Hashable {
    case child(Child)
}
