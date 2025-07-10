use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};

impl JVM {
  pub(crate) fn native_dispatcher_java_lang_ref_reference(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("refersTo0", "(Ljava/lang/Object;)Z") => {
        self.exec_native_java_lang_ref_reference_refers_to0()
      }
      ("clear0", "()V") => self.exec_native_java_lang_ref_reference_clear0(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/lang/ref/Reference".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_java_lang_ref_reference_refers_to0(&mut self) -> Result<Option<types::Type>> {
    let target_ref = self.pop_object_ref()?;

    let reference_ref = self.pop_object_ref()?;

    let reference = self.heap.get_obj_instance(reference_ref)?;

    let referent = reference.get_field("referent");

    let result = match referent {
      Ok(types::Type::ObjectRef(referent_ref)) => referent_ref == target_ref,
      _ => false,
    };

    let ret_value = types::Type::Boolean(result);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_java_lang_ref_reference_clear0(&mut self) -> Result<Option<types::Type>> {
    let reference_ref = self.pop_object_ref()?;

    let reference = self.heap.get_obj_instance_mut(reference_ref)?;

    reference.put_field("referent", types::Type::Null)?;
    Ok(None)
  }
}
