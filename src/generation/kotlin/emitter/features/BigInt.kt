typealias BigInteger = @Serializable(with = BigIntegerSerializer::class) BigInteger

private object BigIntegerSerializer : KSerializer<BigInteger> {
    override val descriptor =
        PrimitiveSerialDescriptor("java.math.BigInteger", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): BigInteger =
        when (decoder) {
            is JsonDecoder -> decoder.decodeJsonElement().jsonPrimitive.content.toBigInteger()
            else -> decoder.decodeString().toBigInteger()
        }

    override fun serialize(encoder: Encoder, value: BigInteger) =
        when (encoder) {
            is JsonEncoder -> encoder.encodeJsonElement(JsonUnquotedLiteral(value.toString()))
            else -> encoder.encodeString(value.toString())
        }
}
