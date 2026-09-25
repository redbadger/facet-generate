import * as $json from "./serde/json";
type Optional<T> = T | null;
type str = string;

export type Uuid = string & { readonly __uuid: unique symbol };

export class StructWithUuid {
    constructor (public id: Uuid, public parent_id: Optional<Uuid>, public name: str) {
    }

    static toJson(value: StructWithUuid): $json.JsonValue {
        return {
            "id": value.id,
            "parent_id": value.parent_id,
            "name": value.name,
        };
    }

    static fromJson(json: unknown): StructWithUuid {
        const obj = $json.readObject(json, "StructWithUuid");
        return new StructWithUuid(
            $json.readUuid($json.field(obj, "id")) as Uuid,
            $json.readOption($json.field(obj, "parent_id"), (j0) => $json.readUuid(j0) as Uuid),
            $json.readStr($json.field(obj, "name")),
        );
    }

    static jsonSerialize(value: StructWithUuid): string {
        return $json.stringify(StructWithUuid.toJson(value));
    }

    static jsonDeserialize(text: string): StructWithUuid {
        return StructWithUuid.fromJson($json.parse(text));
    }
}
