function serializeOption<T>(
    value: T | null,
    serializer: Serializer,
    serializeElement: (value: T, serializer: Serializer) => void,
): void {
    if (value !== null) {
        serializer.serializeOptionTag(true);
        serializeElement(value, serializer);
    } else {
        serializer.serializeOptionTag(false);
    }
}

function deserializeOption<T>(
    deserializer: Deserializer,
    deserializeElement: (deserializer: Deserializer) => T,
): T | null {
    const tag = deserializer.deserializeOptionTag();
    if (!tag) {
        return null;
    } else {
        return deserializeElement(deserializer);
    }
}
