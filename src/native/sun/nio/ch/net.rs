use crate::runtime::{errors, jvm::*, types};
use color_eyre::eyre::{eyre, Result};
use log::{debug, warn};
use std::{io::Error, net::Ipv4Addr};

const IPV4: u16 = 1;

impl JVM {
  pub(crate) fn native_dispatcher_sun_nio_ch_net(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("initIDs", "()V") => self.exec_native_sun_nio_ch_net_init_ids(),
      ("pollinValue", "()S") => self.exec_native_sun_nio_ch_pollin_value(),
      ("polloutValue", "()S") => self.exec_native_sun_nio_ch_pollout_value(),
      ("pollerrValue", "()S") => self.exec_native_sun_nio_ch_pollerr_value(),
      ("pollhupValue", "()S") => self.exec_native_sun_nio_ch_pollhup_value(),
      ("pollnvalValue", "()S") => self.exec_native_sun_nio_ch_pollnval_value(),
      ("pollconnValue", "()S") => self.exec_native_sun_nio_ch_pollconn_value(),
      ("isExclusiveBindAvailable", "()I") => {
        self.exec_native_sun_nio_ch_is_exclusive_bind_available()
      }
      ("isIPv6Available0", "()Z") => self.exec_native_sun_nio_ch_is_ipv6_available(),
      ("isReusePortAvailable0", "()Z") => self.exec_native_sun_nio_ch_is_reuse_port_available(),
      ("socket0", "(ZZZZ)I") => self.exec_native_sun_nio_ch_socket0(),
      ("connect0", "(ZLjava/io/FileDescriptor;Ljava/net/InetAddress;I)I") => {
        self.exec_native_sun_nio_ch_connect0()
      }
      ("localInetAddress", "(Ljava/io/FileDescriptor;)Ljava/net/InetAddress;") => {
        self.exec_native_sun_nio_ch_localinetaddress()
      }
      ("localPort", "(Ljava/io/FileDescriptor;)I") => self.exec_native_sun_nio_ch_localport(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "sun/nio/ch/Net".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_sun_nio_ch_net_init_ids(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.initIDs not supported");

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_pollin_value(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.pollinValue fixed value");

    let ret_value = types::Type::Short(0x1);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_pollout_value(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.polloutValue fixed value");

    let ret_value = types::Type::Short(0x4);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_pollerr_value(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.pollerrValue fixed value");

    let ret_value = types::Type::Short(0x8);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_pollhup_value(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.pollhupValue fixed value");

    let ret_value = types::Type::Short(0x10);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_pollnval_value(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.pollnvalValue fixed value");

    let ret_value = types::Type::Short(0x20);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_pollconn_value(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.pollconnValue fixed value");

    // same as POLLOUT
    let ret_value = types::Type::Short(0x4);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_is_exclusive_bind_available(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.isExclusiveBindAvailable not supported");

    let ret_value = types::Type::Integer(-1);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_is_ipv6_available(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.isIPv6Available0 not supported");

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_sun_nio_ch_is_reuse_port_available(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.isReusePortAvailable0 not supported");

    let ret_value = types::Type::Boolean(false);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  //private static native int socket0(boolean preferIPv6, boolean stream, boolean reuse,
  //                              boolean fastLoopback);
  fn exec_native_sun_nio_ch_socket0(&mut self) -> Result<Option<types::Type>> {
    let _fast_loopback = self.pop_stack()?.as_bool()?;
    let reuse = self.pop_stack()?.as_bool()?;
    let is_stream = self.pop_stack()?.as_bool()?;
    let is_ipv6 = self.pop_stack()?.as_bool()?;

    // TODO: IPv6 not supported
    assert!(!is_ipv6);

    let domain = if is_ipv6 {
      libc::AF_INET6
    } else {
      libc::AF_INET
    };
    let sock_type = if is_stream {
      libc::SOCK_STREAM
    } else {
      libc::SOCK_DGRAM
    };

    // Create the socket
    let fd = unsafe { libc::socket(domain, sock_type, 0) };
    if fd < 0 {
      return Err(Error::last_os_error().into());
    }

    // Set SO_REUSEADDR if requested
    if reuse {
      let yes: i32 = 1;
      let ret = unsafe {
        libc::setsockopt(
          fd,
          libc::SOL_SOCKET,
          libc::SO_REUSEADDR,
          &yes as *const _ as *const _,
          std::mem::size_of_val(&yes) as u32,
        )
      };
      if ret < 0 {
        return Err(Error::last_os_error().into());
      }
    }

    let ret_value = types::Type::Integer(fd);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  //private static native int connect0(boolean preferIPv6,
  //                               FileDescriptor fd,
  //                               InetAddress remote,
  //                               int remotePort)
  fn exec_native_sun_nio_ch_connect0(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/Net.connect0 is in blocking mode");
    let remote_port = self.pop_stack()?.as_integer()?;
    let inetaddr_ref = self.pop_stack()?.as_ref()?; // InetAddress
    let fd_ref = self.pop_stack()?.as_ref()?; // FileDescriptor
    let _is_ipv6 = self.pop_stack()?.as_bool()?;

    assert!(!_is_ipv6);

    let fd_obj = self.heap.get_obj_instance(fd_ref)?;
    let fd = fd_obj.get_field("fd")?.as_integer()?;

    let inetaddr_obj = self.heap.get_obj_instance(inetaddr_ref)?;
    let holder_ref = inetaddr_obj.get_field("holder")?.as_ref()?;
    let holder = self.heap.get_obj_instance(holder_ref)?;
    debug!("{}", inetaddr_obj);
    debug!("{:?}", inetaddr_obj.get_parent());
    let addr = holder.get_field("address")?.as_integer()?;

    let ipv4_addr = Ipv4Addr::from(addr.to_be_bytes());

    let sockaddr = libc::sockaddr_in {
      sin_family: libc::AF_INET as u16,
      sin_port: libc::htons(remote_port as u16),
      sin_addr: libc::in_addr {
        s_addr: u32::from(ipv4_addr).to_be(),
      },
      sin_zero: [0; 8],
    };

    let conn_ret = unsafe {
      libc::connect(
        fd,
        &sockaddr as *const _ as *const libc::sockaddr,
        std::mem::size_of::<libc::sockaddr_in>() as u32,
      )
    };

    let ret_value = if conn_ret == 0 {
      types::Type::Integer(1) // no errors
    } else {
      let errno = unsafe { *libc::__errno_location() };
      if errno == libc::EINPROGRESS {
        types::Type::Integer(0)
      } else {
        return Err(eyre!(errors::JavaException::IO(format!(
          "Socket Connection failed: {}",
          errno
        ))));
      }
    };

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn fd_to_sockaddr(&self, fd: i32) -> Result<libc::sockaddr_in> {
    let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;

    let res =
      unsafe { libc::getsockname(fd, &mut addr as *mut _ as *mut libc::sockaddr, &mut len) };

    if res < 0 {
      return Err(eyre!(errors::JavaException::IO(format!(
        "GetSockName failed : {}",
        res
      ))));
    }

    Ok(addr)
  }

  fn fd_to_java_inetaddress(&mut self, fd: i32) -> Result<types::Type> {
    let addr = self.fd_to_sockaddr(fd)?;

    let inetaddr_ret = if addr.sin_family == libc::AF_INET as u16 {
      let ipv4_int = addr.sin_addr.s_addr.to_be();
      let ipv4_addr = Ipv4Addr::from(ipv4_int);
      let address_str = format!("{}", ipv4_addr);
      let holder_ref = self
        .heap
        .alloc_obj(
          &mut self.class_loader,
          "java/net/InetAddress$InetAddressHolder",
        )?
        .as_ref()?;
      let hostname_ref = self
        .heap
        .alloc_string(&mut self.class_loader, &address_str)?;

      let holder = self.heap.get_obj_instance_mut(holder_ref)?;
      debug!("{}", holder);
      holder.put_field("originalHostName", hostname_ref)?;
      holder.put_field("hostName", hostname_ref)?;
      holder.put_field("address", types::Type::Integer(ipv4_int as i32))?;
      holder.put_field("family", types::Type::Integer(IPV4 as i32))?;

      let inet4addr_ref = self
        .heap
        .alloc_obj(&mut self.class_loader, "java/net/Inet4Address")?;
      let inet4addr_obj = self.heap.get_obj_instance_mut(inet4addr_ref.as_ref()?)?;
      inet4addr_obj.put_field("holder", types::Type::ObjectRef(holder_ref))?;

      inet4addr_ref
    } else {
      panic!("family not available {}", addr.sin_family);
    };
    Ok(inetaddr_ret)
  }

  fn exec_native_sun_nio_ch_localinetaddress(&mut self) -> Result<Option<types::Type>> {
    let fd_ref = self.pop_stack()?.as_ref()?; // FileDescriptor

    let fd_obj = self.heap.get_obj_instance(fd_ref)?;
    let fd = fd_obj.get_field("fd")?.as_integer()?;

    let inetaddr_ref = self.fd_to_java_inetaddress(fd)?;

    self.push_stack(inetaddr_ref)?;
    Ok(Some(inetaddr_ref))
  }

  fn exec_native_sun_nio_ch_localport(&mut self) -> Result<Option<types::Type>> {
    let fd_ref = self.pop_stack()?.as_ref()?; // FileDescriptor

    let fd_obj = self.heap.get_obj_instance(fd_ref)?;
    let fd = fd_obj.get_field("fd")?.as_integer()?;

    let addr = self.fd_to_sockaddr(fd)?;
    let port = libc::ntohs(addr.sin_port);

    let ret_value = types::Type::Integer(port as i32);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }
}
