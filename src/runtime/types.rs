use std::{
  collections::HashMap,
  fmt,
  mem::discriminant,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use color_eyre::eyre::{eyre, OptionExt, Result};
use heap::Heap;
use log::{debug, error, info, warn};

use crate::{
  class_loader::{class_file::ClassFile, loader::ClassLoader},
  notimpl,
  runtime::*,
  utils::*,
};

#[derive(Debug, Copy, Clone, PartialEq)]

pub enum Type {
  None,
  _Unitialized,
  Null,
  Byte(i8),
  Boolean(bool),
  Character(i8),
  Short(i16),
  Integer(i32),
  Float(f32),
  Long(i64),
  Double(f64),

  ArrayRef(ju4),
  ObjectRef(ju4),
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Type {
  pub fn get_category(&self) -> u8 {
    match self {
      Type::Byte(_) => 1,
      Type::Boolean(_) => 1,
      Type::Character(_) => 1,
      Type::Short(_) => 1,
      Type::Integer(_) => 1,
      Type::Float(_) => 1,
      Type::Long(_) => 2,
      Type::Double(_) => 2,
      Type::Null => 1,
      Type::ArrayRef(_) => 1,
      Type::ObjectRef(_) => 1,
      Type::None => 0,
      v => panic!("{:?}", v),
    }
  }

  pub fn as_float(&self) -> Result<f32> {
    let value: f32 = match self {
      Type::Float(l) => *l,
      v => return Err(eyre!(errors::InternalError::WrongType("Number", *v))),
    };

    Ok(value)
  }

  pub fn as_bool(&self) -> Result<bool> {
    let value: bool = match self {
      Type::Byte(b) => *b != 0,
      Type::Boolean(b) => *b,
      Type::Character(c) => *c != 0,
      Type::Short(s) => *s != 0,
      Type::Integer(i) => *i != 0,
      Type::Long(l) => *l != 0,
      Type::Float(l) => *l != 0.0,
      Type::Double(l) => *l != 0.0,
      v => return Err(eyre!(errors::InternalError::WrongType("Boolean", *v))),
    };

    Ok(value)
  }

  pub fn as_short(&self) -> Result<i16> {
    let value: i16 = match self {
      Type::Byte(b) => *b as i16,
      Type::Boolean(b) => *b as i16,
      Type::Character(c) => *c as i16,
      Type::Short(s) => *s,
      Type::Integer(i) => *i as i16,
      Type::Long(l) => *l as i16,
      v => return Err(eyre!(errors::InternalError::WrongType("Short", *v))),
    };

    Ok(value)
  }

  pub fn as_byte(&self) -> Result<i8> {
    let value = match self {
      Type::Byte(b) => *b,
      Type::Boolean(b) => *b as i8,
      Type::Character(c) => *c,
      Type::Short(s) => *s as i8,
      Type::Integer(i) => *i as i8,
      Type::Long(l) => *l as i8,
      v => return Err(eyre!(errors::InternalError::WrongType("Byte", *v))),
    };

    Ok(value)
  }

  pub fn as_integer(&self) -> Result<i32> {
    let value: i32 = match self {
      Type::Byte(b) => *b as i32,
      Type::Boolean(b) => *b as i32,
      Type::Character(c) => *c as i32,
      Type::Short(s) => *s as i32,
      Type::Integer(i) => *i,
      Type::Long(l) => *l as i32,
      v => return Err(eyre!(errors::InternalError::WrongType("Integer", *v))),
    };

    Ok(value)
  }

  pub fn as_long(&self) -> Result<i64> {
    let value: i64 = match self {
      Type::Byte(b) => *b as i64,
      Type::Boolean(b) => *b as i64,
      Type::Character(c) => *c as i64,
      Type::Short(s) => *s as i64,
      Type::Integer(i) => *i as i64,
      Type::Long(l) => *l,
      v => return Err(eyre!(errors::InternalError::WrongType("Long", *v))),
    };

    Ok(value)
  }

  pub fn as_double(&self) -> Result<f64> {
    let value: f64 = match self {
      Type::Byte(b) => *b as f64,
      Type::Character(c) => *c as f64,
      Type::Short(s) => *s as f64,
      Type::Integer(i) => *i as f64,
      Type::Long(l) => *l as f64,
      Type::Float(l) => *l as f64,
      Type::Double(l) => *l,
      v => return Err(eyre!(errors::InternalError::WrongType("Long", *v))),
    };

    Ok(value)
  }

  pub fn as_ref(&self) -> Result<ju4> {
    let value: ju4 = match self {
      Type::ObjectRef(b) => *b as ju4,
      Type::ArrayRef(b) => *b as ju4,
      Type::Null => 0,
      v => return Err(eyre!(errors::InternalError::WrongType("Reference", *v))),
    };

    Ok(value)
  }

  pub fn is_integer(&self) -> bool {
    matches!(
      self,
      types::Type::Integer(_)
        | types::Type::Short(_)
        | types::Type::Byte(_)
        | types::Type::Character(_)
        | types::Type::Boolean(_)
    )
  }

  pub fn get_type(&self, heap: &Heap) -> String {
    match self {
      Type::Byte(_) => "B".to_string(),
      Type::Boolean(_) => "Z".to_string(),
      Type::Character(_) => "C".to_string(),
      Type::Short(_) => "S".to_string(),
      Type::Integer(_) => "I".to_string(),
      Type::Float(_) => "F".to_string(),
      Type::Long(_) => "J".to_string(),
      Type::Double(_) => "D".to_string(),
      Type::Null => "Null".to_string(),
      Type::ArrayRef(array_ref) => heap
        .get_array_instance(*array_ref)
        .unwrap()
        .get_classname()
        .to_string(),
      Type::ObjectRef(obj_ref) => {
        format!(
          "L{};",
          heap.get_obj_instance(*obj_ref).unwrap().get_classname()
        )
      }
      v => panic!("{:?}", v),
    }
  }

  pub fn is_primitive(type_str: &str) -> bool {
    type_str == "F"
      || type_str == "D"
      || type_str == "Z"
      || type_str == "C"
      || type_str == "I"
      || type_str == "B"
      || type_str == "S"
      || type_str == "J"
  }

  fn is_class_integer(type_str: &str) -> bool {
    type_str == "B"
      || type_str == "Z"
      || type_str == "C"
      || type_str == "S"
      || type_str == "I"
      || type_str == "J"
      || type_str == "java/lang/Boolean"
      || type_str == "java/lang/Character"
      || type_str == "java/lang/Byte"
      || type_str == "java/lang/Short"
      || type_str == "java/lang/Integer"
      || type_str == "java/lang/Long"
  }

  // input are like [B, [Z, [Ljava/lang/String']
  pub fn convert_array_descriptor_to_class_type(descriptor: &str) -> Result<String> {
    match descriptor {
      v if Type::is_primitive(v) => Ok(descriptor.to_owned()),
      t if t.starts_with("[L") && t.ends_with(';') => {
        // [Ljava/lang/String; -> java/lang/String
        Ok(t[2..t.len() - 1].into())
      }
      t if t.starts_with('[') => {
        // [B => B
        Ok(t[1..t.len()].into())
      }
      t if t.starts_with('L') && t.ends_with(';') => {
        // Ljava/lang/String; => java/lang/String
        Ok(t[1..t.len() - 1].into())
      }
      _ => panic!("{}", descriptor),
    }
  }

  // order is important becase
  // if B <= A then
  //     A := B is ok
  //     B := A is wrong
  //  left = right is ok
  //  is right a subtype of left ?
  pub fn check_type(loader: &mut ClassLoader, left: &str, right: &str) -> Result<bool> {
    if left == right {
      return Ok(true);
    }

    if right == "Null" && !Type::is_primitive(left) {
      return Ok(true);
    }

    if left == "Ljava/lang/Object;" || right == "Ljava/lang/Object;" {
      return Ok(true);
    }

    if left == "java/lang/Object" || right == "java/lang/Object" {
      return Ok(true);
    }

    if left.starts_with("[") && right.starts_with("[") {
      warn!("Not handled : {:?} = {:?}", left, right);

      if Type::check_type(
        loader,
        &get_array_element_class_name(left),
        &get_array_element_class_name(right),
      )? {
        return Ok(true);
      } else {
        notimpl!();
      }
    }

    if left.starts_with("[") && !right.starts_with("[") {
      return Ok(false);
    }

    if !left.starts_with("[") && right.starts_with("[") {
      return Ok(false);
    }

    if Type::is_class_integer(left) && Type::is_class_integer(right) {
      return Ok(true);
    }

    if Type::is_subtype(loader, left, right)? {
      return Ok(true);
    }

    if Type::implements_interface(loader, left, right)? {
      return Ok(true);
    }

    debug!("Not possible : {:?} = {:?}", left, right);

    // check for parents
    Ok(false)
  }

  // is right a subtype of left ? right <= left ?
  pub fn is_subtype(loader: &mut ClassLoader, left: &str, right: &str) -> Result<bool> {
    if left.starts_with("L") || right.starts_with("L") {
      warn!(
        "SUBTYPE FOR FIELD TYPE NOT IMPLEMENTED YET {:?} {:?}",
        left, right
      );

      return Ok(true);
    }

    if left == "java/lang/Object" || left == right {
      return Ok(true);
    }

    let mut parent_class_name = right.to_string();

    while parent_class_name != *"java/lang/Object" {
      parent_class_name = {
        let class = loader.get(&parent_class_name)?;

        class.get_parent_name().to_owned()
      };

      if parent_class_name == left {
        return Ok(true);
      }
    }

    Ok(false)
  }

  // does right implement left ?
  pub fn implements_interface(loader: &mut ClassLoader, left: &str, right: &str) -> Result<bool> {
    debug!("Check if {:?} implements {:?}", right, left);

    loader.has_interface(left, right)
  }
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
  // might not be a bad idea to use Rc<ClassFile> instead of class_name here
  //class: Arc<RwLock<ClassFile>>,
  classname: String,
  obj_ref: ju4,
  hash_code: ju4,
  fields: HashMap<String, types::Type>,
  fields_offset: HashMap<String, i64>,
  rev_fields_offset: HashMap<i64, String>,
  interfaces: Vec<String>,

  parent_obj: Option<Arc<RwLock<ObjectInstance>>>,
  monitor_count: ju4,
}

impl fmt::Display for ObjectInstance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "ObjectInstace {} {} {}",
      self.classname, self.obj_ref, self.hash_code
    )
  }
}

