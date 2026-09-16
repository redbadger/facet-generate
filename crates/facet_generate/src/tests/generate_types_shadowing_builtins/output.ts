type bool = boolean;
type bytes = Uint8Array;
type int32 = number;
type Optional<T> = T | null;
type Seq<T> = T[];
type str = string;
type Tuple<T extends any[]> = T;
type uint64 = bigint;
type uint8 = number;

export type BoolResult =
    | { kind: "Ok"; value: bool }
    | { kind: "Err"; value: str };

export const boolResultOk = (value: bool): BoolResult => ({ kind: "Ok", value });

export const boolResultErr = (value: str): BoolResult => ({ kind: "Err", value });

export function matchBoolResult<R>(value: BoolResult, cases: {
    Ok: (v: Extract<BoolResult, { kind: "Ok" }>) => R;
    Err: (v: Extract<BoolResult, { kind: "Err" }>) => R;
}): R {
    return cases[value.kind as BoolResult["kind"]](value as never);
}

export class Delete {
    constructor (public key: str) {
    }
}

export class Exists {
    constructor (public key: str) {
    }
}

export class Get {
    constructor (public key: str) {
    }
}

export class Keys {
    constructor (public items: Seq<str>, public next_cursor: uint64) {
    }
}

export type KeysResult =
    | { kind: "Ok"; value: Keys }
    | { kind: "Err"; value: str };

export const keysResultOk = (value: Keys): KeysResult => ({ kind: "Ok", value });

export const keysResultErr = (value: str): KeysResult => ({ kind: "Err", value });

export function matchKeysResult<R>(value: KeysResult, cases: {
    Ok: (v: Extract<KeysResult, { kind: "Ok" }>) => R;
    Err: (v: Extract<KeysResult, { kind: "Err" }>) => R;
}): R {
    return cases[value.kind as KeysResult["kind"]](value as never);
}

export class ListKeys {
    constructor (public prefix: str, public cursor: uint64) {
    }
}

export class Set {
    constructor (public key: str, public value: bytes) {
    }
}

export class Store {
    constructor (public tags: Seq<str>, public entries: Map<str,str>, public blob: Seq<uint8>, public pair: Tuple<[int32, str]>) {
    }
}

export type ValueResult =
    | { kind: "Ok"; value: Optional<Seq<uint8>> }
    | { kind: "Err"; value: str };

export const valueResultOk = (value: Optional<Seq<uint8>>): ValueResult => ({ kind: "Ok", value });

export const valueResultErr = (value: str): ValueResult => ({ kind: "Err", value });

export function matchValueResult<R>(value: ValueResult, cases: {
    Ok: (v: Extract<ValueResult, { kind: "Ok" }>) => R;
    Err: (v: Extract<ValueResult, { kind: "Err" }>) => R;
}): R {
    return cases[value.kind as ValueResult["kind"]](value as never);
}
