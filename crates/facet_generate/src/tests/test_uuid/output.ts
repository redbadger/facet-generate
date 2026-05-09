type Optional<T> = T | null;

export class Foo {
    constructor (public id: Uuid, public maybeId: Optional<Uuid>) {
    }
}