impl ObjectInstance {
  pub fn new(class: Arc<RwLock<ClassFile>>, obj_ref: ju4) -> Result<ObjectInstance> {
    let classname = class.read().unwrap().get_name().to_owned();

    let hash_code = generate_hash(&classname, obj_ref);

    let fields = HashMap::new();

    let fields_offset = HashMap::new();

    let rev_fields_offset = HashMap::new();

    let interfaces = vec![];

    Ok(ObjectInstance {
      //class,
      classname,
      obj_ref,
      hash_code,
      fields,
      fields_offset,
      rev_fields_offset,
      interfaces,
      parent_obj: None,
      monitor_count: 0,
    })
  }

  //pub fn get_class(&self) -> Result<RwLockReadGuard<ClassFile>> {
  //  self
  //    .class
  //    .read()
  //    .map_err(|e| eyre!("lock poisoned in get class: {}", e))
  //}

  pub fn is_string(&self) -> bool {
    self.get_classname() == "java/lang/String"
  }

  pub fn is_primitive(&self) -> bool {
    types::Type::is_primitive(self.get_classname())
  }

  pub fn get_hash_code(&self) -> ju4 {
    self.hash_code
  }

  pub fn set_parent(&mut self, parent: Arc<RwLock<ObjectInstance>>) {
    if parent.read().unwrap().get_ref() == 0 {
      self.parent_obj = None;

      panic!()
    } else {
      self.parent_obj = Some(parent)
    }
  }

