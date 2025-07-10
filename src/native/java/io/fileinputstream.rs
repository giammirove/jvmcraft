use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};
use log::warn;
use std::ffi::CString;

impl JVM {
  pub(crate) fn native_dispatcher_java_io_fileinputstream(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("initIDs", "()V") => self.exec_native_java_io_fileinputstream_init_ids(),
      ("open0", "(Ljava/lang/String;)V") => self.exec_native_java_io_fileinputstream_open0(),
      ("read0", "()I") => self.exec_native_java_io_fileinputstream_read0(),
      ("readBytes", "([BII)I") => self.exec_native_java_io_fileinputstream_read_bytes(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/io/FileInputStream".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_java_io_fileinputstream_init_ids(&mut self) -> Result<Option<types::Type>> {
    warn!("java/io/FileInputStream.initIDs not supported");

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_java_io_fileinputstream_open0(&mut self) -> Result<Option<types::Type>> {
    let filepath_ref = self.pop_stack()?.as_ref()?; // String
    let this_ref = self.pop_stack()?.as_ref()?; // java/io/FileInputStream

    let filepath = self.heap.get_string(filepath_ref)?;
    let this = self.heap.get_obj_instance_mut(this_ref)?;

    let c_path = CString::new(filepath.clone())?;

    let fd = unsafe { libc::open(c_path.as_ptr(), 666) };

    if fd < 0 {
      return Err(eyre!(errors::JavaException::FileNotFound(filepath)));
    }

    let fd_ref = this.get_field("fd")?.as_ref()?; // java/io/FileDescriptor
    let fd_obj = self.heap.get_obj_instance_mut(fd_ref)?;

    fd_obj.put_field("fd", types::Type::Integer(fd))?;

    Ok(None)
  }

  fn exec_native_java_io_fileinputstream_read_bytes(&mut self) -> Result<Option<types::Type>> {
    let read_len = self.pop_ioperand()? as usize;
    let offset = self.pop_ioperand()? as usize;
    let array_ref = self.pop_stack()?.as_ref()?; // [B
    let this_ref = self.pop_stack()?.as_ref()?; // java/io/FileInputStream

    let this = self.heap.get_obj_instance_mut(this_ref)?;
    let fd_ref = this.get_field("fd")?.as_ref()?;
    let fd_obj = self.heap.get_obj_instance_mut(fd_ref)?;
    let fd = fd_obj.get_field("fd")?.as_integer()?;

    let array = self.heap.get_array_instance_mut(array_ref)?;
    let array_len = array.get_elements().len();

    if offset + read_len > array_len {
      return Err(eyre!(errors::JavaException::ArrayIndexOutOfBounds(
        offset + read_len,
        array_len
      )));
    }

    let mut buffer = vec![0u8; array_len];

    let bytes_read = unsafe { libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, array_len) };

    match bytes_read.cmp(&0) {
      std::cmp::Ordering::Less => {
        return Err(eyre!(errors::JavaException::IO(format!(
          "Failed to to read from {}",
          fd
        ))));
      }
      std::cmp::Ordering::Equal => {
        let ret_value = types::Type::Integer(-1);
        self.push_stack(ret_value)?;
        return Ok(Some(ret_value)); // EOF
      }
      _ => {}
    };

    for (i, buf) in buffer.iter().enumerate().take(bytes_read as usize) {
      array.set(offset + i, types::Type::Byte(*buf as i8))?;
    }

    let ret_value = types::Type::Integer(bytes_read as i32);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_java_io_fileinputstream_read0(&mut self) -> Result<Option<types::Type>> {
    let this_ref = self.pop_stack()?.as_ref()?; // java/io/FileInputStream

    let this = self.heap.get_obj_instance_mut(this_ref)?;
    let fd_ref = this.get_field("fd")?.as_ref()?;
    let fd_obj = self.heap.get_obj_instance(fd_ref)?;
    let fd = fd_obj.get_field("fd")?.as_integer()?;

    if fd < 0 {
      return Err(eyre!(errors::JavaException::IO(
        "Failed to to read from inputstream".to_string()
      )));
    }

    let mut buffer = vec![0u8; 1];

    let bytes_read = unsafe { libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, 1) };

    match bytes_read.cmp(&0) {
      std::cmp::Ordering::Less => {
        return Err(eyre!(errors::JavaException::IO(format!(
          "Failed to to read from {}",
          fd
        ))));
      }
      std::cmp::Ordering::Equal => {
        let ret_value = types::Type::Integer(-1);
        self.push_stack(ret_value)?;
        return Ok(Some(ret_value)); // EOF
      }
      _ => {}
    };

    let ret_value = types::Type::Integer(buffer[0] as i32);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }
}
