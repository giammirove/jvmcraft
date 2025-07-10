use core::fmt;

use color_eyre::eyre::{eyre, Result};
use log::debug;

use crate::{notimpl, utils::*};

#[derive(Default, Debug, Clone)]

enum CpInfoTagEnum {
  #[default]
  None = 0,
  Class = 7,
  Fieldref = 9,
  Methodref = 10,
  Interfaceref = 11,
  String = 8,
  Integer = 3,
  Float = 4,
  Long = 5,
  Double = 6,
  NameAndType = 12,
  Utf8 = 1,
  MethodHandle = 15,
  MethodType = 16,
  Dynamic = 17,
  InvokeDynamic = 18,
  Module = 19,
  Package = 2,
}

impl CpInfoTagEnum {
  fn from_num(tag: ju1) -> Result<CpInfoTagEnum> {
    match tag {
      1 => Ok(CpInfoTagEnum::Utf8),
      2 => Ok(CpInfoTagEnum::Package),
      3 => Ok(CpInfoTagEnum::Integer),
      4 => Ok(CpInfoTagEnum::Float),
      5 => Ok(CpInfoTagEnum::Long),
      6 => Ok(CpInfoTagEnum::Double),
      7 => Ok(CpInfoTagEnum::Class),
      8 => Ok(CpInfoTagEnum::String),
      9 => Ok(CpInfoTagEnum::Fieldref),
      10 => Ok(CpInfoTagEnum::Methodref),
      11 => Ok(CpInfoTagEnum::Interfaceref),
      12 => Ok(CpInfoTagEnum::NameAndType),
      // 13,14
      15 => Ok(CpInfoTagEnum::MethodHandle),
      16 => Ok(CpInfoTagEnum::MethodType),
      17 => Ok(CpInfoTagEnum::Dynamic),
      18 => Ok(CpInfoTagEnum::InvokeDynamic),
      19 => Ok(CpInfoTagEnum::Module),
      _ => Err(eyre!("cp info tag not handled: {:?}", tag)),
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct RefInfo {
  class_index: ju2,
  name_and_type_index: ju2,
}

impl RefInfo {
  pub(crate) fn get_class_index(&self) -> ju2 {
    self.class_index
  }

  pub(crate) fn _get_name_and_type_index(&self) -> ju2 {
    self.name_and_type_index
  }
}

#[derive(Debug, Clone)]
pub(crate) struct StringInfo {
  string_index: ju2,
}

impl StringInfo {
  pub(crate) fn get_string_index(&self) -> ju2 {
    self.string_index
  }
}

#[derive(Debug, Clone)]
pub(crate) struct NumericInfo {
  bytes: ju4,
}

impl NumericInfo {
  pub(crate) fn int(&self) -> i32 {
    self.bytes as i32
  }

  pub(crate) fn float(&self) -> f32 {
    f32::from_bits(self.bytes)
  }
}

#[derive(Debug, Clone)]
pub(crate) struct BigNumericInfo {
  high_bytes: ju4,
  low_bytes: ju4,
}

impl BigNumericInfo {
  pub(crate) fn value(&self) -> ju8 {
    ((self.high_bytes as u64) << 32) + self.low_bytes as u64
  }
}

#[derive(Debug, Clone)]
pub(crate) struct NameAndTypeInfo {
  name_index: ju2,
  descriptor_index: ju2,
}

#[derive(Debug, Clone)]
pub(crate) struct Utf8Info {
  length: ju2,
  bytes: Vec<ju1>, // of length `length`
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct MethodHandleInfo {
  reference_kind: ju1,  // Ref kind (like Ref_invokeStatic)
  reference_index: ju2, // reference in ConstantPool to MethodRef
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct MethodTypeInfo {
  descriptor_index: ju2,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DynamicInfo {
  bootstrap_method_attr_index: ju2,
  name_and_type_index: ju2,
}

#[derive(Debug, Clone)]
pub(crate) struct InvokeDynamicInfo {
  bootstrap_method_attr_index: ju2,
  name_and_type_index: ju2,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ModuleInfo {
  name_index: ju2,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct PackageInfo {
  name_index: ju2,
}

#[derive(Debug, Clone)]
pub(crate) struct ClassInfo {
  name_index: ju2,
}

impl ClassInfo {
  pub(crate) fn get_name_index(&self) -> ju2 {
    self.name_index
  }
}

#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum CpInfoInfoEnum {
  #[default]
  None,
  Fieldref(RefInfo),
  Methodref(RefInfo),
  Interfaceref(RefInfo),
  // ============================================== //
  String(StringInfo),
  // ============================================== //
  Integer(NumericInfo),
  Float(NumericInfo),
  // ============================================== //
  Long(BigNumericInfo),
  Double(BigNumericInfo),
  // ============================================== //
  NameAndType(NameAndTypeInfo),
  // ============================================== //
  Utf8(Utf8Info),
  // ============================================== //
  MethodHandle(MethodHandleInfo),
  // ============================================== //
  MethodType(MethodTypeInfo),
  // ============================================== //
  Dynamic(DynamicInfo),
  InvokeDynamic(InvokeDynamicInfo),
  // ============================================== //
  _Module(ModuleInfo),
  // ============================================== //
  Package(PackageInfo),
  // ============================================== //
  Class(ClassInfo),
}

impl fmt::Display for CpInfoInfoEnum {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Utf8Info {
  fn _print(&self) {
    // TODO: alternative to clone
    match String::from_utf8(self.bytes.clone()) {
      Ok(string) => debug!("{}", string),
      Err(e) => debug!("Error: {}", e),
    }
  }

  fn name(&self) -> String {
    // TODO: alternative to clone
    match String::from_utf8(self.bytes.clone()) {
      Ok(string) => string,
      Err(_) => "".to_string(),
    }
  }
}

impl ParseInfo<Utf8Info> for Utf8Info {
  fn parse(bytes: &[u8]) -> Result<Utf8Info> {
    let length: usize = u16::from_be_bytes(bytes[0..2].try_into()?) as usize;

    let mut data: Vec<ju1> = vec![];

    for i in 0..length {
      data.push(bytes[i + 2]);
    }

    Ok(Utf8Info {
      length: length as ju2,
      bytes: data,
    })
  }
}

impl ParseInfo<NumericInfo> for NumericInfo {
  fn parse(bytes: &[u8]) -> Result<NumericInfo> {
    let bytes = u32::from_be_bytes(bytes[0..4].try_into()?);

    Ok(NumericInfo { bytes })
  }
}

impl ParseInfo<BigNumericInfo> for BigNumericInfo {
  fn parse(bytes: &[u8]) -> Result<BigNumericInfo> {
    let high_bytes = u32::from_be_bytes(bytes[0..4].try_into()?);

    let low_bytes = u32::from_be_bytes(bytes[4..8].try_into()?);

    Ok(BigNumericInfo {
      high_bytes,
      low_bytes,
    })
  }
}

impl ParseInfo<StringInfo> for StringInfo {
  fn parse(bytes: &[u8]) -> Result<StringInfo> {
    let string_index = u16::from_be_bytes(bytes[0..2].try_into()?);

    Ok(StringInfo { string_index })
  }
}

impl ParseInfo<RefInfo> for RefInfo {
  fn parse(bytes: &[u8]) -> Result<RefInfo> {
    let class_index = u16::from_be_bytes(bytes[0..2].try_into()?);

    let name_and_type_index = u16::from_be_bytes(bytes[2..4].try_into()?);

    Ok(RefInfo {
      class_index,
      name_and_type_index,
    })
  }
}

impl ParseInfo<NameAndTypeInfo> for NameAndTypeInfo {
  fn parse(bytes: &[u8]) -> Result<NameAndTypeInfo> {
    let name_index = u16::from_be_bytes(bytes[0..2].try_into()?);

    let descriptor_index = u16::from_be_bytes(bytes[2..4].try_into()?);

    Ok(NameAndTypeInfo {
      name_index,
      descriptor_index,
    })
  }
}

impl ParseInfo<PackageInfo> for PackageInfo {
  fn parse(bytes: &[u8]) -> Result<PackageInfo> {
    let name_index = ju2_from_bytes(&bytes[0..2])?;

    Ok(PackageInfo { name_index })
  }
}

impl ParseInfo<ClassInfo> for ClassInfo {
  fn parse(bytes: &[u8]) -> Result<ClassInfo> {
    let name_index = ju2_from_bytes(&bytes[0..2])?;

    Ok(ClassInfo { name_index })
  }
}

impl ParseInfo<DynamicInfo> for DynamicInfo {
  fn parse(bytes: &[u8]) -> Result<DynamicInfo> {
    let bootstrap_method_attr_index = ju2_from_bytes(&bytes[0..2])?;

    let name_and_type_index = ju2_from_bytes(&bytes[2..4])?;

    Ok(DynamicInfo {
      bootstrap_method_attr_index,
      name_and_type_index,
    })
  }
}

impl ParseInfo<InvokeDynamicInfo> for InvokeDynamicInfo {
  fn parse(bytes: &[u8]) -> Result<InvokeDynamicInfo> {
    let bootstrap_method_attr_index = ju2_from_bytes(&bytes[0..2])?;

    let name_and_type_index = ju2_from_bytes(&bytes[2..4])?;

    Ok(InvokeDynamicInfo {
      bootstrap_method_attr_index,
      name_and_type_index,
    })
  }
}

impl ParseInfo<MethodTypeInfo> for MethodTypeInfo {
  fn parse(bytes: &[u8]) -> Result<MethodTypeInfo> {
    let descriptor_index = ju2_from_bytes(&bytes[0..2])?;

    Ok(MethodTypeInfo { descriptor_index })
  }
}

impl ParseInfo<MethodHandleInfo> for MethodHandleInfo {
  fn parse(bytes: &[u8]) -> Result<MethodHandleInfo> {
    let reference_kind = bytes[0];

    let reference_index = ju2_from_bytes(&bytes[1..3])?;

    Ok(MethodHandleInfo {
      reference_kind,
      reference_index,
    })
  }
}

impl MethodHandleInfo {
  pub(crate) fn get_reference_kind(&self) -> ju1 {
    self.reference_kind
  }

  pub(crate) fn get_reference_index(&self) -> ju2 {
    self.reference_index
  }
}

impl MethodTypeInfo {
  pub(crate) fn get_descriptor_index(&self) -> ju2 {
    self.descriptor_index
  }
}

impl CpInfoInfoEnum {
  fn from_tag(tag: &CpInfoTagEnum, bytes: &[u8]) -> Result<CpInfoInfoEnum> {
    let res = match tag {
      CpInfoTagEnum::Utf8 => CpInfoInfoEnum::Utf8(Utf8Info::parse(bytes)?),
      CpInfoTagEnum::Integer => CpInfoInfoEnum::Integer(NumericInfo::parse(bytes)?),
      CpInfoTagEnum::Float => CpInfoInfoEnum::Float(NumericInfo::parse(bytes)?),
      CpInfoTagEnum::Long => CpInfoInfoEnum::Long(BigNumericInfo::parse(bytes)?),
      CpInfoTagEnum::Double => CpInfoInfoEnum::Double(BigNumericInfo::parse(bytes)?),
      CpInfoTagEnum::String => CpInfoInfoEnum::String(StringInfo::parse(bytes)?),
      CpInfoTagEnum::Fieldref => CpInfoInfoEnum::Fieldref(RefInfo::parse(bytes)?),
      CpInfoTagEnum::Methodref => CpInfoInfoEnum::Methodref(RefInfo::parse(bytes)?),
      CpInfoTagEnum::Interfaceref => CpInfoInfoEnum::Interfaceref(RefInfo::parse(bytes)?),
      CpInfoTagEnum::NameAndType => CpInfoInfoEnum::NameAndType(NameAndTypeInfo::parse(bytes)?),
      CpInfoTagEnum::Package => CpInfoInfoEnum::Package(PackageInfo::parse(bytes)?),
      CpInfoTagEnum::Class => CpInfoInfoEnum::Class(ClassInfo::parse(bytes)?),
      CpInfoTagEnum::Dynamic => CpInfoInfoEnum::Dynamic(DynamicInfo::parse(bytes)?),
      CpInfoTagEnum::InvokeDynamic => {
        CpInfoInfoEnum::InvokeDynamic(InvokeDynamicInfo::parse(bytes)?)
      }
      CpInfoTagEnum::MethodType => CpInfoInfoEnum::MethodType(MethodTypeInfo::parse(bytes)?),
      CpInfoTagEnum::MethodHandle => CpInfoInfoEnum::MethodHandle(MethodHandleInfo::parse(bytes)?),
      _ => {
        notimpl!("CpInfo not recognized for tag: {:?}", tag)
      }
    };

    Ok(res)
  }

  // expressed in bytes
  fn get_size(&self) -> usize {
    // 1 to consider the tag
    1 + match self {
      CpInfoInfoEnum::Utf8(info) => 2 + (info.length as usize),
      CpInfoInfoEnum::Integer(_) => 4,
      CpInfoInfoEnum::Float(_) => 4,
      CpInfoInfoEnum::Long(_) => 8,
      CpInfoInfoEnum::Double(_) => 8,
      CpInfoInfoEnum::String(_) => 2,
      CpInfoInfoEnum::Fieldref(_) => 4,
      CpInfoInfoEnum::Methodref(_) => 4,
      CpInfoInfoEnum::Interfaceref(_) => 4,
      CpInfoInfoEnum::NameAndType(_) => 4,
      CpInfoInfoEnum::Package(_) => 2,
      CpInfoInfoEnum::Class(_) => 2,
      CpInfoInfoEnum::Dynamic(_) => 4,
      CpInfoInfoEnum::InvokeDynamic(_) => 4,
      CpInfoInfoEnum::MethodType(_) => 2,
      CpInfoInfoEnum::MethodHandle(_) => 3,
      _ => notimpl!("get size not impl for {:?}", self),
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct CpInfo {
  info: CpInfoInfoEnum,
}

impl CpInfo {
  fn parse(bytes: &[u8]) -> Result<(CpInfo, usize)> {
    let tag = CpInfoTagEnum::from_num(bytes[0])?;

    let info = CpInfoInfoEnum::from_tag(&tag, &bytes[1..])?;

    let size = info.get_size();

    let cpinfo = CpInfo { info };

    Ok((cpinfo, size))
  }

  pub(crate) fn get_info(&self) -> &CpInfoInfoEnum {
    &self.info
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ConstantPool {
  constant_pool_count: ju2,
  constant_pool: Vec<CpInfo>,
}

impl fmt::Display for ConstantPool {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "Constant Pool {}", self.constant_pool_count)?;

    for i in 0..self.constant_pool.len() {
      writeln!(
        f,
        "\t{} {:?}",
        i,
        self.constant_pool.get(i).unwrap().get_info()
      )?;
    }

    Ok(())
  }
}

impl ConstantPool {
  pub(crate) fn parse(bytes: &[u8]) -> Result<(ConstantPool, usize)> {
    debug!("[-] Parsing Constant Pools");

    let mut v: Vec<CpInfo> = vec![];

    let mut index: usize = 0;

    let count: usize = u16::from_be_bytes(bytes[0..2].try_into()?) as usize;

    debug!("    Found {:?} constant pools", count - 1);

    index += 2;

    // The constant_pool table is indexed from 1 to constant_pool_count - 1.
    let mut i = 1;

    while i < count {
      let slice = &bytes[index..];

      let (cpinfo, bytes_read) = CpInfo::parse(slice)?;

      index += bytes_read;

      // https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-4.html#jvms-4.4.5
      // ony valid for long and double
      let mut add_fake_entry = false;

      match cpinfo.info {
        CpInfoInfoEnum::Double(_) | CpInfoInfoEnum::Long(_) => {
          add_fake_entry = true;
        }
        _ => {}
      };

      i += 1;

      v.push(cpinfo);

      if add_fake_entry {
        i += 1;

        v.push(CpInfo {
          info: CpInfoInfoEnum::None,
        });
      }
    }

    Ok((
      ConstantPool {
        constant_pool_count: count as ju2,
        constant_pool: v,
      },
      index,
    ))
  }

  pub(crate) fn empty() -> ConstantPool {
    ConstantPool {
      constant_pool_count: 0,
      constant_pool: vec![],
    }
  }

  pub(crate) fn _print_name(&self, index: ju2) {
    match self.constant_pool.get((index - 1) as usize) {
      None => {
        debug!("[!] Index {:?} not found", index);
      }
      Some(val) => match &val.info {
        CpInfoInfoEnum::Utf8(info) => {
          print!("Resolved string at {:?} is: ", index);

          info._print()
        }
        _ => {
          debug!("[!] Index {:?} is not Utf8 but {:?}", index, val.info);
        }
      },
    }
  }

  pub(crate) fn resolve_index(&self, index: ju2) -> Result<&CpInfo> {
    match self.constant_pool.get((index.wrapping_sub(1)) as usize) {
      None => Err(eyre!(
        "{:?} not found in constant pool {:?}",
        index,
        self.constant_pool_count
      )),
      Some(val) => Ok(val),
    }
  }

  pub(crate) fn resolve_name(&self, index: ju2) -> Result<String> {
    let utf8 = self.resolve_index(index)?;

    match &utf8.info {
      CpInfoInfoEnum::Utf8(info) => Ok(info.name()),
      _ => Err(eyre!("{:?} is not a utf8 but is {:?}", index, utf8.info)),
    }
  }

  // given an index it returns the class associated with it
  pub(crate) fn resolve_class_name(&self, index: ju2) -> Result<String> {
    if index == 0 {
      return Ok("".to_string());
    }

    let class: &CpInfo = self.resolve_index(index)?;

    match &class.info {
      CpInfoInfoEnum::Class(info) => {
        if info.name_index == 0 {
          return Ok("".to_string());
        }

        self.resolve_name(info.name_index)
      }
      _ => Err(eyre!("[!] Index {:?} is not Class but {:?}", index, class)),
    }
  }

  pub(crate) fn resolve_name_and_type(&self, index: ju2) -> Result<(String, String)> {
    let name_and_type_index = self.resolve_name_and_type_index(index)?;

    let name_and_type: &CpInfo = self.resolve_index(name_and_type_index)?;

    match &name_and_type.info {
      CpInfoInfoEnum::NameAndType(info) => {
        // TODO: is it true tho the +1  ?
        Ok((
          self.resolve_name(info.name_index)?,
          self.resolve_name(info.descriptor_index)?,
        ))
      }
      _ => Err(eyre!(
        "[!] Index {:?} is not Class but {:?}",
        index,
        name_and_type
      )),
    }
  }

  pub(crate) fn resolve_class(&self, index: ju2) -> Result<ju2> {
    let value: &CpInfo = self.resolve_index(index)?;

    match &value.info {
      CpInfoInfoEnum::Class(classinfo) => Ok(classinfo.name_index),
      _ => Err(eyre!("[!] Index {:?} is not Class but {:?}", index, value)),
    }
  }

  pub(crate) fn resolve_field_ref(&self, index: ju2) -> Result<(String, String, String)> {
    let value: &CpInfo = self.resolve_index(index)?;

    let refinfo = match &value.info {
      CpInfoInfoEnum::Fieldref(refinfo) => refinfo,
      _ => {
        return Err(eyre!(
          "[!] Index {:?} is not FieldRef but {:?}",
          index,
          value
        ))
      }
    };

    let class = self.resolve_class_name(refinfo.get_class_index())?;

    let (name, _type) = self.resolve_name_and_type(index)?;

    Ok((class, name, _type))
  }

  pub(crate) fn resolve_method_ref(&self, index: ju2) -> Result<(String, String, String)> {
    let value: &CpInfo = self.resolve_index(index)?;

    let refinfo = match &value.info {
      CpInfoInfoEnum::Methodref(refinfo) => refinfo,
      CpInfoInfoEnum::Interfaceref(refinfo) => refinfo,
      _ => {
        return Err(eyre!(
          "[!] Index {:?} is not MethodRef but {:?}",
          index,
          value
        ));
      }
    };

    let class = self.resolve_class_name(refinfo.get_class_index())?;

    let (name, _type) = self.resolve_name_and_type(index)?;

    Ok((class, name, _type))
  }

  pub(crate) fn resolve_method_handle(&self, index: ju2) -> Result<(ju1, String, String, String)> {
    let value: &CpInfo = self.resolve_index(index)?;
    let metinfo = match &value.info {
      CpInfoInfoEnum::MethodHandle(metinfo) => metinfo,
      _ => {
        return Err(eyre!(
          "[!] Index {:?} is not MethodHandle but {:?}",
          index,
          value
        ));
      }
    };

    let reference_kind = metinfo.get_reference_kind();
    let reference_index = metinfo.get_reference_index();

    let (method_class, method_name, method_type) = self.resolve_method_ref(reference_index)?;

    Ok((reference_kind, method_class, method_name, method_type))
  }

  pub(crate) fn resolve_method_type(&self, index: ju2) -> Result<String> {
    let value: &CpInfo = self.resolve_index(index)?;
    let metinfo = match &value.info {
      CpInfoInfoEnum::MethodType(metinfo) => metinfo,
      _ => {
        return Err(eyre!(
          "[!] Index {:?} is not MethodType but {:?}",
          index,
          value
        ));
      }
    };

    let descriptor_index = metinfo.get_descriptor_index();

    let descriptor_str = self.resolve_name(descriptor_index)?;

    Ok(descriptor_str)
  }

  pub(crate) fn resolve_invokedynamic(&self, index: ju2) -> Result<(ju2, String, String)> {
    let value: &CpInfo = self.resolve_index(index)?;

    let refinfo = match &value.info {
      CpInfoInfoEnum::InvokeDynamic(refinfo) => refinfo,
      _ => {
        return Err(eyre!(
          "[!] Index {:?} is not MethodRef but {:?}",
          index,
          value
        ));
      }
    };

    let (name, _type) = self.resolve_name_and_type(index)?;

    Ok((refinfo.bootstrap_method_attr_index, name, _type))
  }

  fn resolve_name_and_type_index(&self, index: ju2) -> Result<ju2> {
    let value: &CpInfo = self.resolve_index(index)?;

    match &value.info {
      CpInfoInfoEnum::NameAndType(_) => Ok(index),
      CpInfoInfoEnum::Fieldref(refinfo) => Ok(refinfo.name_and_type_index),
      CpInfoInfoEnum::Methodref(refinfo) => Ok(refinfo.name_and_type_index),
      CpInfoInfoEnum::Interfaceref(refinfo) => Ok(refinfo.name_and_type_index),
      CpInfoInfoEnum::InvokeDynamic(refinfo) => Ok(refinfo.name_and_type_index),
      _ => Err(eyre!("[!] Index {:?} is not Ref but {:?}", index, value)),
    }
  }

  pub(crate) fn _resolve_static(&self, index: ju2) -> Result<()> {
    let value: &CpInfo = self.resolve_index(index)?;

    match &value.info {
      CpInfoInfoEnum::Fieldref(refinfo) => {
        let class = self.resolve_class_name(refinfo.class_index)?;

        // TODO: this works only for static fields in the same class
        // TODO: based on class find the correct Class struct and use it instead
        let name_and_type = self.resolve_name_and_type(refinfo.name_and_type_index)?;

        debug!("Index: {:?} -> {:?}", index, refinfo);

        debug!("Class: {:?}", class);

        debug!("Name: {:?}", name_and_type);
      }
      _ => {
        return Err(eyre!(
          "[!] Index {:?} is not FieldRed but {:?}",
          index,
          value
        ))
      }
    }

    Ok(())
  }

  pub(crate) fn _resolve_float(&self, index: ju2) -> Result<ju4> {
    let value = self.resolve_index(index)?;

    match &value.info {
      CpInfoInfoEnum::Float(num) => Ok(num.bytes),
      _ => Err(eyre!("[!] Index {:?} is not Float but {:?}", index, value)),
    }
  }
}
