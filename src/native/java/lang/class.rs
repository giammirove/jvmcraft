use color_eyre::eyre::{eyre, Result};
use core::panic;
use log::warn;

use crate::{
  notimpl,
  runtime::{
    errors,
    jvm::*,
    types::{self, Type},
  },
  utils::class_to_dotclass,
};

impl JVM {
  pub(crate) fn native_dispatcher_java_lang_class(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("getPrimitiveClass", "(Ljava/lang/String;)Ljava/lang/Class;") => {
        self.exec_native_get_primitive_class()
      }
      ("initClassName", "()Ljava/lang/String;") => self.exec_native_init_class_name(),
      ("isPrimitive", "()Z") => self.exec_native_is_primitive(),
      ("isInstance", "(Ljava/lang/Object;)Z") => self.exec_native_is_instance(),
      ("desiredAssertionStatus0", "(Ljava/lang/Class;)Z") => {
        let ret_value = types::Type::Boolean(true);
        self.push_stack(ret_value)?;
        Ok(Some(ret_value))
      }
      ("isArray", "()Z") => self.exec_native_is_array(),
      (
        "forName0",
        "(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;",
      ) => self.exec_native_for_name0(),
      ("getDeclaredMethods0", "(Z)[Ljava/lang/reflect/Method;") => {
        self.exec_native_get_declared_methods0()
      }
      ("getConstantPool", "()Ljdk/internal/reflect/ConstantPool;") => {
        self.exec_native_get_constant_pool()
      }
      ("isInterface", "()Z") => self.exec_native_class_is_interface(),
      ("getDeclaredConstructors0", "(Z)[Ljava/lang/reflect/Constructor;") => {
        self.exec_native_get_declared_constructors0()
      }
      ("getModifiers", "()I") => self.exec_native_get_modifiers(),
      ("isAssignableFrom", "(Ljava/lang/Class;)Z") => self.exec_native_is_assignable_from(),
      ("getSuperclass", "()Ljava/lang/Class;") => self.exec_native_get_superclass(),
      ("getDeclaredFields0", "(Z)[Ljava/lang/reflect/Field;") => {
        self.exec_native_get_declared_fields0()
      }
      ("isHidden", "()Z") => self.exec_native_is_hidden(),
      ("getEnclosingMethod0", "()[Ljava/lang/Object;") => self.exec_native_get_enclosing_method0(),
      ("getDeclaringClass0", "()Ljava/lang/Class;") => self.exec_native_get_declaring_class0(),
      ("getNestHost0", "()Ljava/lang/Class;") => self.exec_native_get_nest_host0(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/lang/Class".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  pub(crate) fn exec_native_get_primitive_class(&mut self) -> Result<Option<types::Type>> {
    // Class.getPrimitiveClass("int") -> so push `Class` ref  then the "int"
    let str_obj_ref = self.pop_stack()?;

    match str_obj_ref {
      types::Type::ObjectRef(str_ref) => {
        let class_name: &str = &self.heap.get_string(str_ref)?;

        let class_name = match class_name {
          "float" => "F",
          "int" => "I",
          "double" => "D",
          "short" => "S",
          "char" => "C",
          "byte" => "B",
          "boolean" => "Z",
          "long" => "J",
          "void" => "V",
          _ => notimpl!(class_name),
        };

        let class_ref = self
          .heap
          .get_class_instance(&mut self.class_loader, class_name)?
          .get_ref();

        let ret_value = types::Type::ObjectRef(class_ref);
        self.push_stack(ret_value)?;

        Ok(Some(ret_value))
      }
      _ => notimpl!(),
    }
  }

  // private native String initClassName();
  pub(crate) fn exec_native_init_class_name(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_object_ref()?;

    let class_name = {
      let obj = self.heap.get_obj_instance(obj_ref)?;

      class_to_dotclass(obj.get_classname()).to_string()
    };

    let string = self
      .heap
      .alloc_string(&mut self.class_loader, &class_name)?;

    self.push_stack(string)?;

    let obj = self.heap.get_obj_instance_mut(obj_ref)?;

    obj.new_field("name", string)?;
    Ok(Some(string))
  }

  /// public native boolean isPrimitive();
  pub(crate) fn exec_native_is_primitive(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_object_ref()?;

    let obj = self.heap.get_obj_instance(obj_ref)?;

    let class_name = obj.get_classname();

    if class_name == "java/lang/Class" {
      let instanceof = obj.get_field("name")?;

      let name = self.heap.get_string(instanceof.as_ref()?)?;

      let ret_value = types::Type::Boolean(types::Type::is_primitive(&name));
      self.push_stack(ret_value)?;
      return Ok(Some(ret_value));
    }

    Err(eyre!(errors::InternalError::General(
      "is primitive not handled".to_string()
    )))
  }

  /// public native boolean isHidden();
  pub(crate) fn exec_native_is_hidden(&mut self) -> Result<Option<types::Type>> {
    warn!("Hidden class not supported");
    let _ = self.pop_object_ref()?;

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_get_enclosing_method0(&mut self) -> Result<Option<types::Type>> {
    let class_obj = self.pop_object_ref()?;
    let class = self
      .heap
      .get_class_from_class_obj(&mut self.class_loader, class_obj)?;

    let ret_value = if let Some(enclosing_info) = class.get_enclosing_method() {
      let cp = enclosing_info.clone();
      drop(class);

      if !cp.define_in_method() {
        let ret_value = types::Type::Null;
        self.push_stack(ret_value)?;
        return Ok(Some(ret_value));
      }

      let declaring_class_obj = self
        .heap
        .alloc_class_obj(&mut self.class_loader, cp.get_classname())?;
      let name_str = self
        .heap
        .alloc_string(&mut self.class_loader, cp.get_method_name())?;
      let desc_str = self
        .heap
        .alloc_string(&mut self.class_loader, cp.get_method_descriptor())?;

      self.heap.alloc_array(
        "java/lang/Object",
        vec![declaring_class_obj, name_str, desc_str],
        0,
      )?
    } else {
      drop(class);
      types::Type::Null
    };

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_get_declaring_class0(&mut self) -> Result<Option<types::Type>> {
    let class_obj = self.pop_object_ref()?;
    let class = self
      .heap
      .get_class_from_class_obj(&mut self.class_loader, class_obj)?;

    let ret_value = if let Some(inner_classes) = class.get_inner_classes() {
      'stop: loop {
        if let Some(entry) = inner_classes.get_inner_classes().iter().next() {
          if entry.get_name() == class.get_name() {
            let entry_name = entry.get_name().to_owned();
            drop(class);
            let entry_class = self
              .heap
              .get_class_instance(&mut self.class_loader, &entry_name)?
              .get_ref();

            break 'stop types::Type::ObjectRef(entry_class);
          } else {
            drop(class);
            break 'stop types::Type::Null;
          }
        }
      }
    } else {
      drop(class);
      types::Type::Null
    };

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  pub(crate) fn exec_native_is_instance(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_object_ref()?; // other object
    let this_ref = self.pop_object_ref()?; // this Class<T>

    let obj = self.heap.get_obj_instance(obj_ref)?;

    let this_inner_class = self.heap.get_classname_from_class_obj(this_ref)?;

    // this = obj is valid ?
    let is_instance = Type::check_type(
      &mut self.class_loader,
      &this_inner_class,
      obj.get_classname(),
    )?;
    let ret_value = types::Type::Boolean(is_instance);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_get_nest_host0(&mut self) -> Result<Option<types::Type>> {
    let class_obj = self.pop_object_ref()?;
    let class = self
      .heap
      .get_class_from_class_obj(&mut self.class_loader, class_obj)?;

    let ret_value = if let Some(nest_host) = class.get_nest_host() {
      let host_class_str = class.resolve_class_name(nest_host.get_host_class_index())?;
      drop(class);
      let host_class_ref = self
        .heap
        .get_class_instance(&mut self.class_loader, &host_class_str)?
        .get_ref();
      types::Type::ObjectRef(host_class_ref)
    } else {
      drop(class);
      types::Type::Null
    };

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }
}
