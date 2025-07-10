use core::panic;
use std::{
  collections::HashMap,
  fmt, fs,
  path::Path,
  sync::{Arc, RwLock},
};

use color_eyre::eyre::{eyre, OptionExt, Result};
use log::debug;

use crate::{
  class_loader::{attributes, constant_pool, fields, interfaces, methods},
  runtime::{errors, types, types::Type},
  utils::*,
};

use super::fields::FieldInfo;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct MethodHandleResolved {
  ref_kind: ju1,
  class_name: String,  // metafactory class
  method_name: String, // metafactory method
  method_type: String, // metafactory method descriptor
}

impl fmt::Display for MethodHandleResolved {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl MethodHandleResolved {
  pub(crate) fn get_ref_kind(&self) -> ju1 {
    self.ref_kind
  }

  pub(crate) fn get_classname(&self) -> &str {
    &self.class_name
  }

  pub(crate) fn get_method_name(&self) -> &str {
    &self.method_name
  }

  pub(crate) fn get_method_type(&self) -> &str {
    &self.method_type
  }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct InvokeDynamicResolved {
  bootstrap: MethodHandleResolved,
  bootstrap_arguments: Vec<ju2>, // metafactory lambda arguments (after lookup, methodname,
  // methodtype)
  method_name: String,
  method_type: String,
}

impl fmt::Display for InvokeDynamicResolved {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl InvokeDynamicResolved {
  pub(crate) fn _get_bootstrap_ref_kind(&self) -> ju1 {
    self.bootstrap.get_ref_kind()
  }

  pub(crate) fn get_bootstrap_class_name(&self) -> &str {
    &self.bootstrap.class_name
  }

  pub(crate) fn get_bootstrap_method_name(&self) -> &str {
    &self.bootstrap.method_name
  }

  pub(crate) fn get_bootstrap_method_type(&self) -> &str {
    &self.bootstrap.method_type
  }

  pub(crate) fn get_method_name(&self) -> &str {
    &self.method_name
  }

  pub(crate) fn get_method_type(&self) -> &str {
    &self.method_type
  }

  pub(crate) fn get_arguments(&self) -> &Vec<ju2> {
    &self.bootstrap_arguments
  }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ClassFile {
  magic: ju4,
  minor_version: ju2,
  major_version: ju2,
  constant_pool: constant_pool::ConstantPool,
  access_flags: ju2,
  this_class: ju2,
  super_class: ju2,
  interfaces: interfaces::Interfaces,
  // contains static and non static fields (non static due to reflection purposes)
  fields: fields::Fields,
  methods: methods::Methods,
  attributes: attributes::Attributes,

  is_init: bool,
  // use to get field info given a index in the constan pool
  static_fields: HashMap<String, Type>,

  this_class_name: String,
  super_class_name: String,
}

impl fmt::Display for ClassFile {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "{}", self.constant_pool)
  }
}

impl ClassFile {
  pub fn parse(file_path: String) -> Result<Arc<RwLock<Self>>> {
    debug!("=================================================================");

    debug!("[-] Parsing {:?}", file_path);

    let path = Path::new(&file_path);

    if !path.exists() {
      return Err(eyre!(errors::InternalError::ClassNotFound(file_path)));
    }

    let data: Vec<u8> = fs::read(file_path)?;

    ClassFile::parse_from_bytes(&data)
  }

