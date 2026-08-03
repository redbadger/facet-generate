/**
 * Copyright (c) Facebook, Inc. and its affiliates
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

import { Deserializer } from "./deserializer";

export abstract class BinaryDeserializer implements Deserializer {
  private static readonly BIG_64: bigint = BigInt(64);
  private static readonly textDecoder = new TextDecoder();
  public buffer: ArrayBuffer;
  public offset: number;

  constructor(data: Uint8Array) {
    // copies data to prevent outside mutation of buffer.
    this.buffer = new ArrayBuffer(data.length);
    new Uint8Array(this.buffer).set(data, 0);
    this.offset = 0;
  }

  private read(length: number): ArrayBuffer {
    const remaining = this.buffer.byteLength - this.offset;
    if (length > remaining) {
      throw new Error(
        `Unexpected end of input: tried to read ${length} byte(s) at offset ` +
          `${this.offset}, but only ${remaining} remain`,
      );
    }

    const bytes = this.buffer.slice(this.offset, this.offset + length);
    this.offset += length;
    return bytes;
  }

  abstract deserializeLen(): number;

  abstract deserializeVariantIndex(): number;

  abstract checkThatKeySlicesAreIncreasing(
    key1: [number, number],
    key2: [number, number],
  ): void;

  public deserializeStr(): string {
    const value = this.deserializeBytes();
    return BinaryDeserializer.textDecoder.decode(value);
  }

  public deserializeBytes(): Uint8Array {
    const len = this.deserializeLen();
    if (len < 0) {
      throw new Error("Length of a bytes array can't be negative");
    }
    return new Uint8Array(this.read(len));
  }

  public deserializeBool(): boolean {
    const bool = new Uint8Array(this.read(1))[0];
    return bool == 1;
  }

  public deserializeUnit(): null {
    return null;
  }

  public deserializeU8(): number {
    return new DataView(this.read(1)).getUint8(0);
  }

  public deserializeU16(): number {
    return new DataView(this.read(2)).getUint16(0, true);
  }

  public deserializeU32(): number {
    return new DataView(this.read(4)).getUint32(0, true);
  }

  public deserializeU64(): bigint {
    return new DataView(this.read(8)).getBigUint64(0, true);
  }

  public deserializeU128(): bigint {
    // both limbs are unsigned, so they combine without sign extension
    const low = this.deserializeU64();
    const high = this.deserializeU64();
    return low | (high << BinaryDeserializer.BIG_64);
  }

  public deserializeI8(): number {
    return new DataView(this.read(1)).getInt8(0);
  }

  public deserializeI16(): number {
    return new DataView(this.read(2)).getInt16(0, true);
  }

  public deserializeI32(): number {
    return new DataView(this.read(4)).getInt32(0, true);
  }

  public deserializeI64(): bigint {
    return new DataView(this.read(8)).getBigInt64(0, true);
  }

  public deserializeI128(): bigint {
    const low = BigInt.asUintN(64, this.deserializeI64());
    const high = BigInt.asUintN(64, this.deserializeI64());
    return BigInt.asIntN(128, low | (high << BinaryDeserializer.BIG_64));
  }

  public deserializeOptionTag(): boolean {
    return this.deserializeBool();
  }

  public getBufferOffset(): number {
    return this.offset;
  }

  public deserializeChar(): string {
    throw new Error("Method deserializeChar not implemented.");
  }

  public deserializeF32(): number {
    return new DataView(this.read(4)).getFloat32(0, true);
  }

  public deserializeF64(): number {
    return new DataView(this.read(8)).getFloat64(0, true);
  }
}
