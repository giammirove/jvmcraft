use crate::runtime::*;
use color_eyre::eyre::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{env, io};
use walkdir::WalkDir;

#[allow(non_camel_case_types)]
pub type ju1 = u8;
#[allow(non_camel_case_types)]
pub type ju2 = u16;
#[allow(non_camel_case_types)]
pub type ju4 = u32;
#[allow(non_camel_case_types)]
pub type ju8 = u64;

static NONCE: AtomicU64 = AtomicU64::new(0);

pub fn generate_nonce() -> u64 {
    NONCE.fetch_add(1, Ordering::Relaxed)
}

#[macro_export]
macro_rules! notimpl {
    () => {
        panic!("not implemented")
    };
    ($msg: expr) => {
        panic!("not implemented: {}", $msg)
    };
    ($($arg:tt)*) => {
        panic!("not implemented: {}", format!($($arg)*))
    };
}

pub trait ParseInfo<T> {
    fn parse(bytes: &[u8]) -> Result<T>;
}

#[allow(dead_code)]
pub fn get_slice(v: &[u8], start: usize, size: usize) -> &[u8] {
    &v[start..(start + size)]
}

#[allow(dead_code)]
pub fn get_slice_arr(v: &[u8], start: usize, size: usize) -> &[u8] {
    &v[start..(start + size)]
}

#[allow(dead_code)]
pub fn ju2_from_bytes(bytes: &[u8]) -> Result<ju2> {
    Ok(u16::from_be_bytes(bytes.try_into()?) as ju2)
}

#[allow(dead_code)]
pub fn ju4_from_bytes(bytes: &[u8]) -> Result<ju4> {
    Ok(u32::from_be_bytes(bytes.try_into()?) as ju4)
}

#[inline]
fn sign_extend(data: u32, size: u32) -> i32 {
    assert!(size > 0 && size <= 32);
    ((data << (32 - size)) as i32) >> (32 - size)
}

#[inline]
pub fn sign_extend8(data: u8) -> i32 {
    sign_extend(data.into(), 8)
}

#[inline]
pub fn sign_extend16(data: u16) -> i32 {
    sign_extend(data.into(), 16)
}

#[inline]
pub fn get_default_value(class_name: &str) -> types::Type {
    match class_name {
        "Z" => types::Type::Boolean(false),
        "C" => types::Type::Character(0),
        "F" => types::Type::Float(0.0),
        "D" => types::Type::Double(0.0),
        "B" => types::Type::Byte(0),
        "S" => types::Type::Short(0),
        "I" => types::Type::Integer(0),
        "J" => types::Type::Long(0),
        _ if class_name.contains("/") => types::Type::Null,
        _ if class_name.starts_with("[") => types::Type::Null,
        _ => panic!("def value for class not found {:?}", class_name),
    }
}

#[inline]
#[allow(dead_code)]
pub(crate) fn pause() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

pub fn get_return_type_descriptor(desc: &str) -> String {
    descriptor_to_class_name(desc.split(')').nth(1).unwrap())
}

fn extract_parameters(desc: &str) -> Option<&str> {
    let open = desc.find('(')?;
    let close = desc.find(')')?;
    Some(&desc[open + 1..close])
}
fn parse_parameter_types(params: &str) -> Vec<&str> {
    let mut types = vec![];
    let chars: Vec<char> = params.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let start = i;
        let ch = chars[i];

        match ch {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' => {
                types.push(&params[i..=i]);
                i += 1;
            }
            'L' => {
                // Object type: Ljava/lang/String;
                i += 1;
                while i < chars.len() && chars[i] != ';' {
                    i += 1;
                }
                i += 1; // include ;
                types.push(&params[start..i]);
            }
            '[' => {
                // Array: can be multiple '[' + type
                while i < chars.len() && chars[i] == '[' {
                    i += 1;
                }
                if i < chars.len() && chars[i] == 'L' {
                    i += 1;
                    while i < chars.len() && chars[i] != ';' {
                        i += 1;
                    }
                    i += 1;
                } else {
                    i += 1;
                }
                types.push(&params[start..i]);
            }
            _ => break,
        }
    }

    types
}
pub(crate) fn descriptor_to_class_name(desc: &str) -> String {
    match desc {
        d if d.starts_with('L') && d.ends_with(';') => d[1..d.len() - 1].to_string(),
        d if d.starts_with('[') => descriptor_to_class_name(&d[1..]).to_string(),
        _ => desc.to_string(),
    }
}
pub(crate) fn get_argument_class_names(descriptor: &str) -> Option<Vec<String>> {
    let param_str = extract_parameters(descriptor)?;
    let types = parse_parameter_types(param_str);
    Some(types.into_iter().map(descriptor_to_class_name).collect())
}

pub(crate) fn get_index_scale(class_name: &str) -> u32 {
    match class_name {
        "[B" => 1,
        "[Z" => 1,
        "[S" => 2,
        "[C" => 2,
        "[I" => 4,
        "[F" => 4,
        "[J" => 8,
        "[D" => 8,
        _ if class_name.starts_with("[L") || class_name.starts_with("[[") => {
            // Reference types (arrays of objects)
            4 // or 8 depending on your JVM pointer width
        }
        _ => panic!("{}", class_name), // Not an array
    }
}

pub(crate) fn get_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| "".to_string())
}

// get relative paths of files from root
pub(crate) fn get_class_names_in_module(root: &str) -> Vec<String> {
    let mut files = vec![];
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let real_path = path.strip_prefix(root).unwrap().to_str().unwrap();
        files.push(real_path.replace(".class", "").to_string());
    }
    files
}
