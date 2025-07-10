use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};
use log::warn;

impl JVM {
  pub(crate) fn native_dispatcher_java_io_filedescriptor(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("initIDs", "()V") => self.exec_native_filedescriptor_init_ids(),
      ("getHandle", "(I)J") => self.exec_native_get_handle(),
      ("getAppend", "(I)Z") => self.exec_native_get_append(),
      ("close0", "()V") => self.exec_native_java_io_filedescriptor_close0(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/io/FileDescriptor".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  // private static native void initIDs();
  fn exec_native_filedescriptor_init_ids(&mut self) -> Result<Option<types::Type>> {
    // No-op stub (for now)
    // fd = 0 for in
    // fd = 1 for out
    // fd = 2 for err
    Ok(None)
  }

  // private static native long getHandle(int);
  fn exec_native_get_handle(&mut self) -> Result<Option<types::Type>> {
    let fd = self.pop_ioperand()?;

    // fd = 0 for in
    // fd = 1 for out
    // fd = 2 for err
    self.push_stack(types::Type::Long(fd as i64))?;

    Ok(None)
  }

  // private static native boolean getAppend(int);
  fn exec_native_get_append(&mut self) -> Result<Option<types::Type>> {
    warn!("Default append mode = false");

    let _fd = self.pop_ioperand()?;

    self.push_stack(types::Type::Boolean(false))?;

    Ok(None)
  }

  fn exec_native_java_io_filedescriptor_close0(&mut self) -> Result<Option<types::Type>> {
    let this_ref = self.pop_stack()?.as_ref()?; // java/io/FileInputStream

    let this = self.heap.get_obj_instance_mut(this_ref)?;
    this.put_field("fd", types::Type::Integer(-1))?;

    Ok(None)
  }
}