  pub fn get_parent(&self) -> Option<RwLockReadGuard<ObjectInstance>> {
    self.parent_obj.as_ref().map(|arc| arc.read().unwrap())
  }

  pub fn get_parent_mut(&self) -> Option<RwLockWriteGuard<ObjectInstance>> {
    self.parent_obj.as_ref().map(|arc| arc.write().unwrap())
  }

  pub fn add_interface(&mut self, interfacename: &str) {
    self.interfaces.push(interfacename.to_string());
  }

  pub fn _has_interface(&self, interfacename: &str) -> bool {
    for interface in &self.interfaces {
      if interface == interfacename {
        return true;
      }
    }

    false
  }

  pub fn get_field(&self, fieldname: &str) -> Result<types::Type> {
    if fieldname == "jfrTracing" {
      return Ok(types::Type::Boolean(false));
    }

    match self.fields.get(fieldname) {
      Some(v) => Ok(*v),
      _ => {
        // maybe it's a field of the parent class

        if let Some(parent_obj) = self.get_parent() {
          return parent_obj.get_field(fieldname);
        }

        Err(eyre!("field not found: {:?} in {:?}", fieldname, self))
      }
    }
  }

  pub fn put_field(&mut self, fieldname: &str, fieldvalue: types::Type) -> Result<()> {
    match self.fields.get(fieldname) {
      Some(_) => self.fields.insert(fieldname.to_string(), fieldvalue),
      _ => {
        // maybe it's a field of the parent class
        if let Some(mut parent_obj) = self.get_parent_mut() {
          return parent_obj.put_field(fieldname, fieldvalue);
        }

        return Err(eyre!(
          "field not found: {:?} in {:?}",
          fieldname,
          self.get_classname()
        ));
      }
    };

    Ok(())
  }

