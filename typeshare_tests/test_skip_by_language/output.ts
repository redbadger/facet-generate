export interface NotVisibleInSwift {
    inner: number;
}

export interface NotVisibleInKotlin {
    inner: number;
}

export enum EnumWithVariantsPerLanguage {
    NotVisibleInSwift = "NotVisibleInSwift",
    NotVisibleInKotlin = "NotVisibleInKotlin",
}

