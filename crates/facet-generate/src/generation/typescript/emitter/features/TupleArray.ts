function serializeTupleArray<T>(
    value: T[],
    serializer: Serializer,
    serializeElement: (item: T, serializer: Serializer) => void,
): void {
    value.forEach((item) => {
        serializeElement(item, serializer);
    });
}

function deserializeTupleArray<T>(
    deserializer: Deserializer,
    size: number,
    deserializeElement: (deserializer: Deserializer) => T,
): T[] {
    const list: T[] = [];
    for (let i = 0; i < size; i++) {
        list.push(deserializeElement(deserializer));
    }
    return list;
}
