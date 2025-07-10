use color_eyre::eyre::Result;
use log::debug;

use crate::{
  class_loader::{attributes::*, constant_pool::*},
  utils::*,
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MethodInfo {
  name: String,       // not in original struct
  descriptor: String, // not in original struct
  access_flags: ju2,
  name_index: ju2,
  descriptor_index: ju2,
  attributes: Attributes,

  has_code: bool,    // for performance
  code_index: usize, // for performance, index of code in `attributes`

  has_polymorphic_signature: bool, // for performance
}

impl MethodInfo {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(MethodInfo, usize)> {
    let mut index: usize = 0;

    let access_flags: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;

    index += 2;

    let name_index: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;

    index += 2;

    let descriptor_index: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;

    index += 2;

    let (attributes, attributes_size) = Attributes::parse(&bytes[index..], cp)?;

    // native methods dont have code attribute but they still have a method body
    let (has_code, code_index) = 'found: {
      if (access_flags & 0x0100) != 0 {
        (true, 0)
      } else {
        for i in 0..attributes.len() {
          if let AttributeInfoInfoEnum::Code(_) = attributes.get(i).get_info() {
            break 'found (true, i);
          }
        }
        (false, 0)
      }
    };

    let has_polymorphic_signature = 'found: {
      for i in 0..attributes.len() {
        if let AttributeInfoInfoEnum::RuntimeVisibleAnnotations(annotations) =
          attributes.get(i).get_info()
        {
          break 'found annotations.has_polymorphic_signature();
        }
      }
      false
    };

    index += attributes_size;

    let name = cp.resolve_name(name_index)?;

    let descriptor = cp.resolve_name(descriptor_index)?;

    Ok((
      MethodInfo {
        name,
        descriptor,
        access_flags,
        name_index,
        descriptor_index,
        attributes,

        has_code,
        code_index,

        has_polymorphic_signature,
      },
      index,
    ))
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }

  pub fn _get_name_index(&self) -> ju2 {
    self.name_index
  }

  pub fn get_descriptor(&self) -> &str {
    &self.descriptor
  }

  pub fn _get_descriptor_index(&self) -> ju2 {
    self.descriptor_index
  }

  pub fn _get_attributes(&self) -> &Attributes {
    &self.attributes
  }

  pub fn has_code(&self) -> bool {
    self.has_code
  }

  pub fn get_code(&self) -> Option<&Code> {
    if self.has_code {
      if let AttributeInfoInfoEnum::Code(c) = self.attributes.get(self.code_index).get_info() {
        return Some(c);
      }
    }
    None
  }

  pub fn get_access_flags(&self) -> ju2 {
    self.access_flags
  }

  pub fn is_public(&self) -> bool {
    (self.access_flags & 0x0001) != 0
  }

  pub fn is_native(&self) -> bool {
    (self.access_flags & 0x0100) != 0
  }

  pub fn is_static(&self) -> bool {
    (self.access_flags & 0x0008) != 0
  }

  pub fn has_polymorphic_signature(&self) -> bool {
    self.has_polymorphic_signature
  }
}

#[derive(Debug)]
pub struct Methods {
  methods: Vec<MethodInfo>,
}

impl Methods {
  pub fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Methods, usize)> {
    debug!("[-] Parsing Methods");
    let mut methods: Vec<MethodInfo> = vec![];

    let mut index: usize = 0;

    let methods_count: ju2 = u16::from_be_bytes(bytes[0..2].try_into().unwrap());

    debug!("    Found {:?} methods", methods_count);
    index += 2;

    for _ in 0..methods_count {
      let slice = &bytes[index..];

      let (method, size) = MethodInfo::parse(slice, cp)?;
      //debug!(
      //  "    Method : {} {}",
      //  method.get_name(),
      //  method.get_descriptor()
      //);

      index += size;

      methods.push(method);
    }

    Ok((Methods { methods }, index))
  }

  pub fn empty() -> Methods {
    Methods { methods: vec![] }
  }

  pub fn len(&self) -> usize {
    self.methods.len()
  }

  pub fn get(&self, index: usize) -> &MethodInfo {
    &self.methods[index]
  }

  pub fn get_methods(&self) -> &Vec<MethodInfo> {
    &self.methods
  }

  pub fn _get_code_by_name_index(&self, index: ju2) -> Option<&MethodInfo> {
    for i in 0..self.methods.len() {
      if self.methods[i]._get_name_index() == index {
        return Some(&self.methods[i]);
      }
    }

    None
  }

  pub fn _get_by_name_index(&self, index: ju2) -> Option<&MethodInfo> {
    for i in 0..self.methods.len() {
      if self.methods[i]._get_name_index() == index {
        return Some(&self.methods[i]);
      }
    }

    None
  }
}
