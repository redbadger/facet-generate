
export class NamedEmptyStruct {
    constructor () {
    }
}

export type Test =
    | { type: "NamedEmptyStruct"; content: NamedEmptyStruct }
    | { type: "AnonymousEmptyStruct" }
    | { type: "NoStruct" };

export const testNamedEmptyStruct = (value: NamedEmptyStruct): Test => ({ type: "NamedEmptyStruct", content: value });

export const testAnonymousEmptyStruct = (): Test => ({ type: "AnonymousEmptyStruct" });

export const testNoStruct = (): Test => ({ type: "NoStruct" });

export function matchTest<R>(value: Test, cases: {
    NamedEmptyStruct: (v: Extract<Test, { type: "NamedEmptyStruct" }>) => R;
    AnonymousEmptyStruct: (v: Extract<Test, { type: "AnonymousEmptyStruct" }>) => R;
    NoStruct: (v: Extract<Test, { type: "NoStruct" }>) => R;
}): R {
    return cases[value.type as Test["type"]](value as never);
}
