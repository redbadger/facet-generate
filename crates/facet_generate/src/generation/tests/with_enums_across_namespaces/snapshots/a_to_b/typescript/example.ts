import { Serializer, Deserializer } from "./serde";
import * as A from "./a";

export class App {
    constructor (public row: A.Row) {
    }

    public serialize(serializer: Serializer): void {
        this.row.serialize(serializer);
    }

    static deserialize(deserializer: Deserializer): App {
        const row = A.Row.deserialize(deserializer);
        return new App(row);
    }
}
