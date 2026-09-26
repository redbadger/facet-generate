type Optional<T> = T | null;
export type Uuid = string & { readonly __uuid: unique symbol };

export class Foo {
    constructor (public id: Uuid, public maybeId: Optional<Uuid>) {
    }
}
