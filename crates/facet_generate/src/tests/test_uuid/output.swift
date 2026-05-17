
public struct Foo {
    public var id: UUID
    public var maybeId: UUID?

    public init(id: UUID, maybeId: UUID?) {
        self.id = id
        self.maybeId = maybeId
    }
}
