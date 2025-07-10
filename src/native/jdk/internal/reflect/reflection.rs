use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};

impl JVM {
  pub(crate) fn native_dispatcher_jdk_internal_reflect_reflection(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("getCallerClass", "()Ljava/lang/Class;") => self.exec_native_reflection_get_caller_class(),
      ("getClassAccessFlags", "(Ljava/lang/Class;)I") => self.exec_native_get_class_access_flags(),
      ("areNestMates", "(Ljava/lang/Class;Ljava/lang/Class;)Z") => {
        self.exec_native_are_nest_mates()
      }
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "jdk/internal/reflect/Reflection".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_reflection_get_caller_class(&mut self) -> Result<Option<types::Type>> {
    // Skip 2 frames: [0] getCallerClass, [1] Unsafe.getUnsafe
    let ret_value = if let Some(caller_frame) = self.frames.get(self.frames.len().wrapping_sub(2)) {
      let class_name = caller_frame.get_classname().to_owned();

      let class_obj_ref = self
        .heap
        .get_class_instance(&mut self.class_loader, &class_name)?
        .get_ref();

      types::Type::ObjectRef(class_obj_ref)
    } else {
      panic!("get caller class")
    };

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_get_class_access_flags(&mut self) -> Result<Option<types::Type>> {
    let class_obj_ref = self.pop_object_ref()?;

    let obj = self.heap.get_instance(class_obj_ref)?;

    let class_name = obj.get_class_field_type();

    let access_flags = {
      let class_info = self.class_loader.get(class_name)?;

      class_info.get_access_flags() as i32
    };

    let ret_value = types::Type::Integer(access_flags);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_are_nest_mates(&mut self) -> Result<Option<types::Type>> {
    let class_a_ref = self.pop_ref()?; // Class<?>
    let class_b_ref = self.pop_ref()?; // Class<?>

    let class_a_name = self.heap.get_classname_from_class_obj(class_a_ref)?;
    let class_b_name = self.heap.get_classname_from_class_obj(class_b_ref)?;

    let nest_host_a = {
      let class_a = self.class_loader.get(&class_a_name)?;
      if let Some(nesthost) = class_a.get_nest_host() {
        nesthost.get_host_class_index() as i32
      } else {
        -1
      }
    };

    let nest_host_b = {
      let class_b = self.class_loader.get(&class_b_name)?;
      if let Some(nesthost) = class_b.get_nest_host() {
        nesthost.get_host_class_index() as i32
      } else {
        -1
      }
    };

    let ret_value = types::Type::Boolean(nest_host_a == nest_host_b);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }
}
