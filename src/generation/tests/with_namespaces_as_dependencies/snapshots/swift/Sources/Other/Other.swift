
public struct OtherChild: Hashable {
    @Indirect public var name: String

    public init(name: String) {
        self.name = name
    }
}

indirect public enum OtherParent: Hashable {
    case child(OtherChild)
}
