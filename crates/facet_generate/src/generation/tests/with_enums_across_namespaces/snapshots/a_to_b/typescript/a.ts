import { Serializer, Deserializer } from "./serde";
import * as B from "./b";

export class Row {
    constructor (public status: B.Status, public signal: B.Signal) {
    }

    public serialize(serializer: Serializer): void {
        B.serializeStatus(this.status, serializer);
        B.serializeSignal(this.signal, serializer);
    }

    static deserialize(deserializer: Deserializer): Row {
        const status = B.deserializeStatus(deserializer);
        const signal = B.deserializeSignal(deserializer);
        return new Row(status,signal);
    }
}
