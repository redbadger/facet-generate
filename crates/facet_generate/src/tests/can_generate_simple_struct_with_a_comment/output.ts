export interface Location {
}

/** This is a comment. */
export interface Person {
    /** This is another comment */
    name: string;
    age: number;
    info?: string | null;
    emails: string[];
    location: Location;
}

