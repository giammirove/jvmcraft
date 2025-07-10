use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};

impl JVM {
  pub(crate) fn native_dispatcher_sun_nio_ch_socketdispatcher(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      // polymorphic signature
      ("write0", "(Ljava/io/FileDescriptor;JI)I") => {
        self.exec_native_sun_nio_ch_socketdispatcher_write0()
      }
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "sun/nio/ch/SocketDispatcher".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_sun_nio_ch_socketdispatcher_write0(&mut self) -> Result<Option<types::Type>> {
    let length = self.pop_stack()?.as_integer()?;
    let address = self.pop_stack()?.as_long()? as u64;
    let fd_ref = self.pop_stack()?.as_ref()?; // FileDescriptor

    let fd_obj = self.heap.get_obj_instance(fd_ref)?;
    let fd = fd_obj.get_field("fd")?.as_integer()?;

    if !self.nativememory.is_valid(address) {
      return Err(eyre!(errors::InternalError::SegmentationFault(address)));
    }

    let bytes_written = unsafe {
      let ptr = address as *const libc::c_void;
      let res = libc::write(fd, ptr, length as usize);
      res as i32
    };

    let ret_value = types::Type::Integer(bytes_written);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }
}
