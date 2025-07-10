use core::panic;
use std::{
  collections::HashMap,
  fmt,
  sync::{Arc, RwLock, RwLockReadGuard},
};

use color_eyre::eyre::{eyre, OptionExt, Result};
use log::{debug, warn};

use super::types::{ArrayInstance, ObjectInstance, Type};
use crate::{
  class_loader::{class_file, fields, loader::ClassLoader, methods},
  runtime::{errors, types},
  utils::*,
};

#[derive(Debug)]
pub struct Heap {
  next_obj_ref: ju4, // 0 is used for Null
  heap: HashMap<ju4, types::Instance>,
  classes: HashMap<String, ju4>,        // Class<T>
  static_classes: HashMap<String, ju4>, // instance of classes to access static info
  strings: HashMap<String, ju4>,
}

impl fmt::Display for Heap {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Heap: ")?;

    for (k, v) in &self.heap {
      write!(
        f,
        "\n\t{} : {:?} {:?} {:?}",
        k,
        v.get_classname(),
        v.get_ref(),
        v.get_hash_code()
      )?;

      if let types::Instance::ObjectInstance(obj) = v {
        if v.get_classname() == "java/lang/String" {
          let value = obj.get_field("value").unwrap();

          if let types::Type::ArrayRef(array_ref) = value {
            let array = self.get_array_instance(array_ref).unwrap();

            write!(f, " -> {:?}", array.get_string())?;
          }
        }
      }
    }

    write!(f, "\nClasses: ")?;

    for (k, v) in &self.classes {
      write!(f, "\n\t{} : {}", k, v)?;
    }

    write!(f, "\nStrings: ")?;

    for (k, v) in &self.strings {
      write!(f, "\n\t{} : {}", k, v)?;
    }

    Ok(())
  }
}

impl Heap {
  pub fn new() -> Self {
    Heap {
      next_obj_ref: 1,
      heap: HashMap::new(),
      classes: HashMap::new(),
      static_classes: HashMap::new(),
      strings: HashMap::new(),
    }
  }

  /// Allocate a string
  ///
  /// # Arguments
  ///
  /// * `value` - String to allocate
  ///
  /// # Returns
  ///
  /// A `ju4` representing the object reference in the heap
  pub fn alloc_string(&mut self, loader: &mut ClassLoader, value: &str) -> Result<types::Type> {
    // strings are immutable
    if self.strings.contains_key(value) {
      debug!("ALREADY EXISTS {:?}", value);

      return Ok(types::Type::ObjectRef(
        *self
          .strings
          .get(value)
          .ok_or_eyre(eyre!("something went wrong in alloc string"))?,
      ));
    }

    let arr: Vec<types::Type> = value.bytes().map(|c| types::Type::Byte(c as i8)).collect();

    let arr_len = arr.len();

    // TODO: is this the correct way to create a string ?
    // TODO: is alloc string used only to create a class obj ?
    let str_ref = self.alloc_array_primitive("B", arr, arr_len)?;

    let string_ref = self.alloc_obj(loader, "java/lang/String")?;

    let (string_obj, curr_ref) = match string_ref {
      types::Type::ObjectRef(obj_ref) => (self.get_obj_instance_mut(obj_ref)?, obj_ref),
      _ => panic!(),
    };

    string_obj.new_field("value", str_ref)?;

    self.strings.insert(value.to_string(), curr_ref);

    Ok(types::Type::ObjectRef(curr_ref))
  }

  /// Get the current reference id
  ///
  /// # Arguments
  ///
  /// # Returns
  ///
  /// Current reference id
  pub fn get_curr_obj_ref(&mut self) -> ju4 {
    self.next_obj_ref
  }

  /// Get the new reference id and increment the id counter
  ///
  /// # Arguments
  ///
  /// # Returns
  ///
  /// New reference id (before increment)
  pub fn get_next_obj_ref(&mut self) -> ju4 {
    let curr_ref = self.next_obj_ref;

    self.next_obj_ref += 1;

    curr_ref
  }

