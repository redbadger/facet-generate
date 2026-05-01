//  Copyright (c) Facebook, Inc. and its affiliates.

import Serde
import XCTest

class SerdeTests: XCTestCase {
    func testSerializer() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u8(value: 255)
        try serializer.serialize_u32(value: 1)
        try serializer.serialize_u32(value: 1)
        try serializer.serialize_u32(value: 2)
        XCTAssertEqual(serializer.get_buffer_offset(), 13, "the buffer size should be same")
        XCTAssertEqual(
            serializer.get_bytes(), [255, 1, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0],
            "the array should be same")
    }

    func testDeserializer() throws {
        let deserializer = BincodeDeserializer(input: [1, 0, 0, 0])
        let result = try deserializer.deserialize_u32()
        XCTAssertEqual(result, 1, "should match")
    }

    func testSerializeUint8() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u8(value: 255)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u8()
        XCTAssertEqual(result, 255, "should be same")
    }

    func testSerializeUint16() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u16(value: 65535)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u16()
        XCTAssertEqual(result, 65535, "should be same")
    }

    func testSerializeUint32() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u32(value: 4_294_967_295)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u32()
        XCTAssertEqual(result, 4_294_967_295, "should be same")
    }

    func testSerializeInt8() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u8(value: 127)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u8()
        XCTAssertEqual(result, 127, "should be same")
    }

    func testSerializeInt16() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_i16(value: 32767)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_i16()
        XCTAssertEqual(result, 32767, "should be same")
    }

    func testSerializeInt32() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_i32(value: 2_147_483_647)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_i32()
        XCTAssertEqual(result, 2_147_483_647, "should be same")
    }

    func testSerializeInt64() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_i64(value: 9_223_372_036_854_775_807)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_i64()
        XCTAssertEqual(result, 9_223_372_036_854_775_807, "should be same")
    }

    // MARK: - UInt128

    func testSerializeU128() throws {
        // max: all bits set — low 8 bytes then high 8 bytes, both 0xFF
        let serMax = BincodeSerializer()
        try serMax.serialize_u128(value: UInt128(high: UInt64.max, low: UInt64.max))
        XCTAssertEqual(
            serMax.get_bytes(),
            [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255])

        // one: low word = 1, high word = 0
        let serOne = BincodeSerializer()
        try serOne.serialize_u128(value: UInt128(high: 0, low: 1))
        XCTAssertEqual(
            serOne.get_bytes(),
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

        // zero
        let serZero = BincodeSerializer()
        try serZero.serialize_u128(value: UInt128(high: 0, low: 0))
        XCTAssertEqual(
            serZero.get_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    func testDeserializeU128() throws {
        // max
        let desMax = BincodeDeserializer(
            input: [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255])
        let max = try desMax.deserialize_u128()
        XCTAssertEqual(max.low, UInt64.max)
        XCTAssertEqual(max.high, UInt64.max)

        // one
        let desOne = BincodeDeserializer(
            input: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        let one = try desOne.deserialize_u128()
        XCTAssertEqual(one.low, 1)
        XCTAssertEqual(one.high, 0)

        // zero
        let desZero = BincodeDeserializer(
            input: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        let zero = try desZero.deserialize_u128()
        XCTAssertEqual(zero.low, 0)
        XCTAssertEqual(zero.high, 0)
    }

    func testRoundTripU128() throws {
        let values: [UInt128] = [
            UInt128(high: 0, low: 0),
            UInt128(high: 0, low: 1),
            UInt128(high: 1, low: 0),
            UInt128(high: UInt64.max, low: UInt64.max),
            UInt128(high: 0xDEAD_BEEF_CAFE_BABE, low: 0x0102_0304_0506_0708),
        ]
        for value in values {
            let serializer = BincodeSerializer()
            try serializer.serialize_u128(value: value)
            let deserializer = BincodeDeserializer(input: serializer.get_bytes())
            let result = try deserializer.deserialize_u128()
            XCTAssertEqual(result.high, value.high, "high mismatch for \(value)")
            XCTAssertEqual(result.low, value.low, "low mismatch for \(value)")
        }
    }

    // MARK: - Int128

    func testSerializeI128() throws {
        // -1 in two's complement: all 128 bits set
        // high = Int64(-1), low = UInt64.max
        let serNeg1 = BincodeSerializer()
        try serNeg1.serialize_i128(value: Int128(high: -1, low: UInt64.max))
        XCTAssertEqual(
            serNeg1.get_bytes(),
            [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255])

        // +1: high = 0, low = 1
        let serPos1 = BincodeSerializer()
        try serPos1.serialize_i128(value: Int128(high: 0, low: 1))
        XCTAssertEqual(
            serPos1.get_bytes(),
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])

        // max: high = Int64.max, low = UInt64.max
        let serMax = BincodeSerializer()
        try serMax.serialize_i128(value: Int128(high: Int64.max, low: UInt64.max))
        XCTAssertEqual(
            serMax.get_bytes(),
            [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127])

        // min: high = Int64.min, low = 0
        let serMin = BincodeSerializer()
        try serMin.serialize_i128(value: Int128(high: Int64.min, low: 0))
        XCTAssertEqual(
            serMin.get_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80])
    }

    func testDeserializeI128() throws {
        // -1
        let desNeg1 = BincodeDeserializer(
            input: [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255])
        let neg1 = try desNeg1.deserialize_i128()
        XCTAssertEqual(neg1.low, UInt64.max)
        XCTAssertEqual(neg1.high, -1)

        // +1
        let desPos1 = BincodeDeserializer(
            input: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        let pos1 = try desPos1.deserialize_i128()
        XCTAssertEqual(pos1.low, 1)
        XCTAssertEqual(pos1.high, 0)

        // max
        let desMax = BincodeDeserializer(
            input: [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127])
        let max = try desMax.deserialize_i128()
        XCTAssertEqual(max.low, UInt64.max)
        XCTAssertEqual(max.high, Int64.max)

        // min
        let desMin = BincodeDeserializer(
            input: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80])
        let min = try desMin.deserialize_i128()
        XCTAssertEqual(min.low, 0)
        XCTAssertEqual(min.high, Int64.min)
    }

    func testRoundTripI128() throws {
        let values: [Int128] = [
            Int128(high: 0, low: 0),
            Int128(high: 0, low: 1),
            Int128(high: -1, low: UInt64.max),
            Int128(high: Int64.max, low: UInt64.max),
            Int128(high: Int64.min, low: 0),
            Int128(high: 0x1234_5678_9ABC_DEF0, low: 0xFEDC_BA98_7654_3210),
        ]
        for value in values {
            let serializer = BincodeSerializer()
            try serializer.serialize_i128(value: value)
            let deserializer = BincodeDeserializer(input: serializer.get_bytes())
            let result = try deserializer.deserialize_i128()
            XCTAssertEqual(result.high, value.high, "high mismatch for \(value)")
            XCTAssertEqual(result.low, value.low, "low mismatch for \(value)")
        }
    }
}
