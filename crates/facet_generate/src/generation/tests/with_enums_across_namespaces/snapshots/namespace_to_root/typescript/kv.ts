import type { Serializer, Deserializer } from "./serde";
import * as Example from "./example";

export class Entry {
    constructor (public level: Example.Level, public outcome: Example.Outcome) {
    }

    public serialize(serializer: Serializer): void {
        Example.serializeLevel(this.level, serializer);
        Example.serializeOutcome(this.outcome, serializer);
    }

    static deserialize(deserializer: Deserializer): Entry {
        const level = Example.deserializeLevel(deserializer);
        const outcome = Example.deserializeOutcome(deserializer);
        return new Entry(level,outcome);
    }
}
