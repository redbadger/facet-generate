export type ExternallyTaggedEnum = 
    | { "Some Variant": {
    "Variant Field": boolean;
}};

export type InternallyTaggedEnum = 
    | { type: "Some Variant",
    "Variant Field": boolean;
};

export type AdjacentlyTaggedEnum = 
    | { name: "Some Variant", properties: {
    "Variant Field": boolean;
}};