  pub fn parse_from_bytes(data: &[u8]) -> Result<Arc<RwLock<Self>>> {
    let mut index = 0;
    let magic = ju4_from_bytes(get_slice(data, index, 4))?;

    index += 4;

    let minor_version = ju2_from_bytes(get_slice(data, index, 2))?;

    index += 2;

    let major_version = ju2_from_bytes(get_slice(data, index, 2))?;

    index += 2;

    let (constant_pool, bytes_read) = constant_pool::ConstantPool::parse(&data[index..])?;

    index += bytes_read;

    let access_flags = ju2_from_bytes(get_slice(data, index, 2))?;

    index += 2;

    let this_class = ju2_from_bytes(get_slice(data, index, 2))?;

    index += 2;

    let super_class = ju2_from_bytes(get_slice(data, index, 2))?;

    index += 2;

    let (interfaces, bytes_read) = interfaces::Interfaces::parse(&data[index..], &constant_pool)?;

    index += bytes_read;

    let (fields, bytes_read) = fields::Fields::parse(&data[index..], &constant_pool)?;

    index += bytes_read;

    let (methods, bytes_read) = methods::Methods::parse(&data[index..], &constant_pool)?;

    index += bytes_read;

    let (attributes, _) = attributes::Attributes::parse(&data[index..], &constant_pool)?;

    let mut static_fields = HashMap::new();

    for field in fields.get_fields() {
      if field.is_static() {
        let name = field.get_name();

        let field_type: &str = field.get_descriptor();

        let value = get_default_value(field_type);

        static_fields.insert(name.to_string(), value);
      }
    }

    let this_class_name = constant_pool.resolve_class_name(this_class)?;

    let super_class_name = constant_pool.resolve_class_name(super_class)?;

    debug!("=================================================================");

    Ok(Arc::new(RwLock::new(ClassFile {
      magic,
      major_version,
      minor_version,
      constant_pool,
      access_flags,
      this_class,
      super_class,
      interfaces,
      fields,
      methods,
      attributes,

      is_init: false,
      static_fields,
      this_class_name,
      super_class_name,
    })))
  }

  pub fn get_access_flags(&self) -> ju2 {
    self.access_flags
  }

  pub fn create_array(array_type: String) -> Result<Arc<RwLock<Self>>> {
    debug!("=================================================================");

    debug!("[-] Create {:?}", array_type);

    let magic = 0;

    let minor_version = 0;

    let major_version = 0;

    let constant_pool = constant_pool::ConstantPool::empty();

    let access_flags = 0;

    let this_class = 0;

    let super_class = 0;

    let mut interfaces = interfaces::Interfaces::empty();

    interfaces.add_interface("java/lang/Cloneable".to_string());

    interfaces.add_interface("java/io/Serializable".to_string());

    let fields = fields::Fields::empty();

    let methods = methods::Methods::empty();

    let attributes = attributes::Attributes::empty();

    let static_fields = HashMap::new();

    let this_class_name = array_type;

    let super_class_name = "java/lang/Object".to_string();

    debug!("=================================================================");

    Ok(Arc::new(RwLock::new(ClassFile {
      magic,
      major_version,
      minor_version,
      constant_pool,
      access_flags,
      this_class,
      super_class,
      interfaces,
      fields,
      methods,
      attributes,

      is_init: false,
      static_fields,
      this_class_name,
      super_class_name,
    })))
  }

  /// Create a new Object instance of this class
  ///
  /// # Returns
  ///
  /// A `ObjectInstance` of this class
  pub fn new_obj<'a>(&'a self, new_obj: &'a mut types::ObjectInstance) -> Result<()> {
    for field in self.fields.get_fields() {
      // static field should be set not on an instance but through class_loader
      if !field.is_static() {
        let name = field.get_name();

        // TODO: use field type to check
        let field_type: &str = field.get_descriptor();

        let value = get_default_value(field_type);

        new_obj.new_field(name, value)?;
      }
    }

    for interface in self.interfaces.get_interfaces() {
      new_obj.add_interface(interface);
    }

    Ok(())
  }

  pub fn get_init(&self) -> bool {
    self.is_init
  }

  pub fn init(&mut self) {
    self.is_init = true;
  }

  pub fn get_interfaces(&self) -> &Vec<String> {
    self.interfaces.get_interfaces()
  }

  pub fn has_function(&self, method_name: &str, type_str: &str) -> bool {
    for i in 0..self.methods.len() {
      let method = self.methods.get(i);

      if method_name == method.get_name() && type_str == method.get_descriptor() {
        return true;
      }
    }

    // check parents
    false
  }

  pub fn get_fields(&self) -> &Vec<fields::FieldInfo> {
    self.fields.get_fields()
  }

  pub fn get_field_offset(&self, field_name: &str) -> Result<i64> {
    let fields = self.get_fields();

    for i in 0..fields.len() {
      if fields.get(i).unwrap().get_name() == field_name {
        return Ok(i as i64);
      }
    }

    Err(eyre!("field by offset not found"))
  }

  pub fn get_field_by_offset(&self, offset: i64) -> Result<FieldInfo> {
    let fields = self.get_fields();

    match fields.get(offset as usize) {
      Some(field) => Ok(field.clone()),
      None => Err(eyre!("field by offset not found")),
    }
  }

