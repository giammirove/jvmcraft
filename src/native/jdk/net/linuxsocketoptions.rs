use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};
use log::warn;

impl JVM {
  pub(crate) fn native_dispatcher_jdk_net_linux(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("quickAckSupported0", "()Z") => self.exec_native_jdk_net_linux_quick_ack(),
      ("keepAliveOptionsSupported0", "()Z") => self.exec_native_jdk_net_linux_keep_alive(),
      ("incomingNapiIdSupported0", "()Z") => self.exec_native_jdk_net_linux_incoming_napi(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "jdk/net/LinuxSocketOptions".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  pub(crate) fn exec_native_jdk_net_linux_quick_ack(&mut self) -> Result<Option<types::Type>> {
    warn!("jdk/net/LinuxSocketOptions.quickAckSupported0 not supported");

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  pub(crate) fn exec_native_jdk_net_linux_keep_alive(&mut self) -> Result<Option<types::Type>> {
    warn!("jdk/net/LinuxSocketOptions.keepAliveOptionsSupported0 not supported");

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  pub(crate) fn exec_native_jdk_net_linux_incoming_napi(&mut self) -> Result<Option<types::Type>> {
    warn!("jdk/net/LinuxSocketOptions.incomingNapiIdSupported0 not supported");

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }
}