  /// Allocate an object
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class to instantiate
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated object
  pub fn alloc_obj(&mut self, loader: &mut ClassLoader, classname: &str) -> Result<types::Type> {
    // TODO: not sure about how I create a new object
    let curr_ref = self.get_next_obj_ref();

    // TODO: resolve inheritance
    let class = loader.get_lock(classname)?;

    let mut new_obj = types::ObjectInstance::new(class.clone(), curr_ref)?;

    let class_read = class.read().unwrap();

    class_read.new_obj(&mut new_obj)?;

    if classname != "java/lang/Object" {
      let parent_classname = {
        let class = loader.get(classname)?;
        class.get_parent_name().to_owned()
      };
      let parent_ref = self.alloc_obj(loader, &parent_classname)?.as_ref()?;
      let parent_obj = self.get_obj_instance(parent_ref)?;
      new_obj.set_parent(Arc::new(RwLock::new(parent_obj.clone())));
    }

    self
      .heap
      .insert(curr_ref, types::Instance::ObjectInstance(new_obj));

    Ok(types::Type::ObjectRef(curr_ref))
  }

  /// Get the component type of an array
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class<T> where T is an array type
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the class of the component type (i.e. Class<T> where T is the component type
  /// class)
  pub fn get_component_type(
    &mut self,
    loader: &mut ClassLoader,
    classname: &str,
  ) -> Result<types::Type> {
    let comp_type = types::Type::convert_array_descriptor_to_class_type(classname)?;

    // TODO: force load it
    if !self.has_class_instance(&comp_type) {
      self.alloc_class_obj(loader, &comp_type)?;
    }

    if !types::Type::is_primitive(&comp_type) {
      let v = loader.get(&comp_type)?;

      drop(v);
    }

    let comp_class = self.get_class_instance(loader, &comp_type)?;

    Ok(types::Type::ObjectRef(comp_class.get_ref()))
  }

  /// Allocate a Class<T> object
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class to instantiate
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated object
  pub fn alloc_class_obj(
    &mut self,
    loader: &mut ClassLoader,
    classname: &str,
  ) -> Result<types::Type> {
    if self.classes.contains_key(classname) {
      return Ok(types::Type::ObjectRef(
        *self
          .classes
          .get(classname)
          .ok_or_eyre(eyre!("something went wrong in alloc class obj"))?,
      ));
    }

    let mut fields = vec![];

    // adding component type information if array
    if classname.starts_with("[") {
      let comp_type = self.get_component_type(loader, classname)?;

      fields.push(("componentType", comp_type));
    }

    let string = self.alloc_string(loader, &class_to_dotclass(classname))?;

    fields.push(("name", string));

    let obj_ref = self.alloc_obj(loader, "java/lang/Class")?.as_ref()?;

    let obj_mod = self.get_obj_instance_mut(obj_ref)?;

    for f in fields {
      obj_mod.new_field(f.0, f.1)?;
    }

    // TODO: Check which ClassLoader is loading it.
    // TODO: if part of the bootstrap classLoader is NULL
    obj_mod.put_field("classLoader", types::Type::Null)?;

    if !Type::is_primitive(classname) && !classname.starts_with("[") {
      let module = loader.modulemanager.get_module_by_class(classname)?;

      obj_mod.put_field("module", types::Type::ObjectRef(module.get_obj_ref()))?;
    }

    self.classes.insert(classname.to_string(), obj_ref);

    if !Type::is_primitive(classname) && !classname.starts_with("[") {
      let static_class_obj_ref = self.alloc_obj(loader, classname)?.as_ref()?;
      self
        .static_classes
        .insert(classname.to_string(), static_class_obj_ref);
    }

    Ok(types::Type::ObjectRef(obj_ref))
  }

  /// Allocate a Class<T> object for a primitive type
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class to instantiate
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated object
  pub fn alloc_primitive_class_obj(
    &mut self,
    loader: &mut ClassLoader,
    classname: &str,
  ) -> Result<types::Type> {
    if self.classes.contains_key(classname) {
      return Ok(types::Type::ObjectRef(
        *self
          .classes
          .get(classname)
          .ok_or_eyre(eyre!("something went wrong in alloc class obj"))?,
      ));
    }

    let name_ref = self.alloc_string(loader, &class_to_dotclass(classname))?;

    let obj_ref = self.alloc_obj(loader, "java/lang/Class")?.as_ref()?;

    let obj_mod = self.get_obj_instance_mut(obj_ref)?;

    obj_mod.new_field("name", name_ref)?;

    // TODO: Check which ClassLoader is loading it.
    // TODO: if part of the bootstrap classLoader is NULL
    obj_mod.put_field("classLoader", types::Type::Null)?;

    // using something that is for sure in java.base
    let module = loader
      .modulemanager
      .get_module_by_class("java/lang/Integer")?;

    obj_mod.put_field("module", types::Type::ObjectRef(module.get_obj_ref()))?;

    self.classes.insert(classname.to_string(), obj_ref);

    Ok(types::Type::ObjectRef(obj_ref))
  }

  /// Allocate an array of non primitive types
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class of elements to instantiate (so if you want an array of
  ///   java/lang/Integer, you pass `java/lang/Integer` and NOT `[Ljava/lang/Integer;`
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated array
  pub fn alloc_array(
    &mut self,
    classname: &str,
    array: Vec<types::Type>,
    size: usize, // size is used when array is empty -> fill in default values
  ) -> Result<types::Type> {
    if types::Type::is_primitive(classname) {
      return self.alloc_array_primitive(classname, array, size);
    }

    let curr_array_ref = self.get_next_obj_ref();

    let array_len = array.len();

    let mut elements = array;

    if array_len == 0 {
      let def_value = get_default_value(classname);

      for _ in 0..size {
        elements.push(def_value);
      }
    }

    let array = ArrayInstance::new(classname, curr_array_ref, elements)?;

    self
      .heap
      .insert(curr_array_ref, types::Instance::ArrayInstance(array));

    Ok(types::Type::ArrayRef(curr_array_ref))
  }

  /// Allocate an array of primitive types
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class of elements to instantiate
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated array
  pub fn alloc_array_primitive(
    &mut self,
    classname: &str,
    array: Vec<types::Type>,
    size: usize, // size is used when array is empty -> fill in default values
  ) -> Result<types::Type> {
    let curr_array_ref = self.get_next_obj_ref();

    let array_len = array.len();

    let mut elements = array;

    if array_len == 0 {
      let def_value = get_default_value(classname);

      for _ in 0..size {
        elements.push(def_value);
      }
    }

    let array = ArrayInstance::new(classname, curr_array_ref, elements)?;

    self
      .heap
      .insert(curr_array_ref, types::Instance::ArrayInstance(array));

    Ok(types::Type::ArrayRef(curr_array_ref))
  }

  /// Allocate an multi dimenstional array
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class of elements to instantiate (e.g. [[B to create a 2D array of bytes)
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated array
  pub fn alloc_multiarray(
    &mut self,
    classname: &str,
    dims: &[usize], // size is used when array is empty -> fill in default values
  ) -> Result<types::Type> {
    let curr_array_ref = self.get_next_obj_ref();

    let classname = &get_array_element_class_name(classname);

    assert!(!dims.is_empty());

    if dims.len() == 1 {
      return self.alloc_array(classname, vec![], dims[0]);
    }

    // multi dimension
    let first_dim = dims[0];

    let rest_dims = &dims[1..];

    // Allocate top-level array
    let mut elements = Vec::with_capacity(first_dim);

    for _ in 0..first_dim {
      // Recursive: allocate sub-arrays
      let element = self.alloc_multiarray(classname, rest_dims)?;

      elements.push(element);
    }

    let array = ArrayInstance::new(classname, curr_array_ref, elements)?;

    self
      .heap
      .insert(curr_array_ref, types::Instance::ArrayInstance(array));

    Ok(types::Type::ArrayRef(curr_array_ref))
  }

  /// Allocate a java/lang/reflect/Field
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `classname` - Class of elements to instantiate
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated array
  pub fn alloc_reflect_field(
    &mut self,
    loader: &mut ClassLoader,
    classname: &str,
    field: &fields::FieldInfo,
  ) -> Result<types::Type> {
    // not sure about signature and type class
    warn!("alloc reflec field not fully implemented");
    let obj_ref = self
      .alloc_obj(loader, "java/lang/reflect/Field")?
      .as_ref()?;

    let class_ref = self.get_class_instance(loader, classname)?.get_ref();
    let name_ref = self.alloc_string(loader, &class_to_dotclass(field.get_name()))?;
    let type_class_ref = self
      .get_class_instance(loader, &descriptor_to_classname(field.get_descriptor()))?
      .get_ref();
    let sig_ref = self.alloc_string(loader, field.get_descriptor())?;

    let obj = self.get_obj_instance_mut(obj_ref)?;

    obj.put_field("clazz", types::Type::ObjectRef(class_ref))?;
    obj.put_field("name", name_ref)?;
    obj.put_field("type", types::Type::ObjectRef(type_class_ref))?;
    obj.put_field(
      "modifiers",
      types::Type::Integer(field.get_access_flags() as i32),
    )?;
    obj.put_field("signature", sig_ref)?;

    Ok(types::Type::ObjectRef(obj_ref))
  }

  pub fn get_instance(&self, obj_ref: ju4) -> Result<&types::Instance> {
    self
      .heap
      .get(&obj_ref)
      .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))
  }

  pub fn get_instance_mut(&mut self, obj_ref: ju4) -> Result<&mut types::Instance> {
    self
      .heap
      .get_mut(&obj_ref)
      .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))
  }

