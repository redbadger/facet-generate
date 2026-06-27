
export type Test =
    | { type: "AnonymousEmptyStruct" }
    | { type: "NoStruct" };

export const testAnonymousEmptyStruct = (): Test => ({ type: "AnonymousEmptyStruct" });

export const testNoStruct = (): Test => ({ type: "NoStruct" });

export function matchTest<R>(value: Test, cases: {
    AnonymousEmptyStruct: (v: Extract<Test, { type: "AnonymousEmptyStruct" }>) => R;
    NoStruct: (v: Extract<Test, { type: "NoStruct" }>) => R;
}): R {
    return cases[value.type as Test["type"]](value as never);
}
