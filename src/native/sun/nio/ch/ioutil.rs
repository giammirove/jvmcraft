use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};
use log::warn;
use std::io::Error;

impl JVM {
  pub(crate) fn native_dispatcher_sun_nio_ch_ioutil(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("initIDs", "()V") => self.exec_native_sun_nio_ch_ioutil_init_ids(),
      ("setfdVal", "(Ljava/io/FileDescriptor;I)V") => self.exec_native_sun_nio_ch_ioutil_setfdval(),
      ("iovMax", "()I") => self.exec_native_sun_nio_ch_ioutil_iovmax(),
      ("writevMax", "()J") => self.exec_native_sun_nio_ch_ioutil_writevmax(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "sun/nio/ch/IOUtil".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_sun_nio_ch_ioutil_init_ids(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/IOUtil.initIDs not supported");
    Ok(None)
  }

  fn exec_native_sun_nio_ch_ioutil_setfdval(&mut self) -> Result<Option<types::Type>> {
    let fd_value = self.pop_stack()?.as_integer()?;
    let fd_ref = self.pop_stack()?.as_ref()?;
    let fd_obj = self.heap.get_obj_instance_mut(fd_ref)?;
    fd_obj.put_field("fd", types::Type::Integer(fd_value))?;
    Ok(None)
  }

  fn exec_native_sun_nio_ch_ioutil_iovmax(&mut self) -> Result<Option<types::Type>> {
    let iovmax = unsafe { libc::sysconf(libc::_SC_IOV_MAX) };
    if iovmax == -1 {
      return Err(Error::last_os_error().into());
    }
    self.push_stack(types::Type::Integer(iovmax as i32))?;
    Ok(None)
  }

  fn exec_native_sun_nio_ch_ioutil_writevmax(&mut self) -> Result<Option<types::Type>> {
    self.push_stack(types::Type::Integer(1024_i32))?;
    Ok(None)
  }
}
