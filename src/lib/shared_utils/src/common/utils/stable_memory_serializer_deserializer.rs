use ic_cdk::api::stable::{BufferedStableReader, BufferedStableWriter};
use serde::{de::DeserializeOwned, Serialize};
use std::cmp::min;
use std::error::Error;
use std::io::{Read, Write};

const WASM_PAGE_SIZE_BYTES: usize = 64 * 1024; // 64KB

pub fn serialize_to_stable_memory<S: Serialize>(
    state: S,
    buffer_size: usize,
) -> Result<(), impl Error> {
    let writer = BufferedStableWriter::new(buffer_size);
    serialize(state, writer)
}

pub fn deserialize_from_stable_memory<S: DeserializeOwned>(
    max_buffer_size: usize,
) -> Result<S, impl Error> {
    let stable_size = ic_cdk::api::stable::stable_size() as usize * WASM_PAGE_SIZE_BYTES;
    let buffer_size = min(max_buffer_size, stable_size);
    let reader = BufferedStableReader::new(buffer_size);
    deserialize(reader)
}

pub fn serialize<T, W>(value: T, writer: W) -> Result<(), impl Error>
where
    T: Serialize,
    W: Write,
{
    let mut serializer = rmp_serde::Serializer::new(writer).with_struct_map();
    value.serialize(&mut serializer).map(|_| ())
}

pub fn deserialize<T, R>(reader: R) -> Result<T, impl Error>
where
    T: DeserializeOwned,
    R: Read,
{
    let mut deserializer = rmp_serde::Deserializer::new(reader);
    T::deserialize(&mut deserializer)
}
