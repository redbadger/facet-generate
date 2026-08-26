import * as Kit from "./kit";
type Seq<T> = T[];

export class FeatureView {
    constructor (public rows: Seq<Kit.Row>) {
    }
}
