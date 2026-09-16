type bool = boolean;
type int32 = number;
type Optional<T> = T | null;
type str = string;
type Tuple<T extends any[]> = T;

export type KeywordEnum =
    | { kind: "Default" }
    | { kind: "Case" }
    | { kind: "Switch"; value: str }
    | { kind: "Where"; in: int32; default: str };

export const keywordEnumDefault = (): KeywordEnum => ({ kind: "Default" });

export const keywordEnumCase = (): KeywordEnum => ({ kind: "Case" });

export const keywordEnumSwitch = (value: str): KeywordEnum => ({ kind: "Switch", value });

export const keywordEnumWhere = (in_: int32, default_: str): KeywordEnum => ({ kind: "Where", in: in_, default: default_ });

export function matchKeywordEnum<R>(value: KeywordEnum, cases: {
    Default: (v: Extract<KeywordEnum, { kind: "Default" }>) => R;
    Case: (v: Extract<KeywordEnum, { kind: "Case" }>) => R;
    Switch: (v: Extract<KeywordEnum, { kind: "Switch" }>) => R;
    Where: (v: Extract<KeywordEnum, { kind: "Where" }>) => R;
}): R {
    return cases[value.kind as KeywordEnum["kind"]](value as never);
}

/// A struct whose every field is a keyword in at least one target language.
/// Each language escapes only its own reserved words: `import` is escaped in
/// Swift and TypeScript but is a soft keyword in Kotlin, and `type` is
/// contextual everywhere, so both come through bare where they are legal.
export class KeywordFields {
    public default: str;
    public in: int32;
    public class: bool;
    public object: str;
    public static: bool;
    public let: str;
    public when: int32;
    public is: bool;
    public fun: str;
    public operator: str;
    public import: str;
    public type: str;
    public function: Optional<str>;
    public where: Tuple<[int32, str]>;

    constructor (default_: str, in_: int32, class_: bool, object: str, static_: bool, let_: str, when: int32, is: bool, fun: str, operator: str, import_: str, type: str, function_: Optional<str>, where: Tuple<[int32, str]>) {
        this.default = default_;
        this.in = in_;
        this.class = class_;
        this.object = object;
        this.static = static_;
        this.let = let_;
        this.when = when;
        this.is = is;
        this.fun = fun;
        this.operator = operator;
        this.import = import_;
        this.type = type;
        this.function = function_;
        this.where = where;
    }
}

/// Newtype struct — its member is named `value`, a Kotlin soft keyword that
/// must not be escaped.
export class KeywordNewType {
    constructor (public value: str) {
    }
}

/// Tuple struct — its members are named `field0`, `field1`, which are never
/// keywords.
export class KeywordTuple {
    constructor (public field0: str, public field1: int32) {
    }
}
