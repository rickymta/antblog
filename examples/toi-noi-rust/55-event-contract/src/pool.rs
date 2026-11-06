//! Descriptor set của mọi schema và đọc/ghi động (nhị phân + proto3 JSON) bằng prost-reflect.
use anyhow::{Context, Result};
use prost::Message;
use prost_reflect::{
    DescriptorPool, DeserializeOptions, DynamicMessage, MessageDescriptor, SerializeOptions,
};
use serde_json::Value;
use std::sync::OnceLock;

/// Descriptor set do `protoc` dựng trong build.rs (có `--include_imports`).
pub const DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/schemas.binpb"));

fn pool() -> &'static DescriptorPool {
    static POOL: OnceLock<DescriptorPool> = OnceLock::new();
    POOL.get_or_init(|| DescriptorPool::decode(DESCRIPTOR_SET).expect("descriptor set hợp lệ"))
}

/// Tên đầy đủ của message theo tên schema: `s1` là hợp đồng thật, `sN…` là biến thể thí nghiệm.
pub fn message_name(schema: &str) -> String {
    if schema == "s1" {
        "toinoi.events.v1.LedgerEntryPosted".to_owned()
    } else {
        format!("toinoi.lab.{schema}.LedgerEntryPosted")
    }
}

/// Descriptor của `LedgerEntryPosted` trong schema đã cho.
pub fn descriptor(schema: &str) -> Result<MessageDescriptor> {
    pool()
        .get_message_by_name(&message_name(schema))
        .with_context(|| format!("không có schema {schema} trong descriptor set"))
}

/// Dạng proto3 JSON: `canonical` = int64 thành chuỗi (chuẩn); không canonical = int64 là số (JSON của loạt).
fn serialize_options(canonical: bool, emit_defaults: bool) -> SerializeOptions {
    SerializeOptions::new()
        .stringify_64_bit_integers(canonical)
        .skip_default_fields(!emit_defaults)
}

/// Bytes JSON của message, trường theo thứ tự số field (đúng như producer ghi lên Kafka).
pub fn to_json_bytes(
    msg: &DynamicMessage,
    canonical: bool,
    emit_defaults: bool,
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut ser = serde_json::Serializer::new(&mut out);
    msg.serialize_with_options(&mut ser, &serialize_options(canonical, emit_defaults))?;
    Ok(out)
}

/// Giá trị JSON của message (để so sánh theo đường dẫn).
pub fn to_json_value(msg: &DynamicMessage, canonical: bool, emit_defaults: bool) -> Result<Value> {
    Ok(serde_json::from_slice(&to_json_bytes(
        msg,
        canonical,
        emit_defaults,
    )?)?)
}

/// Đọc JSON vào message. `lenient = true` bỏ qua trường lạ (như `DiscardUnknown`, `IgnoreUnknownFields`).
pub fn from_json_bytes(
    desc: &MessageDescriptor,
    bytes: &[u8],
    lenient: bool,
) -> Result<DynamicMessage> {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let options = DeserializeOptions::new().deny_unknown_fields(!lenient);
    let msg = DynamicMessage::deserialize_with_options(desc.clone(), &mut de, &options)?;
    de.end()?;
    Ok(msg)
}

/// Đọc nhị phân vào message.
pub fn from_bin(desc: &MessageDescriptor, bytes: &[u8]) -> Result<DynamicMessage> {
    Ok(DynamicMessage::decode(desc.clone(), bytes)?)
}

/// Mã hóa nhị phân.
pub fn to_bin(msg: &DynamicMessage) -> Vec<u8> {
    msg.encode_to_vec()
}
