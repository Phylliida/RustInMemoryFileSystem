
use std::io::{Cursor, Read};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::io::Write;
use crate::filesystem::{QID,UInt8Array};
use std::str::from_utf8;

pub fn marshall_u8(val: u8, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u8(val);
    return cursor.position();
}

pub fn unmarshall_u8(data: &[u8], offset: u64) -> (u8, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u8();
    return (result.unwrap(), cursor.position());
}

pub fn marshall_u16(val: u16, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u16::<BigEndian>(val);
    return cursor.position();
}

pub fn unmarshall_u16(data: &[u8], offset: u64) -> (u16, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u16::<BigEndian>();
    return (result.unwrap(), cursor.position());
}

pub fn marshall_u32(val: u32, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u32::<BigEndian>(val);
    return cursor.position();
}

pub fn unmarshall_u32(data: &[u8], offset: u64) -> (u32, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u32::<BigEndian>();
    return (result.unwrap(), cursor.position());
}

pub fn marshall_u64(val: u64, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u64::<BigEndian>(val);
    return cursor.position();
}

pub fn unmarshall_u64(data: &[u8], offset: u64) -> (u64, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u64::<BigEndian>();
    return (result.unwrap(), cursor.position());
}

pub fn marshall_string(val: &str, data: &mut [u8], offset: u64) -> u64 {
    // write string length
    let as_bytes: &[u8] = val.as_bytes();
    let new_offset = marshall_u16(as_bytes.len() as u16, data, offset);
    let mut cursor = Cursor::new(data);
    cursor.set_position(new_offset);
    let _ = cursor.write_all(as_bytes);
    return cursor.position();
}

pub fn string_to_array(val: &str) -> UInt8Array {
    // write string length
    let as_bytes: &[u8] = val.as_bytes();
    let mut array = UInt8Array::new(as_bytes.len());
    array.data.copy_from_slice(as_bytes);
    return array;
} 

pub fn unmarshall_string(data: &[u8], offset: u64) -> (String, u64) {
    let (str_len, new_offset) = unmarshall_u16(data, offset);
    let mut cursor = Cursor::new(data);
    cursor.set_position(new_offset);
    let mut buffer = vec![0u8; str_len as usize];
    let result = cursor.read_exact(&mut buffer);
    debug_assert!(result.is_ok(), "Not enough bytes to read data");
    return (from_utf8(&buffer).unwrap().to_owned(), cursor.position());
}


pub fn marshall_qid(val: &QID, data: &mut [u8], offset: u64) -> u64 {
    let mut offset_tmp = offset;
    offset_tmp = marshall_u8(val.r#type, data, offset_tmp);
    offset_tmp = marshall_u32(val.version, data, offset_tmp);
    offset_tmp = marshall_u64(val.path, data, offset_tmp);
    return offset_tmp;
}

pub fn unmarshall_qid(data: &[u8], offset: u64) -> (QID, u64) {
    let offset_tmp = offset;
    let (r#type, offset_tmp2) = unmarshall_u8(data, offset_tmp);
    let (version, offset_tmp3) = unmarshall_u32(data, offset_tmp2);
    let (path, offset_tmp4) = unmarshall_u64(data, offset_tmp3);
    let res = QID {
        r#type: r#type,
        version: version,
        path: path
    };
    return (res, offset_tmp4);
}


pub fn bytes_to_array(data: &[u8]) -> UInt8Array {
    let mut result = UInt8Array::new(data.len());
    result.data.copy_from_slice(data);
    return result;
}