  pub fn get_static_field(&self, fieldname: &str) -> Result<&Type> {
    if fieldname == "jfrTracing" {
      return Ok(&types::Type::Boolean(false));
    }

    self
      .static_fields
      .get(fieldname)
      .ok_or_eyre(eyre!("static field not found: {:?}", fieldname))
  }

  pub fn _get_method_by_name_index(&self, index: ju2) -> Option<&methods::MethodInfo> {
    self.methods._get_by_name_index(index)
  }

  pub fn resolve_index(&self, index: ju2) -> Result<&constant_pool::CpInfo> {
    self.constant_pool.resolve_index(index)
  }

  pub fn resolve_name(&self, index: ju2) -> Result<String> {
    self.constant_pool.resolve_name(index)
  }

  pub fn resolve_class_name(&self, index: ju2) -> Result<String> {
    let class_index = self.constant_pool.resolve_class(index)?;

    self.constant_pool.resolve_name(class_index)
  }

  pub fn resolve_field_ref(&self, index: ju2) -> Result<(String, String, String)> {
    self.constant_pool.resolve_field_ref(index)
  }

  pub fn resolve_method_ref(&self, index: ju2) -> Result<(String, String, String)> {
    self.constant_pool.resolve_method_ref(index)
  }

  pub fn resolve_method_handle(&self, index: ju2) -> Result<MethodHandleResolved> {
    let (ref_kind, class_name, method_name, method_type) =
      self.constant_pool.resolve_method_handle(index)?;

    Ok(MethodHandleResolved {
      ref_kind,
      class_name,
      method_name,
      method_type,
    })
  }

  pub fn resolve_method_type(&self, index: ju2) -> Result<String> {
    self.constant_pool.resolve_method_type(index)
  }

  pub fn resolve_invokedynamic(&self, index: ju2) -> Result<InvokeDynamicResolved> {
    let (bootstrap_index, method_name, method_type) =
      self.constant_pool.resolve_invokedynamic(index)?;

    let bootstrap_method = self.get_bootstrap_method(bootstrap_index.into());

    let method_handle_cp_ref = bootstrap_method.get_method_ref();

    let method_handle_resolved = self.resolve_method_handle(method_handle_cp_ref)?;

    let bootstrap_arguments_ref = bootstrap_method.get_arguments().clone();

    Ok(InvokeDynamicResolved {
      bootstrap: method_handle_resolved,
      bootstrap_arguments: bootstrap_arguments_ref,
      method_name,
      method_type,
    })
  }

  pub fn put_static_field(&mut self, name: &str, value: Type) -> Result<()> {
    // TODO: check that it is static !
    self.static_fields.insert(name.to_string(), value);

    Ok(())
  }

  pub fn new_field(&mut self, name: &str, descriptor: &str) -> Result<()> {
    // TODO: set proper access flags
    let field = fields::FieldInfo::new(0, name, descriptor);

    self.fields.put_field(field);

    Ok(())
  }

  pub fn get_methods(&self) -> &Vec<methods::MethodInfo> {
    self.methods.get_methods()
  }

  pub fn get_name(&self) -> &str {
    &self.this_class_name
  }

  pub fn get_parent_name(&self) -> &str {
    &self.super_class_name
  }

  pub fn has_parent(&self) -> bool {
    !self.super_class_name.is_empty()
  }

  pub fn get_bootstrap_methods(&self) -> &attributes::BootstrapMethods {
    match self.attributes.get_by_name("BootstrapMethods").get_info() {
      attributes::AttributeInfoInfoEnum::BootstrapMethods(b) => b,
      _ => panic!(),
    }
  }

  pub fn get_bootstrap_method(&self, index: usize) -> &attributes::BootstrapMethod {
    self.get_bootstrap_methods().get(index)
  }

  pub fn is_interface(&self) -> bool {
    self.access_flags & 0x0200 != 0
  }

  pub fn _is_enum(&self) -> bool {
    self.access_flags & 0x04000 != 0
  }

  pub fn get_enclosing_method(&self) -> Option<&attributes::EnclosingMethod> {
    self.attributes.get_enclosing_method()
  }

  pub fn get_inner_classes(&self) -> Option<&attributes::InnerClasses> {
    self.attributes.get_inner_classes()
  }

  pub fn get_nest_host(&self) -> Option<&attributes::NestHost> {
    self.attributes.get_nest_host()
  }
}