  // create new field in current instance
  // TODO: check it exists
  pub fn new_field(&mut self, fieldname: &str, fieldvalue: types::Type) -> Result<()> {
    let n = self.fields.len() as i64;

    self.fields.insert(fieldname.to_string(), fieldvalue);

    self.fields_offset.insert(fieldname.to_string(), n);

    self.rev_fields_offset.insert(n, fieldname.to_string());

    Ok(())
  }

  pub fn get_classname(&self) -> &str {
    &self.classname
  }

  pub fn get_ref(&self) -> ju4 {
    self.obj_ref
  }

  pub fn set_ref(&mut self, obj_ref: ju4) {
    self.obj_ref = obj_ref
  }

  pub fn monitorenter(&mut self) {
    if self.monitor_count == 0 {
      self.monitor_count += 1
    }

    // TODO: check owner and then self.monitor_count += 1 or wait
    warn!("TODO: monitor enter")
  }

  pub fn monitorexit(&mut self) {
    warn!("TODO: monitor exit")
  }
}

#[derive(Debug, Clone)]
pub struct ArrayInstance {
  classname: String, // like [java/lang/Byte
  hash_code: ju4,
  element_classname: String, // like java/lang/Byte
  array_ref: ju4,
  elements: Vec<Type>,
}

impl fmt::Display for ArrayInstance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "ArrayInstance {} {} ({})",
      self.classname,
      self.array_ref,
      self.elements.len()
    )
  }
}

impl ArrayInstance {
  pub fn new(classname: &str, array_ref: ju4, elements: Vec<Type>) -> Result<ArrayInstance> {
    let class_type = match classname {
      "Z" | "C" | "F" | "D" | "B" | "S" | "I" | "J" => &format!("[{}", classname),
      _ if classname.starts_with("[") => &format!("[{}", classname),
      _ => &format!("[L{};", classname),
    };

    let hash_code = generate_hash(classname, array_ref);

    Ok(ArrayInstance {
      classname: class_type.to_string(),
      hash_code,
      element_classname: types::Type::convert_array_descriptor_to_class_type(class_type)?,
      array_ref,
      elements,
    })
  }

