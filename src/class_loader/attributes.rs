use crate::{class_loader::constant_pool::*, notimpl, runtime::types, utils::*};
use color_eyre::eyre::{eyre, Result};

use super::loader::ClassLoader;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ConstantValue {
  constantvalue_index: ju2,
}

impl ConstantValue {
  fn parse(bytes: &[u8]) -> Result<(ConstantValue, usize)> {
    let constantvalue_index = ju2_from_bytes(&bytes[0..2])?;

    Ok((
      ConstantValue {
        constantvalue_index,
      },
      2,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SourceFile {
  sourcefile_index: ju2,
}

impl SourceFile {
  fn parse(bytes: &[u8]) -> Result<(SourceFile, usize)> {
    let sourcefile_index = ju2_from_bytes(&bytes[0..2])?;

    Ok((SourceFile { sourcefile_index }, 2))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ExceptionTableEntry {
  start_pc: ju2,
  end_pc: ju2,
  handler_pc: ju2,
  catch_type: String,
}

impl ExceptionTableEntry {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(ExceptionTableEntry, usize)> {
    let start_pc = ju2_from_bytes(&bytes[0..2])?;

    let end_pc = ju2_from_bytes(&bytes[2..4])?;

    let handler_pc = ju2_from_bytes(&bytes[4..6])?;

    let catch_type_index = ju2_from_bytes(&bytes[6..8])?;
    let catch_type = if catch_type_index == 0 {
      "finally".to_string()
    } else {
      cp.resolve_class_name(catch_type_index)?
    };

    Ok((
      ExceptionTableEntry {
        start_pc,
        end_pc,
        handler_pc,
        catch_type,
      },
      8,
    ))
  }

  pub(crate) fn check_exception(
    &self,
    loader: &mut ClassLoader,
    pc: ju2,
    exception_class: &str,
  ) -> Result<bool> {
    Ok(
      pc >= self.start_pc
        && pc <= self.end_pc
        && types::Type::check_type(loader, &self.catch_type, exception_class)?,
    )
  }

  pub(crate) fn get_handler_pc(&self) -> ju2 {
    self.handler_pc
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Code {
  max_stack: ju2,
  max_locals: ju2,
  code_length: ju4,
  code: Vec<ju1>,
  exception_table: Vec<ExceptionTableEntry>,
  attributes: Attributes,
}

impl Code {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Code, usize)> {
    let max_stack = ju2_from_bytes(&bytes[0..2])?;

    let max_locals = ju2_from_bytes(&bytes[2..4])?;

    let code_length = ju4_from_bytes(&bytes[4..8])?;

    let mut index = 8;

    let code_slice = &bytes[index..];

    let mut code: Vec<ju1> = vec![];

    for code_item in code_slice.iter().take(code_length as usize) {
      code.push(*code_item);
    }

    index += code_length as usize;

    let exception_table_length = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;

    index += 2;

    let mut exception_table: Vec<ExceptionTableEntry> = vec![];

    for _ in 0..(exception_table_length as usize) {
      let slice = &bytes[index..];

      let (excep, bytes_read) = ExceptionTableEntry::parse(slice, cp)?;

      index += bytes_read;

      exception_table.push(excep);
    }

    let (attributes, attributes_size) = Attributes::parse(&bytes[index..], cp)?;

    index += attributes_size;

    Ok((
      Code {
        max_stack,
        max_locals,
        code_length,
        code,
        exception_table,
        attributes,
      },
      index,
    ))
  }

  pub(crate) fn get_max_locals(&self) -> ju2 {
    self.max_locals
  }

  pub(crate) fn _get_attribute(&self, index: usize) -> &AttributeInfo {
    self.attributes.get(index)
  }

  pub(crate) fn get_code_vec(&self) -> &Vec<ju1> {
    &self.code
  }

  pub(crate) fn check_exception(
    &self,
    loader: &mut ClassLoader,
    pc: ju2,
    exception_class: &str,
  ) -> Result<Option<ju2>> {
    for exception in self.exception_table.clone() {
      if exception.check_exception(loader, pc, exception_class)? {
        return Ok(Some(exception.get_handler_pc()));
      }
    }
    Ok(None)
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LineNumberTableEntry {
  start_pc: ju2,
  line_number: ju2,
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.12
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LineNumberTable {
  line_number_table_length: ju2,
  line_number_table: Vec<LineNumberTableEntry>,
}

impl LineNumberTable {
  fn parse(bytes: &[u8]) -> Result<(LineNumberTable, usize)> {
    let line_number_table_length = ju2_from_bytes(&bytes[0..2])?;

    let mut line_number_table: Vec<LineNumberTableEntry> = vec![];

    let mut index = 2;

    for _ in 0..line_number_table_length {
      let slice = &bytes[index..];

      let start_pc = ju2_from_bytes(&slice[0..2])?;

      let line_number = ju2_from_bytes(&slice[2..4])?;

      index += 4;

      line_number_table.push(LineNumberTableEntry {
        start_pc,
        line_number,
      });
    }

    Ok((
      LineNumberTable {
        line_number_table_length,
        line_number_table,
      },
      (2 + line_number_table_length * 4) as usize,
    ))
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.28
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct NestHost {
  host_class_index: ju2,
}

impl NestHost {
  fn parse(bytes: &[u8]) -> Result<(NestHost, usize)> {
    let host_class_index = ju2_from_bytes(&bytes[0..2])?;

    Ok((NestHost { host_class_index }, 2))
  }

  pub(crate) fn get_host_class_index(&self) -> ju2 {
    self.host_class_index
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.29
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NestMembers {
  number_of_classes: ju2,
  classes: Vec<ju2>,
}

impl NestMembers {
  fn parse(bytes: &[u8]) -> Result<(NestMembers, usize)> {
    let number_of_classes = ju2_from_bytes(&bytes[0..2])?;

    let mut classes: Vec<ju2> = vec![];

    let mut index = 2;

    for _ in 0..number_of_classes {
      let slice = &bytes[index..];

      let class = ju2_from_bytes(&slice[0..2])?;

      classes.push(class);

      index += 2;
    }

    Ok((
      NestMembers {
        number_of_classes,
        classes,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InnerClassesClass {
  inner_class_info_index: ju2,
  outer_class_info_index: ju2,
  inner_name: String,
  inner_class_access_flags: ju2,
}

impl InnerClassesClass {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(InnerClassesClass, usize)> {
    let inner_class_info_index = ju2_from_bytes(&bytes[0..2])?;

    let outer_class_info_index = ju2_from_bytes(&bytes[2..4])?;

    let inner_name_index = ju2_from_bytes(&bytes[4..6])?;
    let inner_name = if inner_name_index == 0 {
      // anonymous
      "".to_string()
    } else {
      cp.resolve_name(inner_name_index)?
    };

    let inner_class_access_flags = ju2_from_bytes(&bytes[6..8])?;

    Ok((
      InnerClassesClass {
        inner_class_info_index,
        outer_class_info_index,
        inner_name,
        inner_class_access_flags,
      },
      8,
    ))
  }

  pub(crate) fn get_name(&self) -> &str {
    &self.inner_name
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.6
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct InnerClasses {
  number_of_classes: ju2,
  classes: Vec<InnerClassesClass>,
}

impl InnerClasses {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(InnerClasses, usize)> {
    let number_of_classes = ju2_from_bytes(&bytes[0..2])?;

    let mut classes: Vec<InnerClassesClass> = vec![];

    let mut index = 2;

    for _ in 0..number_of_classes {
      let slice = &bytes[index..];

      let (class, bytes_read) = InnerClassesClass::parse(slice, cp)?;

      classes.push(class);

      index += bytes_read;
    }

    Ok((
      InnerClasses {
        number_of_classes,
        classes,
      },
      index,
    ))
  }

  pub(crate) fn get_inner_classes(&self) -> &Vec<InnerClassesClass> {
    &self.classes
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.9
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Signature {
  signature_index: ju2,
}

impl Signature {
  fn parse(bytes: &[u8]) -> Result<(Signature, usize)> {
    let signature_index = ju2_from_bytes(&bytes[0..2])?;

    Ok((Signature { signature_index }, 2))
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.5
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Exceptions {
  exception_index_table: Vec<ju2>,
}

impl Exceptions {
  fn parse(bytes: &[u8]) -> Result<(Exceptions, usize)> {
    let number_of_exceptions = ju2_from_bytes(&bytes[0..2])?;

    let mut exception_index_table: Vec<ju2> = vec![];

    let mut index = 2;

    for _ in 0..number_of_exceptions {
      let slice = &bytes[index..];

      let excep = ju2_from_bytes(&slice[0..2])?;

      exception_index_table.push(excep);

      index += 2;
    }

    Ok((
      Exceptions {
        exception_index_table,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ElementValueEnum {
  ConstValue {
    const_value_index: u16,
  },
  EnumConstValue {
    type_name_index: u16,
    const_name_index: u16,
  },
  ClassInfo {
    class_info_index: u16,
  },
  AnnotationValue(Annotation),
  ArrayValue(Vec<ElementValue>),
  NotRecognized,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ElementValue {
  tag: ju1,
  value: ElementValueEnum,
}

impl ElementValue {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(ElementValue, usize)> {
    let tag = bytes[0];

    let (value, bytes_read) = match tag {
      b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' | b's' => {
        let const_value_index = ju2_from_bytes(&bytes[1..3])?;

        (ElementValueEnum::ConstValue { const_value_index }, 1 + 2)
      }
      b'e' => {
        let type_name_index = ju2_from_bytes(&bytes[1..3])?;

        let const_name_index = ju2_from_bytes(&bytes[3..5])?;

        (
          ElementValueEnum::EnumConstValue {
            type_name_index,
            const_name_index,
          },
          1 + 4,
        )
      }
      b'c' => {
        let class_info_index = ju2_from_bytes(&bytes[1..3])?;

        (ElementValueEnum::ClassInfo { class_info_index }, 1 + 2)
      }
      b'@' => {
        let (annotation, bytes_read) = Annotation::parse(&bytes[1..], cp)?;

        (
          ElementValueEnum::AnnotationValue(annotation),
          1 + bytes_read,
        )
      }
      b'[' => {
        let num_values = ju2_from_bytes(&bytes[1..3])?;

        let mut values: Vec<ElementValue> = vec![];

        let mut index = 3;

        for _ in 0..num_values {
          let (element, bytes_read) = ElementValue::parse(&bytes[index..], cp)?;

          index += bytes_read;

          values.push(element);
        }

        (ElementValueEnum::ArrayValue(values), index)
      }
      _ => {
        return Err(eyre!(
          "tag not supported in element value: '{}'",
          tag as char
        ))
      }
    };

    Ok((ElementValue { tag, value }, bytes_read))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ElementValuePair {
  element_name_index: ju2,
  value: ElementValue,
}

impl ElementValuePair {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(ElementValuePair, usize)> {
    let element_name_index = ju2_from_bytes(&bytes[0..2])?;

    let (value, bytes_read) = ElementValue::parse(&bytes[2..], cp)?;

    Ok((
      ElementValuePair {
        element_name_index,
        value,
      },
      2 + bytes_read,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Annotation {
  annotation_type: String,
  num_element_value_pairs: ju2,
  element_value_pairs: Vec<ElementValuePair>,
}

impl Annotation {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Annotation, usize)> {
    let type_index = ju2_from_bytes(&bytes[0..2])?;
    let annotation_type = cp.resolve_name(type_index)?;

    let num_element_value_pairs = ju2_from_bytes(&bytes[2..4])?;

    let mut element_value_pairs: Vec<ElementValuePair> = vec![];

    let mut index = 4;

    for _ in 0..num_element_value_pairs {
      let (element, bytes_read) = ElementValuePair::parse(&bytes[index..], cp)?;

      index += bytes_read;

      element_value_pairs.push(element);
    }

    Ok((
      Annotation {
        annotation_type,
        num_element_value_pairs,
        element_value_pairs,
      },
      index,
    ))
  }

  pub(crate) fn get_annotation_type(&self) -> &str {
    &self.annotation_type
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.16
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RuntimeVisibleAnnotations {
  num_annotations: ju2,
  annotations: Vec<Annotation>,
}

impl RuntimeVisibleAnnotations {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(RuntimeVisibleAnnotations, usize)> {
    let num_annotations = ju2_from_bytes(&bytes[0..2])?;

    let mut annotations: Vec<Annotation> = vec![];

    let mut index = 2;

    for _ in 0..num_annotations {
      let (annotation, bytes_read) = Annotation::parse(&bytes[index..], cp)?;

      index += bytes_read;

      annotations.push(annotation);
    }

    Ok((
      RuntimeVisibleAnnotations {
        num_annotations,
        annotations,
      },
      index,
    ))
  }

  pub(crate) fn has_polymorphic_signature(&self) -> bool {
    for ann in &self.annotations {
      if ann.get_annotation_type() == "Ljava/lang/invoke/MethodHandle$PolymorphicSignature;" {
        return true;
      }
    }
    false
  }
}

// https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-4.html#jvms-4.7.17
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RuntimeInvisibleAnnotations {
  num_annotations: ju2,
  annotations: Vec<Annotation>,
}

impl RuntimeInvisibleAnnotations {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(RuntimeInvisibleAnnotations, usize)> {
    let num_annotations = ju2_from_bytes(&bytes[0..2])?;

    let mut annotations: Vec<Annotation> = vec![];

    let mut index = 2;

    for _ in 0..num_annotations {
      let (annotation, bytes_read) = Annotation::parse(&bytes[index..], cp)?;

      index += bytes_read;

      annotations.push(annotation);
    }

    Ok((
      RuntimeInvisibleAnnotations {
        num_annotations,
        annotations,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum VerificationTypeInfoEnum {
  Top,
  Integer,
  Float,
  Long,
  Double,
  Null,
  UninitializedThis,
  Object { cpool_index: ju2 },
  Uninitialized { offset: ju2 },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VerificationTypeInfo {
  tag: ju1,
  info: VerificationTypeInfoEnum,
}

impl VerificationTypeInfo {
  fn parse(bytes: &[u8]) -> Result<(VerificationTypeInfo, usize)> {
    let tag = bytes[0];

    let (info, bytes_read) = match tag {
      0 => (VerificationTypeInfoEnum::Top, 0),
      1 => (VerificationTypeInfoEnum::Integer, 0),
      2 => (VerificationTypeInfoEnum::Float, 0),
      3 => (VerificationTypeInfoEnum::Double, 0),
      4 => (VerificationTypeInfoEnum::Long, 0),
      5 => (VerificationTypeInfoEnum::Null, 0),
      6 => (VerificationTypeInfoEnum::UninitializedThis, 0),
      7 => {
        let cpool_index = ju2_from_bytes(&bytes[1..3])?;

        (VerificationTypeInfoEnum::Object { cpool_index }, 2)
      }
      8 => {
        let offset = ju2_from_bytes(&bytes[1..3])?;

        (VerificationTypeInfoEnum::Uninitialized { offset }, 2)
      }
      _ => return Err(eyre!("verification type info not recognized: {:?}", tag)),
    };

    Ok((
      VerificationTypeInfo { tag, info },
      (bytes_read + 1) as usize,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum StackMapFrameEnum {
  SameFrame, /* 0-63 */
  SameLocals1StackItemFrame {
    /* 64-127 */ stack: Vec<VerificationTypeInfo>,
  },
  SameLocals1StackItemFrameExtended {
    /* 247 */ offset_delta: ju2,
    stack: Vec<VerificationTypeInfo>,
  },
  ChopFrame {
    /* 248-250 */ offset_delta: ju2,
  },
  SameFrameExtended {
    /* 251 */ offset_delta: ju2,
  },
  AppendFrame {
    /* 252-254 */ offset_delta: ju2,
    locals: Vec<VerificationTypeInfo>,
  },
  FullFrame {
    /* 255 */
    offset_delta: ju2,
    number_of_locals: ju2,
    locals: Vec<VerificationTypeInfo>,
    number_of_stack_items: ju2,
    stack: Vec<VerificationTypeInfo>,
  },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StackMapFrame {
  frame_type: ju1,
  frame_enum: StackMapFrameEnum,
}

impl StackMapFrame {
  fn parse(bytes: &[u8]) -> Result<(StackMapFrame, usize)> {
    let frame_type = bytes[0];

    let (frame_enum, bytes_read) = match frame_type {
      0..=63 => (StackMapFrameEnum::SameFrame, 0),
      64..=127 => {
        let mut stack: Vec<VerificationTypeInfo> = vec![];

        let (verinfo, bytes_read) = VerificationTypeInfo::parse(&bytes[1..])?;

        stack.push(verinfo);

        (
          StackMapFrameEnum::SameLocals1StackItemFrame { stack },
          bytes_read,
        )
      }
      247 => {
        let offset_delta = ju2_from_bytes(&bytes[1..3])?;

        let mut stack: Vec<VerificationTypeInfo> = vec![];

        let (verinfo, bytes_read) = VerificationTypeInfo::parse(&bytes[3..])?;

        stack.push(verinfo);

        (
          StackMapFrameEnum::SameLocals1StackItemFrameExtended {
            offset_delta,
            stack,
          },
          bytes_read + 2, // parsing verification info + offset_delta
        )
      }
      248..=250 => {
        let offset_delta = ju2_from_bytes(&bytes[1..3])?;

        (StackMapFrameEnum::ChopFrame { offset_delta }, 2)
      }
      251 => {
        let offset_delta = ju2_from_bytes(&bytes[1..3])?;

        (StackMapFrameEnum::SameFrameExtended { offset_delta }, 2)
      }
      252..=254 => {
        let offset_delta = ju2_from_bytes(&bytes[1..3])?;

        let mut locals: Vec<VerificationTypeInfo> = vec![];

        let num_locals = frame_type - 251;

        let mut index = 3;

        for _ in 0..num_locals {
          let (verinfo, bytes_read) = VerificationTypeInfo::parse(&bytes[index..])?;

          index += bytes_read;

          locals.push(verinfo);
        }

        (
          StackMapFrameEnum::SameLocals1StackItemFrameExtended {
            offset_delta,
            stack: locals,
          },
          index - 1, // not include the frame_type
        )
      }
      255 => {
        let offset_delta = ju2_from_bytes(&bytes[1..3])?;

        let number_of_locals = ju2_from_bytes(&bytes[3..5])?;

        let mut locals: Vec<VerificationTypeInfo> = vec![];

        let mut index = 5;

        for _ in 0..number_of_locals {
          let (verinfo, bytes_read) = VerificationTypeInfo::parse(&bytes[index..])?;

          index += bytes_read;

          locals.push(verinfo);
        }

        let number_of_stack_items = ju2_from_bytes(&bytes[index..index + 2])?;

        let mut stack: Vec<VerificationTypeInfo> = vec![];

        index += 2;

        for _ in 0..number_of_stack_items {
          let (verinfo, bytes_read) = VerificationTypeInfo::parse(&bytes[index..])?;

          index += bytes_read;

          stack.push(verinfo);
        }

        (
          StackMapFrameEnum::FullFrame {
            offset_delta,
            number_of_locals,
            locals,
            number_of_stack_items,
            stack,
          },
          index - 1, // not include the frame_type
        )
      }
      _ => return Err(eyre!("stack map table not recognized: {:?}", frame_type)),
    };

    Ok((
      StackMapFrame {
        frame_type,
        frame_enum,
      },
      bytes_read + 1,
    ))
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.4
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StackMapTable {
  number_of_entries: ju2,
  entries: Vec<StackMapFrame>,
}

impl StackMapTable {
  fn parse(bytes: &[u8]) -> Result<(StackMapTable, usize)> {
    let number_of_entries = ju2_from_bytes(&bytes[0..2])?;

    let mut entries: Vec<StackMapFrame> = vec![];

    let mut index = 2;

    for _ in 0..number_of_entries {
      let (entry, bytes_read) = StackMapFrame::parse(&bytes[index..])?;

      index += bytes_read;

      entries.push(entry);
    }

    Ok((
      StackMapTable {
        number_of_entries,
        entries,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LocalVariableTableEntry {
  start_pc: ju2,
  length: ju2,
  name_index: ju2,
  descriptor_index: ju2,
  index: ju2,
}

impl LocalVariableTableEntry {
  fn parse(bytes: &[u8]) -> Result<(LocalVariableTableEntry, usize)> {
    let start_pc = ju2_from_bytes(&bytes[0..2])?;

    let length = ju2_from_bytes(&bytes[2..4])?;

    let name_index = ju2_from_bytes(&bytes[4..6])?;

    let descriptor_index = ju2_from_bytes(&bytes[6..8])?;

    let index = ju2_from_bytes(&bytes[8..10])?;

    Ok((
      LocalVariableTableEntry {
        start_pc,
        length,
        name_index,
        descriptor_index,
        index,
      },
      10,
    ))
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.13
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LocalVariableTable {
  local_variable_table_length: ju2,
  local_variable_table: Vec<LocalVariableTableEntry>,
}

impl LocalVariableTable {
  fn parse(bytes: &[u8]) -> Result<(LocalVariableTable, usize)> {
    let local_variable_table_length = ju2_from_bytes(&bytes[0..2])?;

    let mut local_variable_table: Vec<LocalVariableTableEntry> = vec![];

    let mut index = 2;

    for _ in 0..local_variable_table_length {
      let (entry, bytes_read) = LocalVariableTableEntry::parse(&bytes[index..])?;

      index += bytes_read;

      local_variable_table.push(entry);
    }

    Ok((
      LocalVariableTable {
        local_variable_table_length,
        local_variable_table,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BootstrapMethod {
  bootstrap_method_ref: ju2,
  bootstrap_arguments: Vec<ju2>,
}

impl BootstrapMethod {
  fn parse(bytes: &[u8]) -> Result<(BootstrapMethod, usize)> {
    let bootstrap_method_ref = ju2_from_bytes(&bytes[0..2])?;

    let num_bootstrap_arguments = ju2_from_bytes(&bytes[2..4])?;

    let mut bootstrap_arguments: Vec<ju2> = vec![];

    let mut index = 4;

    for _ in 0..num_bootstrap_arguments {
      let arg = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;

      index += 2;

      bootstrap_arguments.push(arg);
    }

    Ok((
      BootstrapMethod {
        bootstrap_method_ref,
        bootstrap_arguments,
      },
      index,
    ))
  }

  pub(crate) fn get_method_ref(&self) -> ju2 {
    self.bootstrap_method_ref
  }

  pub(crate) fn get_arguments(&self) -> &Vec<ju2> {
    &self.bootstrap_arguments
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BootstrapMethods {
  num_bootstrap_methods: ju2,
  bootstrap_methods: Vec<BootstrapMethod>,
}

impl BootstrapMethods {
  fn parse(bytes: &[u8]) -> Result<(BootstrapMethods, usize)> {
    let num_bootstrap_methods = ju2_from_bytes(&bytes[0..2])?;

    let mut bootstrap_methods: Vec<BootstrapMethod> = vec![];

    let mut index = 2;

    for _ in 0..num_bootstrap_methods {
      let (entry, bytes_read) = BootstrapMethod::parse(&bytes[index..])?;

      index += bytes_read;

      bootstrap_methods.push(entry);
    }

    Ok((
      BootstrapMethods {
        num_bootstrap_methods,
        bootstrap_methods,
      },
      index,
    ))
  }

  pub fn get(&self, index: usize) -> &BootstrapMethod {
    &self.bootstrap_methods[index]
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MethodParameter {
  name_index: ju2,
  access_flags: ju2,
}

impl MethodParameter {
  fn parse(bytes: &[u8]) -> Result<(MethodParameter, usize)> {
    let name_index = ju2_from_bytes(&bytes[0..2])?;

    let access_flags = ju2_from_bytes(&bytes[2..4])?;

    Ok((
      MethodParameter {
        name_index,
        access_flags,
      },
      4,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MethodParameters {
  parameters_count: ju1,
  parameters: Vec<MethodParameter>,
}

impl MethodParameters {
  fn parse(bytes: &[u8]) -> Result<(MethodParameters, usize)> {
    let parameters_count = bytes[0];

    let mut parameters: Vec<MethodParameter> = vec![];

    let mut index = 1;

    for _ in 0..parameters_count {
      let (entry, bytes_read) = MethodParameter::parse(&bytes[index..])?;

      index += bytes_read;

      parameters.push(entry);
    }

    Ok((
      MethodParameters {
        parameters_count,
        parameters,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct EnclosingMethod {
  class_name: String,
  method_name: String,
  method_descriptor: String,
}

impl EnclosingMethod {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(EnclosingMethod, usize)> {
    let class_index = ju2_from_bytes(&bytes[0..2])?;
    let class_name = cp.resolve_class_name(class_index)?;

    let method_index = ju2_from_bytes(&bytes[2..4])?;
    // no defined within a method
    let (method_name, method_descriptor) = if method_index == 0 {
      ("".to_string(), "".to_string())
    } else {
      cp.resolve_name_and_type(method_index)?
    };

    Ok((
      EnclosingMethod {
        class_name,
        method_name,
        method_descriptor,
      },
      4,
    ))
  }

  pub(crate) fn get_classname(&self) -> &str {
    &self.class_name
  }

  pub(crate) fn define_in_method(&self) -> bool {
    self.method_name.is_empty()
  }

  pub(crate) fn get_method_name(&self) -> &str {
    &self.method_name
  }

  pub(crate) fn get_method_descriptor(&self) -> &str {
    &self.method_descriptor
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PermittedSubclasses {
  number_of_classes: ju2,
  classes: Vec<ju2>,
}

impl PermittedSubclasses {
  fn parse(bytes: &[u8]) -> Result<(PermittedSubclasses, usize)> {
    let number_of_classes = ju2_from_bytes(&bytes[0..2])?;

    let mut classes: Vec<ju2> = vec![];

    let mut index = 2;

    for _ in 0..number_of_classes {
      let slice = &bytes[index..];

      let class = ju2_from_bytes(&slice[0..2])?;

      classes.push(class);

      index += 2;
    }

    Ok((
      PermittedSubclasses {
        number_of_classes,
        classes,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecordComponent {
  name_index: ju2,
  descriptor_index: ju2,
  attributes: Attributes,
}

impl RecordComponent {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(RecordComponent, usize)> {
    let name_index = ju2_from_bytes(&bytes[0..2])?;

    let descriptor_index = ju2_from_bytes(&bytes[2..4])?;

    let (attributes, attributes_size) = Attributes::parse(&bytes[4..], cp)?;

    let index = 4 + attributes_size;

    Ok((
      RecordComponent {
        name_index,
        descriptor_index,
        attributes,
      },
      index,
    ))
  }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Record {
  components_count: ju2,
  components: Vec<RecordComponent>,
}

impl Record {
  fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Record, usize)> {
    let components_count = ju2_from_bytes(&bytes[0..2])?;

    let mut components = vec![];

    let mut index = 2;

    for _ in 0..components_count {
      let slice = &bytes[index..];

      let (comp, size) = RecordComponent::parse(slice, cp)?;

      components.push(comp);

      index += size;
    }

    Ok((
      Record {
        components_count,
        components,
      },
      index,
    ))
  }
}

// https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.6
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AttributeInfoInfoEnum {
  Raw(Vec<ju1>),
  // Critical
  ConstantValue(ConstantValue), /* https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.2 */
  Code(Code), // https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.3
  StackMapTable(StackMapTable),
  BootstrapMethods(BootstrapMethods),
  NestHost(NestHost),
  NestMembers(NestMembers),
  PermittedSubclasses(PermittedSubclasses),
  // ================================================== //
  // Might be critical
  Exceptions(Exceptions),
  InnerClasses(InnerClasses),
  EnclosingMethod(EnclosingMethod),
  Synthetic,
  Signature(Signature),
  Record(Record),
  SourceFile(SourceFile), /* https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.7.10 */
  LineNumberTable(LineNumberTable),
  LocalVariableTable(LocalVariableTable),
  LocalVariableTypeTable,
  // ================================================== //
  // Not critical
  SourceDebugExtension,
  Deprecated,
  RuntimeVisibleAnnotations(RuntimeVisibleAnnotations),
  RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotations),
  RuntimeVisibleParameterAnnotations,
  RuntimeInvisibleParameterAnnotations,
  RuntimeVisibleTypeAnnotations,
  RuntimeInvisibleTypeAnnotations,
  AnnotationDefault,
  MethodParameters(MethodParameters),
  Module,
  ModulePackages,
  ModuleMainClass,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AttributeInfo {
  attribute_name: String, // not part of original struct
  attribute_name_index: ju2,
  attribute_length: ju4,
  //info: Vec<ju1>,
  info: AttributeInfoInfoEnum,
}

impl AttributeInfo {
  pub fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(AttributeInfo, usize)> {
    let mut index = 0;

    let attribute_name_index: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;

    index += 2;

    let attribute_length: ju4 = ju4_from_bytes(get_slice_arr(bytes, index, 4))?;

    index += 4;

    let attribute_name = cp.resolve_name(attribute_name_index)?;

    let info_slice = &bytes[index..];

    //debug!("{:?}", attribute_name);
    let (info, bytes_read) = match attribute_name.as_ref() {
      "Code" => {
        let (code, bytes_read) = Code::parse(info_slice, cp)?;

        (AttributeInfoInfoEnum::Code(code), bytes_read)
      }
      "SourceFile" => {
        let (source_file, bytes_read) = SourceFile::parse(info_slice)?;

        (AttributeInfoInfoEnum::SourceFile(source_file), bytes_read)
      }
      "LineNumberTable" => {
        let (source_file, bytes_read) = LineNumberTable::parse(info_slice)?;

        (
          AttributeInfoInfoEnum::LineNumberTable(source_file),
          bytes_read,
        )
      }
      "NestHost" => {
        let (nest_host, bytes_read) = NestHost::parse(info_slice)?;

        (AttributeInfoInfoEnum::NestHost(nest_host), bytes_read)
      }
      "NestMembers" => {
        let (nest_members, bytes_read) = NestMembers::parse(info_slice)?;

        (AttributeInfoInfoEnum::NestMembers(nest_members), bytes_read)
      }
      "InnerClasses" => {
        let (inner_classes, bytes_read) = InnerClasses::parse(info_slice, cp)?;

        (
          AttributeInfoInfoEnum::InnerClasses(inner_classes),
          bytes_read,
        )
      }
      "Signature" => {
        let (signature, bytes_read) = Signature::parse(info_slice)?;

        (AttributeInfoInfoEnum::Signature(signature), bytes_read)
      }
      "Exceptions" => {
        let (exceptions, bytes_read) = Exceptions::parse(info_slice)?;

        (AttributeInfoInfoEnum::Exceptions(exceptions), bytes_read)
      }
      "RuntimeVisibleAnnotations" => {
        let (annotations, bytes_read) = RuntimeVisibleAnnotations::parse(info_slice, cp)?;

        (
          AttributeInfoInfoEnum::RuntimeVisibleAnnotations(annotations),
          bytes_read,
        )
      }
      "RuntimeInvisibleAnnotations" => {
        let (annotations, bytes_read) = RuntimeInvisibleAnnotations::parse(info_slice, cp)?;

        (
          AttributeInfoInfoEnum::RuntimeInvisibleAnnotations(annotations),
          bytes_read,
        )
      }
      "StackMapTable" => {
        let (stackmap, bytes_read) = StackMapTable::parse(info_slice)?;

        (AttributeInfoInfoEnum::StackMapTable(stackmap), bytes_read)
      }
      "LocalVariableTable" => {
        let (localtable, bytes_read) = LocalVariableTable::parse(info_slice)?;

        (
          AttributeInfoInfoEnum::LocalVariableTable(localtable),
          bytes_read,
        )
      }
      "ConstantValue" => {
        let (constant, bytes_read) = ConstantValue::parse(info_slice)?;

        (AttributeInfoInfoEnum::ConstantValue(constant), bytes_read)
      }
      "LocalVariableTypeTable" => {
        let (local, bytes_read) = LocalVariableTable::parse(info_slice)?;

        (AttributeInfoInfoEnum::LocalVariableTable(local), bytes_read)
      }
      "BootstrapMethods" => {
        let (methods, bytes_read) = BootstrapMethods::parse(info_slice)?;

        (AttributeInfoInfoEnum::BootstrapMethods(methods), bytes_read)
      }
      "MethodParameters" => {
        let (methods, bytes_read) = MethodParameters::parse(info_slice)?;

        (AttributeInfoInfoEnum::MethodParameters(methods), bytes_read)
      }
      "EnclosingMethod" => {
        let (method, bytes_read) = EnclosingMethod::parse(info_slice, cp)?;

        (AttributeInfoInfoEnum::EnclosingMethod(method), bytes_read)
      }
      "PermittedSubclasses" => {
        let (subclasses, bytes_read) = PermittedSubclasses::parse(info_slice)?;

        (
          AttributeInfoInfoEnum::PermittedSubclasses(subclasses),
          bytes_read,
        )
      }
      "Deprecated" => (AttributeInfoInfoEnum::Deprecated, 0),
      "Record" => {
        let (record, bytes_read) = Record::parse(info_slice, cp)?;

        (AttributeInfoInfoEnum::Record(record), bytes_read)
      }
      v => notimpl!(v),
    };

    //debug!("    Attribute: {:?}", attribute_name);
    assert_eq!(attribute_length as usize, bytes_read);

    let attr = AttributeInfo {
      attribute_name,
      attribute_name_index,
      attribute_length,
      info,
    };

    Ok((attr, 2 + 4 + bytes_read))
  }

  pub fn get_name(&self) -> &str {
    &self.attribute_name
  }

  pub fn get_info(&self) -> &AttributeInfoInfoEnum {
    &self.info
  }
}

#[derive(Debug, Clone)]
pub struct Attributes {
  attributes: Vec<AttributeInfo>,
}

impl Attributes {
  pub fn new(attributes: Vec<AttributeInfo>) -> Attributes {
    Attributes { attributes }
  }

  pub fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Attributes, usize)> {
    //debug!("[-] Parsing Attributes");
    let mut attributes: Vec<AttributeInfo> = vec![];

    let mut index: usize = 0;

    let attributes_count: ju2 = ju2_from_bytes(&bytes[0..2])?;

    //debug!("    Found {:?} attributes", attributes_count);
    index += 2;

    for _ in 0..attributes_count {
      let slice = &bytes[index..];

      let (attribute, size) = AttributeInfo::parse(slice, cp)?;

      index += size;

      attributes.push(attribute);
    }

    Ok((Attributes { attributes }, index))
  }

  pub fn empty() -> Attributes {
    Attributes { attributes: vec![] }
  }

  pub fn len(&self) -> usize {
    self.attributes.len()
  }

  pub fn get(&self, index: usize) -> &AttributeInfo {
    &self.attributes[index]
  }

  pub fn _get_code(&self) -> Option<&Code> {
    for attr in &self.attributes {
      if let AttributeInfoInfoEnum::Code(c) = attr.get_info() {
        return Some(c);
      }
    }
    None
  }

  pub fn get_enclosing_method(&self) -> Option<&EnclosingMethod> {
    for attr in &self.attributes {
      if let AttributeInfoInfoEnum::EnclosingMethod(c) = attr.get_info() {
        return Some(c);
      }
    }
    None
  }

  pub fn get_inner_classes(&self) -> Option<&InnerClasses> {
    for attr in &self.attributes {
      if let AttributeInfoInfoEnum::InnerClasses(c) = attr.get_info() {
        return Some(c);
      }
    }
    None
  }

  pub fn get_nest_host(&self) -> Option<&NestHost> {
    for attr in &self.attributes {
      if let AttributeInfoInfoEnum::NestHost(c) = attr.get_info() {
        return Some(c);
      }
    }
    None
  }

  pub fn get_by_name(&self, name: &str) -> &AttributeInfo {
    for a in &self.attributes {
      if a.get_name() == name {
        return a;
      }
    }

    panic!();
  }
}
