//! Snapshot tests for the C# emitter — **Bincode encoding**.
//!
//! Split into submodules by category for manageability: [`structs`],
//! [`enums`], [`collections`], [`pointers`].
//!
//! Generated types include `IFacetSerializable`/`IFacetDeserializable<T>`
//! interface implementations, `BincodeSerialize`/`BincodeDeserialize`
//! convenience methods, and inline serialization loops for collections.

mod collections;
mod enums;
mod pointers;
mod structs;
