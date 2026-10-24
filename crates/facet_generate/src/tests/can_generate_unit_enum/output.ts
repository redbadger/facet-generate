
/**
 * This is a comment.
 * Continued lovingly here
 */
export type Colors =
    | { kind: "Red" }
    | { kind: "Blue" }
    | {
        /** Green is a cool color */
        kind: "Green";
    };

export const colorsRed = (): Colors => ({ kind: "Red" });

export const colorsBlue = (): Colors => ({ kind: "Blue" });

/** Green is a cool color */
export const colorsGreen = (): Colors => ({ kind: "Green" });

export function matchColors<R>(value: Colors, cases: {
    Red: (v: Extract<Colors, { kind: "Red" }>) => R;
    Blue: (v: Extract<Colors, { kind: "Blue" }>) => R;
    Green: (v: Extract<Colors, { kind: "Green" }>) => R;
}): R {
    return cases[value.kind as Colors["kind"]](value as never);
}
