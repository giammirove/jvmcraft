use crate::{
  runtime::{errors, jvm::*, types},
  utils::get_argument_classnames,
};
use color_eyre::eyre::{eyre, Result};
use log::warn;

impl JVM {
  pub(crate) fn native_dispatcher_java_lang_invoke_varhandle(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      // polymorphic signature
      ("getAndBitwiseOr", _) => self.exec_native_java_lang_invoke_varhandle_getandbitwiseor(),
      ("compareAndSet", _) => self.exec_native_java_lang_invoke_varhandle_compareandset(),
      ("set", _) => self.exec_native_java_lang_invoke_varhandle_set(type_str),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/lang/invoke/VarHandle".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_java_lang_invoke_varhandle_getandbitwiseor(
    &mut self,
  ) -> Result<Option<types::Type>> {
    // TODO: make atomic
    warn!("java/lang/invoke/VarHandle.getAndBitwiseOr is not atomic!");
    let value = self.pop_stack()?.as_integer()?;
    let obj_ref = self.pop_stack()?.as_ref()?;
    let this_ref = self.pop_stack()?.as_ref()?; // java/lang/invoke/VarHandle

    let this = self.heap.get_obj_instance(this_ref)?;

    // TODO: this works only if the field is not static

    let field_offset = this.get_field("fieldOffset")?.as_integer()?;

    let receiver_type_ref = this.get_field("receiverType")?.as_ref()?;
    let receiver_type = self.heap.get_classname_from_class_obj(receiver_type_ref)?;

    let obj = self.heap.get_obj_instance_mut(obj_ref)?;
    let class = self.class_loader.get(&receiver_type)?;
    let field = class.get_field_by_offset(field_offset as i64)?;
    let field_name = field.get_name();

    let original_field_value = obj.get_field(field_name)?.as_integer()?;

    obj.put_field(
      field_name,
      types::Type::Integer(original_field_value | value),
    )?;
    drop(class);

    let ret_value = types::Type::Integer(original_field_value);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_java_lang_invoke_varhandle_compareandset(
    &mut self,
  ) -> Result<Option<types::Type>> {
    // TODO: make atomic
    warn!("java/lang/invoke/VarHandle.compareAndSet is not atomic!");

    let new_value = self.pop_stack()?;
    let expected = self.pop_stack()?;
    let obj_ref = self.pop_stack()?.as_ref()?;
    let this_ref = self.pop_stack()?.as_ref()?; // java/lang/invoke/VarHandle

    let this = self.heap.get_obj_instance(this_ref)?;

    // TODO: this works only if the field is not static and with fields only
    // TODO: generalize to any kind

    let field_offset = this.get_field("fieldOffset")?.as_integer()?;

    let receiver_type_ref = this.get_field("receiverType")?.as_ref()?;
    let receiver_type = self.heap.get_classname_from_class_obj(receiver_type_ref)?;

    let obj = self.heap.get_obj_instance_mut(obj_ref)?;
    let class = self.class_loader.get(&receiver_type)?;
    let field = class.get_field_by_offset(field_offset as i64)?;
    let field_name = field.get_name();

    let original_field_value = obj.get_field(field_name)?;

    let successfull = if original_field_value == expected {
      obj.put_field(field_name, new_value)?;
      true
    } else {
      false
    };
    drop(class);

    let ret_value = types::Type::Boolean(successfull);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // java/lang/invoke/VarHandleByteArrayAsShorts$ArrayHandle set ([BIS)V
  fn exec_native_java_lang_invoke_varhandle_set_bytearrayasshorts(
    &mut self,
    args: &[types::Type],
  ) -> Result<Option<types::Type>> {
    assert!(args.len() == 3);
    let value = args[0].as_short()?;
    let index = args[1].as_integer()? as usize;
    let array_ref = args[2].as_ref()?; // [B
    let array = self.heap.get_array_instance_mut(array_ref)?;

    // TODO: check which is the correct byte order
    let bytes = value.to_be_bytes();

    array.set(index, types::Type::Byte(bytes[0] as i8))?;
    array.set(index + 1, types::Type::Byte(bytes[1] as i8))?;

    Ok(None)
  }

  fn exec_native_java_lang_invoke_varhandle_set(
    &mut self,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    let args_str = get_argument_classnames(type_str);

    let mut args = vec![];
    for _ in 0..args_str.len() {
      args.push(self.pop_stack()?);
    }

    let this_ref = self.pop_stack()?.as_ref()?; // java/lang/invoke/VarHandle

    let this = self.heap.get_obj_instance(this_ref)?;

    match this.get_classname() {
      "java/lang/invoke/VarHandleByteArrayAsShorts$ArrayHandle" => {
        self.exec_native_java_lang_invoke_varhandle_set_bytearrayasshorts(&args)?
      }
      // TODO: add the other cases
      _ => panic!("not handled {}", this.get_classname()),
    };

    Ok(None)
  }
}