  pub fn get_obj_instance(&self, obj_ref: ju4) -> Result<&types::ObjectInstance> {
    let obj_enum = self
      .heap
      .get(&obj_ref)
      .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))?;

    if let types::Instance::ObjectInstance(obj) = obj_enum {
      return Ok(obj);
    }

    Err(eyre!(errors::InternalError::WrongInstance(
      "ObjectInstance",
      obj_enum.clone()
    )))
  }

  pub fn get_obj_instance_mut(&mut self, obj_ref: ju4) -> Result<&mut types::ObjectInstance> {
    let obj_enum = self
      .heap
      .get_mut(&obj_ref)
      .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))?;

    if let types::Instance::ObjectInstance(obj) = obj_enum {
      return Ok(obj);
    }

    Err(eyre!(errors::InternalError::WrongInstance(
      "ObjectInstance",
      obj_enum.clone()
    )))
  }

  pub fn get_array_instance(&self, obj_ref: ju4) -> Result<&types::ArrayInstance> {
    let obj_enum = self
      .heap
      .get(&obj_ref)
      .ok_or_eyre(format!("ArrayRef not found in heap: {:?}", obj_ref))?;

    if let types::Instance::ArrayInstance(obj) = obj_enum {
      return Ok(obj);
    }

    Err(eyre!(errors::InternalError::WrongInstance(
      "ArrayInstance",
      obj_enum.clone()
    )))
  }

  pub fn get_array_instance_mut(&mut self, obj_ref: ju4) -> Result<&mut types::ArrayInstance> {
    let obj_enum = self
      .heap
      .get_mut(&obj_ref)
      .ok_or_eyre(format!("ArrayRef not found in heap: {:?}", obj_ref))?;

    if let types::Instance::ArrayInstance(obj) = obj_enum {
      return Ok(obj);
    }

    Err(eyre!(errors::InternalError::WrongInstance(
      "ArrayInstance",
      obj_enum.clone()
    )))
  }

  pub fn get_string(&self, obj_ref: ju4) -> Result<String> {
    let obj = self.get_obj_instance(obj_ref)?;

    assert!(obj.get_classname() == "java/lang/String");

    let array_field = obj.get_field("value")?;

    if let types::Type::ArrayRef(array_ref) = array_field {
      let array = self.get_array_instance(array_ref)?;

      // this is the name of T class in Class<T>
      let name = array.get_string()?;

      return Ok(name);
    }

    Err(eyre!(errors::InternalError::WrongType(
      "Array",
      array_field
    )))
  }

  /// Get class file from class ref Class<T>
  ///
  /// # Arguments
  ///
  /// * `class_ref` - reference id of the class in the heap
  ///
  /// # Returns
  ///
  /// Class file in Class<T>
  /// example: Class<String> -> class file of java/lang/String
  ///
  /// # Note
  ///
  /// does not work for primitive types
  pub fn get_class_from_class_obj<'a>(
    &'a self,
    loader: &'a mut ClassLoader,
    class_ref: ju4,
  ) -> Result<RwLockReadGuard<'a, class_file::ClassFile>> {
    let class_obj = self.get_obj_instance(class_ref)?;

    let class_inner_name_ref = class_obj.get_field("name")?.as_ref()?; // T in Class<T> as string
    let class_inner_name = &self.get_string(class_inner_name_ref)?;

    loader.get(class_inner_name)
  }

  /// Get class name from class ref Class<T>
  ///
  /// # Arguments
  ///
  /// * `class_ref` - reference id of the class in the heap
  ///
  /// # Returns
  ///
  /// Class name in Class<T>
  /// example: Class<String> -> java/lang/String
  ///
  /// # Note
  ///
  /// works for primitive types
  pub fn get_classname_from_class_obj(&self, class_ref: ju4) -> Result<String> {
    let class_obj = self.get_obj_instance(class_ref)?;

    let class_inner_name_ref = class_obj.get_field("name")?.as_ref()?; // T in Class<T> as string
    Ok(dotclass_to_class(&self.get_string(class_inner_name_ref)?))
  }

  pub fn get_class_instance(
    &mut self,
    loader: &mut ClassLoader,
    classname: &str,
  ) -> Result<&types::ObjectInstance> {
    if !self.has_class_instance(classname) {
      self.alloc_class_obj(loader, classname)?;
    }
    let class_ref = self
      .classes
      .get(classname)
      .ok_or_eyre(format!("ClassObject not found in heap: {:?}", classname))?;

    self.get_obj_instance(*class_ref)
  }

  pub fn get_class_instance_mut(&mut self, classname: &str) -> Result<&mut types::ObjectInstance> {
    let class_ref = self
      .classes
      .get(classname)
      .ok_or_eyre(format!("ClassObject not found in heap: {:?}", classname))?;

    self.get_obj_instance_mut(*class_ref)
  }

  pub fn get_static_class_instance(&self, classname: &str) -> Result<&types::ObjectInstance> {
    let class_ref = self
      .static_classes
      .get(classname)
      .ok_or_eyre(format!("ClassObject not found in heap: {:?}", classname))?;

    self.get_obj_instance(*class_ref)
  }

  pub fn has_class_instance(&self, classname: &str) -> bool {
    self.classes.contains_key(classname)
  }

  pub fn clone_instance(&mut self, obj_ref: ju4) -> Result<types::Type> {
    let instance = self.get_instance(obj_ref)?.clone();

    match instance {
      types::Instance::ObjectInstance(ref obj) => {
        let cp_ref = self.clone_obj_instance(obj)?;

        Ok(types::Type::ObjectRef(cp_ref))
      }
      types::Instance::ArrayInstance(ref array) => {
        let cp_ref = self.clone_array_instance(array);

        Ok(types::Type::ArrayRef(cp_ref))
      }
    }
  }

  pub fn clone_obj_instance(&mut self, obj: &ObjectInstance) -> Result<ju4> {
    // TODO: not sure about how I create a new object
    let curr_ref = self.get_next_obj_ref();

    // this should clone maps and fields
    let mut new_obj = obj.clone();
    new_obj.set_ref(curr_ref);

    if let Some(parent_obj) = obj.get_parent() {
      let parent_ref = self.clone_obj_instance(&parent_obj.clone())?;
      let parent_obj = self.get_obj_instance(parent_ref)?;
      new_obj.set_parent(Arc::new(RwLock::new(parent_obj.clone())));
    }

    self
      .heap
      .insert(curr_ref, types::Instance::ObjectInstance(new_obj));

    Ok(curr_ref)
  }

  pub fn clone_array_instance(&mut self, array: &ArrayInstance) -> ju4 {
    let curr_ref = self.get_next_obj_ref();
    let mut copy = array.clone();
    copy.set_ref(curr_ref);
    self
      .heap
      .insert(curr_ref, types::Instance::ArrayInstance(copy));
    curr_ref
  }

  /// Allocate a java/lang/reflect/Method
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `declaring_classname` - Class of the method
  /// * `method` - Method to embed into java/lang/reflect/Method
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated java/lang/reflect/Method
  pub fn alloc_reflect_method(
    &mut self,
    loader: &mut ClassLoader,
    declaring_classname: &str,
    method: &methods::MethodInfo,
  ) -> Result<types::Type> {
    warn!("reflect method not fully implemented");
    let declaring_classname = &dotclass_to_class(declaring_classname);

    // TODO: add propert types for arguments
    let param_types = method.get_descriptor();

    let signature = self.alloc_string(loader, method.get_descriptor())?;

    let ret_type = get_return_type_descriptor(param_types);

    if !self.has_class_instance(&ret_type) {
      debug!("ALLOC {}", ret_type);

      self.alloc_class_obj(loader, &ret_type)?;
    }

    let ret_class_obj_ref = self.get_class_instance(loader, &ret_type)?.get_ref();

    let param_types = get_argument_classnames(param_types);

    let mut param_types_args = vec![];

    for p in param_types {
      let p = &descriptor_to_classname(&p);

      if !self.has_class_instance(p) {
        self.alloc_class_obj(loader, p)?;
      }

      let class = self.get_class_instance(loader, p)?;

      param_types_args.push(types::Type::ObjectRef(class.get_ref()));
    }

    let param_types_args_len = param_types_args.len();

    let param_array =
      self.alloc_array("java/lang/Class", param_types_args, param_types_args_len)?;

    // Allocate a new java/lang/reflect/Method instance
    let method_ref = self.alloc_obj(loader, "java/lang/reflect/Method")?;

    // clazz: reference to Class object
    let class_obj_ref = self
      .get_class_instance(loader, declaring_classname)?
      .get_ref();

    let name_string = self.alloc_string(loader, &class_to_dotclass(method.get_name()))?;

    let method_instance = self.get_obj_instance_mut(method_ref.as_ref()?)?;

    method_instance.new_field("clazz", types::Type::ObjectRef(class_obj_ref))?;

    // name: java/lang/String
    method_instance.new_field("name", name_string)?;

    // modifiers: int
    method_instance.new_field(
      "modifiers",
      types::Type::Integer(method.get_access_flags() as i32),
    )?;

    method_instance.new_field("parameterTypes", param_array)?;

    method_instance.new_field("signature", signature)?;

    // returnType: java/lang/Class
    method_instance.new_field("returnType", types::Type::ObjectRef(ret_class_obj_ref))?;

    Ok(method_ref)
  }

  /// Allocate a java/lang/Integer
  ///
  /// # Arguments
  ///
  /// * `loader` - ClassLoader used to resolve the class to instantiate
  /// * `num` - Value of the integer
  ///
  /// # Returns
  ///
  /// A `ObjectRef` of the newly instantiated java/lang/Integer
  pub(crate) fn alloc_integer(
    &mut self,
    loader: &mut ClassLoader,
    num: i32,
  ) -> Result<types::Type> {
    let obj_ref = self.alloc_obj(loader, "java/lang/Integer")?.as_ref()?;

    let obj = self.get_obj_instance_mut(obj_ref)?;

    obj.put_field("value", types::Type::Integer(num))?;

    Ok(types::Type::ObjectRef(obj_ref))
  }
}
