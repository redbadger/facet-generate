import { Serializer, Deserializer } from "./serde";

export class Entry {
    constructor (public level: Level, public outcome: Outcome) {
    }

    public serialize(serializer: Serializer): void {
        serializeLevel(this.level, serializer);
        serializeOutcome(this.outcome, serializer);
    }

    static deserialize(deserializer: Deserializer): Entry {
        const level = deserializeLevel(deserializer);
        const outcome = deserializeOutcome(deserializer);
        return new Entry(level,outcome);
    }
}
