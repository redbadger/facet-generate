type int32 = number;

export type SomeEnum =
    | { type: "A" }
    | { type: "C"; content: int32 };

export const someEnumA = (): SomeEnum => ({ type: "A" });

export const someEnumC = (value: int32): SomeEnum => ({ type: "C", content: value });

export function matchSomeEnum<R>(value: SomeEnum, cases: {
    A: (v: Extract<SomeEnum, { type: "A" }>) => R;
    C: (v: Extract<SomeEnum, { type: "C" }>) => R;
}): R {
    return cases[value.type as SomeEnum["type"]](value as never);
}
