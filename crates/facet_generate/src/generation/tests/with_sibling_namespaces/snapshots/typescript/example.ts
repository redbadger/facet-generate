import * as Feature from "./feature";

export type Root =
    | { kind: "Feature"; value: Feature.FeatureView };

export const rootFeature = (value: Feature.FeatureView): Root => ({ kind: "Feature", value });

export function matchRoot<R>(value: Root, cases: {
    Feature: (v: Extract<Root, { kind: "Feature" }>) => R;
}): R {
    return cases[value.kind as Root["kind"]](value as never);
}
