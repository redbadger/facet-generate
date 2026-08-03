
export class NamedEmptyStruct {
    constructor () {
    }
}

export type Test =
    | { kind: "NamedEmptyStruct"; value: NamedEmptyStruct }
    | { kind: "AnonymousEmptyStruct" };

export const testNamedEmptyStruct = (value: NamedEmptyStruct): Test => ({ kind: "NamedEmptyStruct", value });

export const testAnonymousEmptyStruct = (): Test => ({ kind: "AnonymousEmptyStruct" });

export function matchTest<R>(value: Test, cases: {
    NamedEmptyStruct: (v: Extract<Test, { kind: "NamedEmptyStruct" }>) => R;
    AnonymousEmptyStruct: (v: Extract<Test, { kind: "AnonymousEmptyStruct" }>) => R;
}): R {
    return cases[value.kind as Test["kind"]](value as never);
}
