function serializeMap<K, V>(
    value: Map<K, V>,
    serializer: Serializer,
    serializeEntry: (key: K, value: V, serializer: Serializer) => void,
): void {
    serializer.serializeLen(value.size);
    const offsets: number[] = [];
    for (const [k, v] of value.entries()) {
        offsets.push(serializer.getBufferOffset());
        serializeEntry(k, v, serializer);
    }
    serializer.sortMapEntries(offsets);
}

function deserializeMap<K, V>(
    deserializer: Deserializer,
    deserializeEntry: (deserializer: Deserializer) => [K, V],
): Map<K, V> {
    const length = deserializer.deserializeLen();
    const obj = new Map<K, V>();
    for (let i = 0; i < length; i++) {
        const [key, value] = deserializeEntry(deserializer);
        obj.set(key, value);
    }
    return obj;
}
