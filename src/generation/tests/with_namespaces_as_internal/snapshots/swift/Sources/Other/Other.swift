
public struct OtherChild {
    public var name: String

    public init(name: String) {
        self.name = name
    }
}

indirect public enum OtherParent {
    case child(Other.OtherChild)
}
