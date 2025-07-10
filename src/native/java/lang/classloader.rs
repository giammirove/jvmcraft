use crate::{
  class_loader::class_file::ClassFile,
  runtime::{
    errors,
    jvm::*,
    types::{self},
  },
};
use color_eyre::eyre::{eyre, Result};
use log::warn;

impl JVM {
  pub(crate) fn native_dispatcher_java_lang_classloader(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("findBootstrapClass", "(Ljava/lang/String;)Ljava/lang/Class;") => {
        self.exec_native_find_bootstrap_class()
      }
      ("defineClass0", "(Ljava/lang/ClassLoader;Ljava/lang/Class;Ljava/lang/String;[BIILjava/security/ProtectionDomain;ZILjava/lang/Object;)Ljava/lang/Class;") => {
        self.exec_native_define_class0()
      }
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/lang/ClassLoader".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_find_bootstrap_class(&mut self) -> Result<Option<types::Type>> {
    warn!("java/lang/ClassLoader.findBootstrapClass returns Null (system-level class loader)");
    let str_ref = self.pop_object_ref()?; // the string instance
    let classname = self.heap.get_string(str_ref)?;
    self.init_class(&classname)?;

    let ret_value = types::Type::Null;
    self.push_stack(ret_value)?;

    Ok(Some(ret_value))
  }

  //static native Class<?> defineClass0(ClassLoader loader,
  //                                Class<?> lookup,
  //                                String name,
  //                                byte[] b, int off, int len,
  //                                ProtectionDomain pd,
  //                                boolean initialize,
  //                                int flags,
  //                                Object classData);
  //
  fn exec_native_define_class0(&mut self) -> Result<Option<types::Type>> {
    warn!("java/lang/ClassLoader.defineClass0 not fully implemented");

    let _class_data = self.pop_ref()?; // may be null
    let _flags = self.pop_ioperand()?;
    let initialize = self.pop_stack()?.as_bool()?;
    let _protection_domain = self.pop_ref()?; // may be null
    let len = self.pop_ioperand()? as usize;
    let off = self.pop_ioperand()? as usize;
    let _byte_array_ref = self.pop_array_ref()?;
    let name_ref = self.pop_ref()?;
    let _lookup = self.pop_ref()?; // context class, may be null
    let class_loader = self.pop_stack()?; // may be null

    let name = self.heap.get_string(name_ref)?;

    let mut data = vec![];
    let byte_array = self.heap.get_array_instance(_byte_array_ref)?;
    for el in byte_array.get_elements() {
      let b = el.as_byte()?;
      data.push(b as u8);
    }

    let new_class_file = ClassFile::parse_from_bytes(&data[off..(off + len)])?;

    self
      .class_loader
      .add_class_file("unnamed", &name, new_class_file)?;

    let new_class_obj = self.get_class_instance_mut(&name)?;
    new_class_obj.new_field("classLoader", class_loader)?;
    let new_class_ref = new_class_obj.get_ref();

    if initialize {
      self.init_class(&name)?;
    }

    let ret_value = types::Type::ObjectRef(new_class_ref);
    self.push_stack(ret_value)?;

    Ok(Some(ret_value))
  }
}
