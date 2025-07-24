export interface StructOnlyInTypeScript {
    field: string;
}

export interface Struct {
    only_in_typescript: string;
}

export type Enum = 
    | { OnlyInTypeScript: string };

