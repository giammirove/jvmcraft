use std::{
  collections::HashMap,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use color_eyre::eyre::{eyre, OptionExt, Result};
use log::{debug, warn};

use super::{
  class_file::{ClassFile, InvokeDynamicResolved, MethodHandleResolved},
  fields::FieldInfo,
  methods::MethodInfo,
};
use crate::{
  class_file,
  class_loader::attributes,
  runtime::{errors, modulemanager::ModuleManager, types},
  utils::{dotclass_to_class, ju2},
};

#[derive(Debug)]
pub struct ClassLoader {
  pub(crate) modulemanager: ModuleManager,
  classes: HashMap<String, Arc<RwLock<class_file::ClassFile>>>,
}

impl ClassLoader {
  pub fn new() -> Self {
    ClassLoader {
      modulemanager: ModuleManager::new(),
      classes: HashMap::new(),
    }
  }

  pub fn add(&mut self, classname: String) -> Result<()> {
    self.classes.insert(
      classname.to_string(),
      class_file::ClassFile::parse(classname + ".class")?,
    );

    Ok(())
  }

  pub fn add_class_file(
    &mut self,
    module: &str,
    classname: &str,
    class_file: Arc<RwLock<ClassFile>>,
  ) -> Result<()> {
    self.modulemanager.add_to_module(module, classname)?;
    self.classes.insert(classname.to_string(), class_file);

    Ok(())
  }

  pub(crate) fn load_class(&mut self, name: &str) -> Result<()> {
    let name = &dotclass_to_class(name);
    debug!("Load class {}", name);
    let class = if name.starts_with("[") {
      class_file::ClassFile::create_array(name.to_string())?
    } else {
      // find module for the class
      let path = {
        let module = self.modulemanager.get_module_by_class(name)?;

        format!("{}/{}.class", module.get_location(), name)
      };

      class_file::ClassFile::parse(path)?
    };

    self.classes.insert(name.to_string(), class);

    Ok(())
  }

  pub fn get_method_max_locals_by_name(
    &mut self,
    classname: &str,
    method_name: &str,
    type_str: &str,
  ) -> Result<u16> {
    let (_, code) = self.get_method_code_by_name(classname, method_name, type_str)?;

    Ok(code.get_max_locals())
  }

  // returns class name where the method `method_name` is found and its info
  pub fn get_method_code_by_name(
    &mut self,
    classname: &str,
    method_name: &str,
    type_str: &str,
  ) -> Result<(String, attributes::Code)> {
    let (method_class, method, _) =
      self.get_method_by_name_with_index(classname, method_name, type_str)?;
    Ok((
      method_class,
      method
        .get_code()
        .ok_or_eyre(eyre!(errors::InternalError::CodeNotFound(
          classname.to_string(),
          method_name.to_string(),
          type_str.to_string()
        )))?
        .clone(),
    ))
  }

  pub fn get_method_by_name(
    &mut self,
    classname: &str,
    method_name: &str,
    type_str: &str,
  ) -> Result<(String, MethodInfo)> {
    let (method_class, method, _) =
      self.get_method_by_name_with_index(classname, method_name, type_str)?;
    Ok((method_class, method))
  }

  /// Get Method by Name with Index (for vtable)
  ///
  /// # Arguments
  ///
  /// * `classname` - Class of the method
  /// * `method_name` - Name of the method
  /// * `type_str` - Type of the method (e.g. java/lang/Integer)
  /// * `allow_abstract` - Whether to allowed the method to be abstract (no code attribute)
  /// * `_increment` - Used for recursion
  ///
  /// # Returns
  ///
  /// Triple as (class: String, method: MethodInfo, index: i32)
  /// where `class` is the name of class where the field was found (can be
  /// `classname` or a parent)
  /// If the method is found in a interface, it is returned only if it has a body
  /// (i.e. implemented with `default` keyword)
  fn _get_method_by_name_with_index(
    &mut self,
    classname: &str,
    method_name: &str,
    type_str: &str,
    allow_abstract: bool,
    _increment: usize,
  ) -> Result<(String, MethodInfo, i32)> {
    let class = self.get(classname)?;

    let mut vtable_index = 0;

    let methods = class.get_methods();

    for method in methods {
      if (allow_abstract || method.has_code())
        && method_name == method.get_name()
        && (method.has_polymorphic_signature() || type_str == method.get_descriptor())
      {
        // static methods are not part of the vtable
        // constructors are special and not part of the vtable (they are resolved directly)
        if method.is_static() || method_name == "<init>" {
          return Ok((classname.to_string(), method.clone(), -1));
        } else {
          return Ok((classname.to_string(), method.clone(), vtable_index));
        }
      }
      vtable_index += 1;
    }

    let has_parent = class.has_parent();

    let interfaces = class.get_interfaces().clone();
    drop(class);

    for interface_name in interfaces {
      let interface = self.get(&interface_name)?;
      let interface_methods = interface.get_methods();
      for method in interface_methods {
        if (allow_abstract || method.has_code())
          && method_name == method.get_name()
          && type_str == method.get_descriptor()
        {
          return Ok((interface_name, method.clone(), vtable_index));
        }
        vtable_index += 1;
      }
    }

    if has_parent {
      let parent_name = self.get(classname)?.get_parent_name().to_owned();

      self._get_method_by_name_with_index(
        &parent_name,
        method_name,
        type_str,
        allow_abstract,
        _increment + vtable_index as usize,
      )
    } else {
      Err(eyre!(errors::InternalError::MethodNotFound(
        classname.to_string(),
        method_name.to_string(),
        type_str.to_string(),
      )))
    }
  }

  /// Get Method by Name with Index (for vtable) (no abstract methods)
  ///
  /// # Arguments
  ///
  /// * `classname` - Class of the method
  /// * `method_name` - Name of the method
  /// * `type_str` - Type of the method (e.g. java/lang/Integer)
  /// * `_increment` - Used for recursion
  ///
  /// # Returns
  ///
  /// Triple as (class: String, method: MethodInfo, index: i32)
  /// where `class` is the name of class where the field was found (can be
  /// `classname` or a parent)
  /// If the method is found in a interface, it is returned only if it has a body
  /// (i.e. implemented with `default` keyword)
  pub fn get_method_by_name_with_index(
    &mut self,
    classname: &str,
    method_name: &str,
    type_str: &str,
  ) -> Result<(String, MethodInfo, i32)> {
    self._get_method_by_name_with_index(classname, method_name, type_str, false, 0)
  }

  /// Get Method by Name with Index (for vtable) (including abstract methods)
  ///
  /// # Arguments
  ///
  /// * `classname` - Class of the method
  /// * `method_name` - Name of the method
  /// * `type_str` - Type of the method (e.g. java/lang/Integer)
  /// * `_increment` - Used for recursion
  ///
  /// # Returns
  ///
  /// Triple as (class: String, method: MethodInfo, index: i32)
  /// where `class` is the name of class where the field was found (can be
  /// `classname` or a parent)
  /// If the method is found in a interface, it is returned only if it has a body
  /// (i.e. implemented with `default` keyword)
  pub fn get_any_method_by_name_with_index(
    &mut self,
    classname: &str,
    method_name: &str,
    type_str: &str,
  ) -> Result<(String, MethodInfo, i32)> {
    self._get_method_by_name_with_index(classname, method_name, type_str, true, 0)
  }

  /// Get Field by Name with Index (for vtable)
  ///
  /// # Arguments
  ///
  /// * `classname` - Class of the field
  /// * `field_name` - Name of the field
  /// * `type_str` - Type of the field (e.g. java/lang/Integer)
  /// * `_increment` - Used for recursion
  ///
  /// # Returns
  ///
  /// Triple as (class: String, method: FieldInfo, index: i32)
  /// where `class` is the name of class where the field was found (can be
  /// `classname` or a parent)
  pub fn get_field_by_name_with_index(
    &mut self,
    classname: &str,
    field_name: &str,
    type_str: &str,
    _increment: usize,
  ) -> Result<(String, FieldInfo, i32)> {
    debug!(
      "get field by name {} {} {}",
      classname, field_name, type_str
    );
    let class = self.get(classname)?;

    let num_fields = class.get_fields().len();

    for i in 0..num_fields {
      let field = class.get_fields().get(i).unwrap();
      debug!("{}", field);

      if field_name == field.get_name() && type_str == field.get_descriptor() {
        return Ok((classname.to_string(), field.clone(), i as i32));
      }
    }

    if class.has_parent() {
      drop(class);

      let parent_name = self.get(classname)?.get_parent_name().to_owned();

      self.get_field_by_name_with_index(&parent_name, field_name, type_str, _increment + num_fields)
    } else {
      Err(eyre!(
        "field not found: {:?} {:?} in {:?}",
        field_name,
        type_str,
        classname
      ))
    }
  }

  // get field ref info as strings
  pub fn resolve_field_ref(
    &mut self,
    classname: &str,
    index: ju2,
  ) -> Result<(String, String, String)> {
    let class = self.get(classname)?;

    match class.resolve_field_ref(index) {
      a @ Ok(_) => a,
      _ => {
        if class.has_parent() {
          drop(class);

          let parent_name = self.get(classname)?.get_parent_name().to_owned();

          self.resolve_field_ref(&parent_name, index)
        } else {
          Err(eyre!("method not found: {:?} in {:?}", index, classname))
        }
      }
    }
  }

  pub fn resolve_method_ref(
    &mut self,
    classname: &str,
    index: ju2,
  ) -> Result<(String, String, String)> {
    let class = self.get(classname)?;

    match class.resolve_method_ref(index) {
      a @ Ok(_) => a,
      e => {
        // check interfaces
        let has_parent = class.has_parent();
        drop(class);

        if has_parent {
          let parent_name = self.get(classname)?.get_parent_name().to_owned();

          self.resolve_method_ref(&parent_name, index)
        } else {
          Err(eyre!(
            "method not found: {:?} in {:?} -> {:?}",
            index,
            classname,
            e
          ))
        }
      }
    }
  }

  pub fn resolve_method_handle(
    &mut self,
    classname: &str,
    index: ju2,
  ) -> Result<MethodHandleResolved> {
    let class = self.get(classname)?;

    match class.resolve_method_handle(index) {
      a @ Ok(_) => a,
      e => {
        if class.has_parent() {
          drop(class);

          let parent_name = self.get(classname)?.get_parent_name().to_owned();

          self.resolve_method_handle(&parent_name, index)
        } else {
          Err(eyre!(
            "methodhandle not found: {:?} in {:?} -> {:?}",
            index,
            classname,
            e
          ))
        }
      }
    }
  }

  pub fn resolve_method_type(&mut self, classname: &str, index: ju2) -> Result<String> {
    let class = self.get(classname)?;

    match class.resolve_method_type(index) {
      a @ Ok(_) => a,
      e => {
        if class.has_parent() {
          drop(class);

          let parent_name = self.get(classname)?.get_parent_name().to_owned();

          self.resolve_method_type(&parent_name, index)
        } else {
          Err(eyre!(
            "methodtype not found: {:?} in {:?} -> {:?}",
            index,
            classname,
            e
          ))
        }
      }
    }
  }

  pub fn resolve_invokedynamic(
    &mut self,
    classname: &str,
    index: ju2,
  ) -> Result<InvokeDynamicResolved> {
    let class = self.get(classname)?;

    match class.resolve_invokedynamic(index) {
      a @ Ok(_) => a,
      e => {
        if class.has_parent() {
          drop(class);

          let parent_name = self.get(classname)?.get_parent_name().to_owned();

          self.resolve_invokedynamic(&parent_name, index)
        } else {
          Err(eyre!(
            "invokedynamic not found: {:?} in {:?} -> {:?}",
            index,
            classname,
            e
          ))
        }
      }
    }
  }

  pub fn resolve_string(&mut self, classname: &str, index: ju2) -> Result<String> {
    let class = self.get(classname)?;

    match class.resolve_name(index) {
      a @ Ok(_) => a,
      e => {
        if class.has_parent() {
          drop(class);

          let parent_name = self.get(classname)?.get_parent_name().to_owned();

          self.resolve_string(&parent_name, index)
        } else {
          Err(eyre!(
            "string not found: {:?} in {:?} -> {:?}",
            index,
            classname,
            e
          ))
        }
      }
    }
  }

  /// get general lock
  pub fn get_lock(&mut self, name: &str) -> Result<Arc<RwLock<class_file::ClassFile>>> {
    let name = &dotclass_to_class(name);
    if !self.classes.contains_key(name) {
      self.load_class(name)?;
    }

    Ok(self.classes.get(name).unwrap().clone())
  }

  /// get lock read
  pub fn get(&mut self, name: &str) -> Result<RwLockReadGuard<class_file::ClassFile>> {
    let name = &dotclass_to_class(name);
    if !self.classes.contains_key(name) {
      self.load_class(name)?;
    }

    Ok(
      self
        .classes
        .get(name)
        .ok_or_eyre(format!("class {:?} not found using get", name))?
        .read()
        .unwrap(),
    )
  }

  /// get lock write
  pub fn get_mut(&mut self, name: &str) -> Result<RwLockWriteGuard<class_file::ClassFile>> {
    let name = &dotclass_to_class(name);
    if !self.classes.contains_key(name) {
      self.load_class(name)?;
    }

    Ok(
      self
        .classes
        .get_mut(name)
        .ok_or_eyre(format!("class {:?} not found using get_mut", name))?
        .write()
        .unwrap(),
    )
  }

  pub fn get_field_offset(&mut self, classname: &str, field_name: &str) -> Result<i64> {
    let class = self.get(&dotclass_to_class(classname))?;

    warn!("Super classes not checked properly in get_field_offset");

    let fields = class.get_fields();

    for f in fields {
      debug!("{:?}", f.get_name())
    }

    match class.get_field_offset(field_name) {
      r @ Ok(_) => r,
      _ => {
        if class.has_parent() {
          let parent_name = class.get_parent_name().to_string();

          drop(class);

          self.get_field_offset(&parent_name, field_name)
        } else {
          panic!()
        }
      }
    }
  }

  pub fn get_field_by_offset(&mut self, classname: &str, offset: i64) -> Result<FieldInfo> {
    let class = self.get(&dotclass_to_class(classname))?;

    warn!("Super classes not checked properly in get_field_by_offset");

    match class.get_field_by_offset(offset) {
      r @ Ok(_) => r,
      _ => {
        if class.has_parent() {
          let parent_name = class.get_parent_name().to_string();

          drop(class);

          self.get_field_by_offset(&parent_name, offset)
        } else {
          panic!("get field by offset")
        }
      }
    }
  }

  // does right implements left ?
  pub fn has_interface(&mut self, left: &str, right: &str) -> Result<bool> {
    if left == right {
      return Ok(true);
    }

    let right_class = self.get(right)?;

    let right_interfaces = right_class.get_interfaces().clone();

    drop(right_class);

    for interface in right_interfaces {
      if self.has_interface(left, &interface)? {
        return Ok(true);
      }
    }

    let right_class = self.get(right)?;

    if right_class.has_parent() {
      drop(right_class);

      let parent_name = self.get(right)?.get_parent_name().to_owned();

      self.has_interface(left, &parent_name)
    } else {
      Ok(false)
    }
  }

  pub fn is_method_native(
    &mut self,
    classname: &str,
    method_name: &str,
    method_type: &str,
  ) -> Result<bool> {
    let (_, method) = self.get_method_by_name(classname, method_name, method_type)?;
    Ok(method.is_native())
  }

  pub fn get_static_field(&mut self, classname: &str, field_name: &str) -> Result<types::Type> {
    let class = self.get(classname)?;

    match class.get_static_field(field_name) {
      Ok(v) => Ok(*v),
      _ => {
        if class.has_parent() {
          drop(class);

          let parent_name = self.get(classname)?.get_parent_name().to_owned();

          self.get_static_field(&parent_name, field_name)
        } else {
          Err(eyre!(
            "static field not found: {:?} in {:?}",
            field_name,
            classname
          ))
        }
      }
    }
  }
}
