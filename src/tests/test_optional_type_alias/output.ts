export type OptionalU32 = number | undefined | null;

export type OptionalU16 = number | undefined | null;

export interface FooBar {
    foo: OptionalU32;
    bar: OptionalU16;
}