  pub fn print(&self) {
    let mut array = vec![];

    for e in &self.elements {
      array.push(match e {
        Type::Integer(int) => *int as u8,
        Type::Short(int) => *int as u8,
        Type::Byte(int) => *int as u8,
        Type::Character(int) => *int as u8,
        v => panic!("{:?}", v),
      });
    }

    let string = String::from_utf8(array);

    info!("Array: {:?}", string);
  }

  pub fn get_string(&self) -> Result<String> {
    let mut array = vec![];

    for e in &self.elements {
      array.push(match e {
        Type::Integer(int) => *int as u8,
        Type::Short(int) => *int as u8,
        Type::Byte(int) => *int as u8,
        _ => panic!("{:?} {:#?}", e, self),
      });
    }

    Ok(String::from_utf8(array)?)
  }

  pub fn get(&self, index: usize) -> Result<&types::Type> {
    self
      .elements
      .get(index)
      .ok_or_eyre(eyre!(errors::JavaException::ArrayIndexOutOfBounds(
        index,
        self.elements.len()
      )))
  }

  pub fn get_hash_code(&self) -> ju4 {
    self.hash_code
  }

  pub fn get_with_index_scale(&self, index: usize) -> Result<&types::Type> {
    let scale = (get_index_scale(self.get_classname())).ilog2();

    let index = index >> scale;

    self.get(index)
  }

  pub fn get_elements(&self) -> &Vec<types::Type> {
    &self.elements
  }

  pub fn set(&mut self, index: usize, value: Type) -> Result<()> {
    if index >= self.elements.len() {
      return Err(eyre!(errors::JavaException::ArrayIndexOutOfBounds(
        index,
        self.elements.len()
      )));
    }

    let def_value = get_default_value(&self.classname);

    if discriminant(&def_value) != discriminant(&value) && def_value != types::Type::Null {
      error!("{:?} {:?}", (&get_default_value(&self.classname)), (&value));

      error!("SET WRONG {:?} {:?}", self.classname, value);

      panic!()
    }

    // TODO: check type
    self.elements[index] = value;

    Ok(())
  }

  // set considering index scale -> rescaling
  pub fn set_with_index_scale(&mut self, index: usize, value: Type) -> Result<()> {
    let scale = (get_index_scale(self.get_classname())).ilog2();

    let index = index >> scale;

    self.set(index, value)
  }

  pub fn get_ref(&self) -> ju4 {
    self.array_ref
  }

  pub fn set_ref(&mut self, array_ref: ju4) {
    self.array_ref = array_ref
  }

  pub fn get_classname(&self) -> &str {
    &self.classname
  }

  pub fn get_element_classname(&self) -> &str {
    &self.element_classname
  }

  pub fn len(&self) -> usize {
    self.elements.len()
  }
}

#[derive(Debug, Clone)]

pub enum Instance {
  ObjectInstance(ObjectInstance),
  ArrayInstance(ArrayInstance),
}

impl fmt::Display for Instance {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Instance::ObjectInstance(obj) => write!(f, "{}", obj),
      Instance::ArrayInstance(obj) => write!(f, "{}", obj),
    }
  }
}

impl Instance {
  // class field type like [B, [Z, etc
  pub fn get_class_field_type(&self) -> &str {
    match self {
      Instance::ObjectInstance(obj) => obj.get_classname(),
      Instance::ArrayInstance(obj) => obj.get_classname(),
    }
  }

  pub fn get_classname(&self) -> &str {
    match self {
      Instance::ObjectInstance(obj) => obj.get_classname(),
      Instance::ArrayInstance(obj) => obj.get_classname(),
    }
  }

  pub fn get_ref(&self) -> ju4 {
    match self {
      Instance::ObjectInstance(obj) => obj.get_ref(),
      Instance::ArrayInstance(obj) => obj.get_ref(),
    }
  }

  pub fn get_hash_code(&self) -> ju4 {
    match self {
      Instance::ObjectInstance(obj) => obj.get_hash_code(),
      Instance::ArrayInstance(obj) => obj.get_hash_code(),
    }
  }

  pub fn _is_array(&self) -> bool {
    matches!(self, Instance::ArrayInstance(_))
  }
}
