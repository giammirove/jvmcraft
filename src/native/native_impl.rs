use crate::{
  runtime::{errors, jvm::*, types},
  utils::{class_to_dotclass, descriptor_to_classname, get_argument_classnames, get_env, ju4},
};
use color_eyre::eyre::{eyre, Result};
use core::panic;
use log::{debug, warn};
use std::ffi::CString;
use std::{
  fs::File,
  io::Write,
  net::Ipv6Addr,
  os::{
    fd::RawFd,
    unix::{fs::MetadataExt, io::FromRawFd},
  },
  path::Path,
};

impl JVM {
  pub(crate) fn exec_native_platform_properties(&mut self) -> Result<()> {
    // Indexes of array elements written by native platformProperties()
    // The order is arbitrary (but alphabetic for convenience)
    let _display_country = 0;

    let _display_language = 1 + _display_country;

    let _display_script = 1 + _display_language;

    let _display_variant = 1 + _display_script;

    let _file_separator = 1 + _display_variant;

    let _format_country = 1 + _file_separator;

    let _format_language = 1 + _format_country;

    let _format_script = 1 + _format_language;

    let _format_variant = 1 + _format_script;

    let _ftp_non_proxy_hosts = 1 + _format_variant;

    let _ftp_proxy_host = 1 + _ftp_non_proxy_hosts;

    let _ftp_proxy_port = 1 + _ftp_proxy_host;

    let _http_non_proxy_hosts = 1 + _ftp_proxy_port;

    let _http_proxy_host = 1 + _http_non_proxy_hosts;

    let _http_proxy_port = 1 + _http_proxy_host;

    let _https_proxy_host = 1 + _http_proxy_port;

    let _https_proxy_port = 1 + _https_proxy_host;

    let _java_io_tmpdir = 1 + _https_proxy_port;

    let _line_separator = 1 + _java_io_tmpdir;

    let _native_encoding = 1 + _line_separator;

    let _os_arch = 1 + _native_encoding;

    let _os_name = 1 + _os_arch;

    let _os_version = 1 + _os_name;

    let _path_separator = 1 + _os_version;

    let _socks_non_proxy_hosts = 1 + _path_separator;

    let _socks_proxy_host = 1 + _socks_non_proxy_hosts;

    let _socks_proxy_port = 1 + _socks_proxy_host;

    let _stderr_encoding = 1 + _socks_proxy_port;

    let _stdout_encoding = 1 + _stderr_encoding;

    let _sun_arch_abi = 1 + _stdout_encoding;

    let _sun_arch_data_model = 1 + _sun_arch_abi;

    let _sun_cpu_endian = 1 + _sun_arch_data_model;

    let _sun_cpu_isalist = 1 + _sun_cpu_endian;

    let _sun_io_unicode_encoding = 1 + _sun_cpu_isalist;

    let _sun_jnu_encoding = 1 + _sun_io_unicode_encoding;

    let _sun_os_patch_level = 1 + _sun_jnu_encoding;

    let _user_dir = 1 + _sun_os_patch_level;

    let _user_home = 1 + _user_dir;

    let _user_name = 1 + _user_home;

    let fixed_length = 1 + _user_name;

    let array = self
      .heap
      .alloc_array("java/lang/String", vec![], fixed_length)?;

    self.push_stack(array)?;

    Ok(())
  }

  pub(crate) fn exec_native_vm_properties(&mut self) -> Result<()> {
    warn!("Assuming Linux when creating VM properties");
    // see  `java -XshowSettings:properties -version`

    // TODO: fixed some values now
    let java_home = get_env("JHOME", "");

    if java_home.is_empty() {
      return Err(eyre!(errors::InternalError::General(
        "JHOME (JAVA HOME) cannot be empty ... plz set it using JHOME=/path/to/home".to_string()
      )));
    }

    let user_home = get_env("JUHOME", "");

    let user_dir = get_env("JUDIR", "");

    let tmp_dir = get_env("JTMPDIR", "/tmp");

    let java_library = get_env(
      "JLIB",
      "/usr/java/packages/lib\n/usr/lib64\n/lib64\n/lib\n/usr/lib",
    );

    let sun_boot_library = format!("{}/lib", java_home);

    let kv = vec![
      ("java.vm.version", "23"),
      ("java.class.version", "0.67"),
      ("java.home", &java_home),
      ("user.home", &user_home),
      ("user.dir", &user_dir),
      ("java.library.path", &java_library),
      ("user.name", "USERNAME"),
      ("java.io.tmpdir", &tmp_dir),
      ("native.encoding", "UTF-8"),
      ("file.encoding", "UTF-8"),
      ("os.name", "OSNAME"),
      ("os.arch", "OSARCH"),
      ("os.version", "OSVERSION"),
      ("file.separator", "/"),
      ("path.separator", ":"),
      ("line.separator", "\n"),
      ("sun.boot.library.path", &sun_boot_library),
      ("sun.cpu.endian", "little"),
      ("sun.arch.data.model", "64"),
      ("sun.jnu.encoding", "UTF-8"),
      ("jdk.image.map.all", "true"),
    ];

    let mut strings = vec![];

    for (k, v) in kv {
      let key = self.heap.alloc_string(&mut self.class_loader, k)?;

      let value = self.heap.alloc_string(&mut self.class_loader, v)?;

      strings.push(key);

      strings.push(value);
    }

    let strings_len = strings.len();

    let array = self
      .heap
      .alloc_array("java/lang/String", strings, strings_len)?;

    self.push_stack(array)?;

    Ok(())
  }

  pub(crate) fn exec_native_hash_code(&mut self) -> Result<Option<types::Type>> {
    let class = self.pop_stack()?;

    match class {
      types::Type::ObjectRef(obj_ref) | types::Type::ArrayRef(obj_ref) => {
        let obj = self.heap.get_instance(obj_ref)?;

        let hash_code = obj.get_hash_code();

        self.push_stack(types::Type::Integer(hash_code as i32))?;
      }
      _ => return Err(eyre!(errors::InternalError::WrongType("Ref", class))),
    }

    Ok(None)
  }

  // public final native Class<?> getClass();
  pub(crate) fn exec_native_get_class(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_ref()?;

    let obj = self.heap.get_instance(obj_ref)?;

    let class_name = obj.get_class_field_type().to_owned();

    let class_ref = self
      .heap
      .get_class_instance(&mut self.class_loader, &class_name)?
      .get_ref();

    self.push_stack(types::Type::ObjectRef(class_ref))?;

    Ok(None)
  }

  // public static native void arraycopy(Object src, int srcPos, Object dest, int destPos, int
  // length)
  pub(crate) fn exec_native_arraycopy(&mut self) -> Result<Option<types::Type>> {
    let length = self.pop_stack()?;

    let destpos = self.pop_stack()?;

    let dest = self.pop_stack()?;

    let srcpos = self.pop_stack()?;

    let src = self.pop_stack()?;

    let src_ref = match src {
      types::Type::ArrayRef(r) => r,
      _ => return Err(eyre!("src is not an object ref {:?}", src)),
    };

    let srcpos = match srcpos {
      types::Type::Integer(r) => r,
      _ => return Err(eyre!("srcpos is not an integer {:?}", srcpos)),
    };

    let dest_ref = match dest {
      types::Type::ArrayRef(r) => r,
      _ => return Err(eyre!("dest is not an array ref {:?}", dest)),
    };

    let destpos = match destpos {
      types::Type::Integer(r) => r,
      _ => return Err(eyre!("destpos is not an integer {:?}", destpos)),
    };

    let length = match length {
      types::Type::Integer(r) => r,
      _ => return Err(eyre!("length is not an integer {:?}", length)),
    };

    // TODO: can I avoid clone here ?
    let src_array = self.heap.get_array_instance(src_ref)?.clone();

    let dest_array = self.heap.get_array_instance_mut(dest_ref)?;

    if src_array.len() < (srcpos + length) as usize
      || dest_array.len() < (destpos + length) as usize
    {
      return Err(eyre!("Index out of bounds in arraycopy"));
    }

    for i in 0..length {
      let value = src_array.get((srcpos + i) as usize)?;

      dest_array.set((destpos + i) as usize, *value)?;
    }

    Ok(None)
  }

  // public static native int floatToRawIntBits(float value);
  pub(crate) fn exec_native_float_to_raw_int_bits(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?;

    let value = match value {
      types::Type::Float(r) => r,
      _ => return Err(eyre!("length is not an float {:?}", value)),
    };

    let bits: i32 = f32::to_bits(value) as i32;

    self.push_stack(types::Type::Integer(bits))?;

    Ok(None)
  }

  // public static native int floatToIntBits(float value);
  pub(crate) fn exec_native_float_to_int_bits(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?;

    let value = match value {
      types::Type::Float(r) => r,
      _ => return Err(eyre!("length is not an float {:?}", value)),
    };

    let bits = if value.is_nan() {
      0x7fc00000 // Canonical NaN
    } else {
      value.to_bits()
    };

    self.push_stack(types::Type::Integer(bits as i32))?;

    Ok(None)
  }

  // public static native long doubleToRawLongBits(double value);
  pub(crate) fn exec_native_double_to_raw_long_bits(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?;

    let value = match value {
      types::Type::Double(r) => r,
      _ => return Err(eyre!("length is not an double{:?}", value)),
    };

    let bits: u64 = value.to_bits(); // equivalent to doubleToRawLongBits
    self.push_stack(types::Type::Long(bits as i64))?;

    Ok(None)
  }

  // public static native long doubleToLongBits(double value);
  pub(crate) fn exec_native_double_to_long_bits(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?;

    let value = match value {
      types::Type::Double(r) => r,
      _ => return Err(eyre!("length is not an double{:?}", value)),
    };

    let bits = if value.is_nan() {
      0x7ff8000000000000 // canonical NaN
    } else {
      value.to_bits()
    };

    self.push_stack(types::Type::Long(bits as i64))?;

    Ok(None)
  }

  // public synchronized native Throwable fillInStackTrace();
  pub(crate) fn exec_native_fill_in_stack_trace(&mut self) -> Result<Option<types::Type>> {
    let class = self.pop_stack()?;

    warn!("FILL IN STACK TRACE NOT IMPLEMENTED YET");

    self.push_stack(class)?;

    Ok(None)
  }

  // public static native void initialize();
  pub(crate) fn exec_native_initialize(&mut self) -> Result<Option<types::Type>> {
    warn!("Initialize NOT IMPLEMENTED YET");

    Ok(None)
  }

  pub(crate) fn exec_native_max_memory(&mut self) -> Result<Option<types::Type>> {
    self.push_stack(types::Type::Long(2000000))?;

    warn!("MAX MEMORY NOT IMPLEMENTED YET");

    Ok(None)
  }

  // public static native int getCDSConfigStatus();
  // 0 - disabled , 1 - enabled and supported, 2+ - reserved
  pub(crate) fn exec_native_get_cds_config_status(&mut self) -> Result<Option<types::Type>> {
    self.push_stack(types::Type::Integer(0))?;

    Ok(None)
  }

  // public static native void initializeFromArchive(Class<?> klass);
  pub(crate) fn exec_native_initialize_from_archive(&mut self) -> Result<Option<types::Type>> {
    warn!("INITIALIZE FROM ARCHIVE NOT IMPLEMENTED YET");

    Ok(None)
  }

  // public static native long getRandomSeedForDumping();
  pub(crate) fn exec_native_get_random_seed_for_dumping(&mut self) -> Result<Option<types::Type>> {
    let seed = rand::random::<u64>();

    self.push_stack(types::Type::Long(seed as i64))?;

    Ok(None)
  }

  // public native int availableProcessors();
  pub(crate) fn exec_native_available_processors(&mut self) -> Result<Option<types::Type>> {
    let cpus = std::thread::available_parallelism()
      .map(|n| n.get())
      .unwrap_or(1); // fallback if unknown
    self.push_stack(types::Type::Integer(cpus as i32))?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_next_thread_id_offset(&mut self) -> Result<Option<types::Type>> {
    // Simulate the offset of Thread.nextThreadID
    let offset = self
      .class_loader
      .get_field_offset("java/lang/Thread", "nextThreadID")?;

    self.push_stack(types::Type::Long(offset))?;

    Ok(None)
  }

  pub(crate) fn get_volatile_field(&mut self, offset: i64, obj_ref: ju4) -> Result<types::Type> {
    // if null the field is static => use Thread Class
    let field = if obj_ref == 0 {
      debug!("static field {}", offset);

      let field = self
        .class_loader
        .get_field_by_offset("java/lang/Thread", offset)?;
      let field_name = field.get_name();

      let class = self.class_loader.get("java/lang/Thread")?;

      let field_type = class.get_static_field(field_name)?;

      *field_type
    } else {
      debug!("Get Long volatile {}", obj_ref);

      let obj = self.heap.get_instance_mut(obj_ref)?;

      match obj {
        types::Instance::ObjectInstance(obj) => {
          let class_name = obj.get_classname();

          let field = self.class_loader.get_field_by_offset(class_name, offset)?;
          let field_name = field.get_name();

          obj.get_field(field_name)?
        }
        types::Instance::ArrayInstance(obj) => *obj.get_with_index_scale(offset as usize)?,
      }
    };

    Ok(field)
  }

  // private static native void setIn0(InputStream in);
  pub(crate) fn exec_native_set_in0(&mut self) -> Result<Option<types::Type>> {
    let input_stream_ref = self.pop_object_ref()?;

    let mut class = self.class_loader.get_mut("java/lang/System")?;

    class.put_static_field("in", types::Type::ObjectRef(input_stream_ref))?;

    Ok(None)
  }

  // private static native void setOut0(PrintStream out);
  pub(crate) fn exec_native_set_out0(&mut self) -> Result<Option<types::Type>> {
    let output_stream_ref = self.pop_object_ref()?;

    let mut class = self.class_loader.get_mut("java/lang/System")?;

    class.put_static_field("out", types::Type::ObjectRef(output_stream_ref))?;

    Ok(None)
  }

  // private static native void setErr0(PrintStream out);
  pub(crate) fn exec_native_set_err0(&mut self) -> Result<Option<types::Type>> {
    let err_ref = self.pop_object_ref()?;

    let mut class = self.class_loader.get_mut("java/lang/System")?;

    class.put_static_field("err", types::Type::ObjectRef(err_ref))?;

    Ok(None)
  }

  pub(crate) fn exec_native_clone(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_ref()?;

    let obj_class_name = self
      .heap
      .get_instance(obj_ref)?
      .get_class_field_type()
      .to_string();

    if !self
      .class_loader
      .has_interface("java/lang/Cloneable", &obj_class_name)?
    {
      return Err(eyre!(errors::JavaException::CloneNotSupported(
        obj_class_name
      )));
    }

    // Allocate new object and copy fields
    let cloned_ref = self.heap.clone_instance(obj_ref)?;

    self.push_stack(cloned_ref)?;

    Ok(None)
  }

  pub(crate) fn exec_native_find_signal0(&mut self) -> Result<Option<types::Type>> {
    let signal_str = self.pop_string()?;

    let signal_num = match signal_str.as_str() {
      "HUP" => 1,
      "INT" => 2,
      "QUIT" => 3,
      "ILL" => 4,
      "ABRT" => 6,
      "FPE" => 8,
      "KILL" => 9,
      "SEGV" => 11,
      "PIPE" => 13,
      "ALRM" => 14,
      "TERM" => 15,
      _ => -1,
    };

    self.push_stack(types::Type::Integer(signal_num))?;

    Ok(None)
  }

  pub(crate) fn exec_native_handle0(&mut self) -> Result<Option<types::Type>> {
    warn!("Handle0 not implemented yet");

    let _handler_addr = self.pop_loperand()?;

    let _sig = self.pop_ioperand()?;

    // return old signal handler's address -> 0 for now
    self.push_stack(types::Type::Long(0))?;

    Ok(None)
  }

  pub(crate) fn exec_native_current_thread(&mut self) -> Result<Option<types::Type>> {
    let thread_obj = self.get_current_thread_obj();

    self.push_stack(thread_obj)?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_current_carrier_thread(&mut self) -> Result<Option<types::Type>> {
    // TODO:
    warn!("Carrier Thread is Virtual Thread for now");

    let thread_obj = self.get_current_thread_obj();

    self.push_stack(thread_obj)?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_stack_access_control_context(
    &mut self,
  ) -> Result<Option<types::Type>> {
    warn!("Empty domain in get stack access control context");

    // Return NULL meaning everything is privileged
    self.push_stack(types::Type::Null)?;

    Ok(None)

    // Create an AccessControlContext object
    //let acc_ref = self
    //    .heap
    //    .alloc_obj(&mut self.classes, "java/security/AccessControlContext")?
    //    .as_ref()?;
    //
    //let array_ref = self
    //    .heap
    //    .alloc_array("java/security/ProtectionDomain", vec![], 0)?
    //    .as_ref()?;
    //
    //let acc_obj = self.heap.get_obj_instance_mut(acc_ref)?;
    //acc_obj.put_field("context", types::Type::ArrayRef(array_ref))?;
    //
    //self.push_stack(types::Type::ObjectRef(acc_ref))?;
    //Ok(None)
  }

  pub(crate) fn exec_native_set_priority0(&mut self) -> Result<Option<types::Type>> {
    let priority = self.pop_ioperand()?;

    let thread_obj = self.get_current_thread_obj().as_ref()?;

    let obj = self.heap.get_obj_instance_mut(thread_obj)?;

    match obj.put_field("priority", types::Type::Integer(priority)) {
      Ok(_) => {}
      _ => {
        obj.new_field("priority", types::Type::Integer(priority))?;
      }
    }

    Ok(None)
  }

  pub(crate) fn exec_native_start0(&mut self) -> Result<Option<types::Type>> {
    warn!("Multithreading not enabled for now");

    let thread_ref = self.get_current_thread_obj().as_ref()?;

    let thread_obj = self.heap.get_obj_instance(thread_ref)?;

    let class_name = thread_obj.get_classname();

    // Get the actual 'run' method
    let (run_class, _) = self
      .class_loader
      .get_method_by_name(class_name, "run", "()V")?;

    self.push_frame_from_class(
      &run_class,
      "run",
      "()V",
      vec![types::Type::ObjectRef(thread_ref)],
    )?;

    Ok(None)
  }

  pub(crate) fn exec_native_is_finalization_enabled(&mut self) -> Result<Option<types::Type>> {
    warn!("Finalization not enabled");

    self.push_stack(types::Type::Boolean(false))?;

    Ok(None)
  }

  pub(crate) fn exec_native_object_notify_all(&mut self) -> Result<Option<types::Type>> {
    let _obj_ref = self.pop_ref()?;

    warn!("Notify all not implemented");

    Ok(None)
  }

  pub(crate) fn exec_native_is_array(&mut self) -> Result<Option<types::Type>> {
    let class_ref = self.pop_object_ref()?;

    let class_obj = self.heap.get_obj_instance(class_ref)?;

    let class_name_field = class_obj.get_field("name")?.as_ref()?;

    let class_name_str = self.heap.get_string(class_name_field)?;

    self.push_stack(types::Type::Boolean(class_name_str.starts_with("[")))?;

    Ok(None)
  }

  pub(crate) fn exec_set_boot_loader_unnamed_module0(&mut self) -> Result<Option<types::Type>> {
    let module_ref = self.pop_ref()?;

    self.set_boot_loader_unnamed_module(module_ref);

    Ok(None)
  }

  pub(crate) fn exec_native_map_library_name(&mut self) -> Result<Option<types::Type>> {
    let libname_ref = self.pop_object_ref()?;

    let libname_str = self.heap.get_string(libname_ref)?;

    let mapped = if cfg!(target_os = "windows") {
      format!("{}.dll", libname_str)
    } else if cfg!(target_os = "macos") {
      format!("lib{}.dylib", libname_str)
    } else {
      format!("lib{}.so", libname_str)
    };

    let result_ref = self.heap.alloc_string(&mut self.class_loader, &mapped)?;

    self.push_stack(result_ref)?;

    Ok(None)
  }

  pub(crate) fn exec_native_find_builtin_lib(&mut self) -> Result<Option<types::Type>> {
    warn!("Fixing my path here for JDK-23");

    let libstr = self.pop_string()?;

    let lib_path = format!("{}{}", get_env("JHOME", ""), libstr);

    let path = Path::new(&lib_path);

    if path.exists() {
      let result_ref = self.heap.alloc_string(&mut self.class_loader, &lib_path)?;

      self.push_stack(result_ref)?;
    } else {
      // fall back to system wise libs
      self.push_stack(types::Type::Null)?;
    }

    Ok(None)
  }

  pub(crate) fn exec_native_get_boolean_attributes0(&mut self) -> Result<Option<types::Type>> {
    let file_obj = self.pop_object_ref()?;

    let file_instance = self.heap.get_obj_instance(file_obj)?;

    let path_field = file_instance.get_field("path")?;

    let path_ref = path_field.as_ref()?;

    let path_str = self.heap.get_string(path_ref)?;

    let path = Path::new(&path_str);

    let mut result = 0;

    if path.exists() {
      result |= 0x01;
    } // BA_EXISTS
    if path.is_file() {
      result |= 0x02;
    } // BA_REGULAR
    if path.is_dir() {
      result |= 0x04;
    } // BA_DIRECTORY
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
      if name.starts_with('.') {
        result |= 0x08; // BA_HIDDEN
      }
    }

    self.push_stack(types::Type::Integer(result))?;

    Ok(None)
  }

  pub(crate) fn exec_native_ensure_materialized_for_stack_walk(
    &mut self,
  ) -> Result<Option<types::Type>> {
    warn!("Every object is materialized");

    Ok(None)
  }

  pub(crate) fn exec_unix_native_dispatcher_init(&mut self) -> Result<Option<types::Type>> {
    warn!("Dispatcher Init is not fully implemented");

    self.push_stack(types::Type::Integer(0))?;

    Ok(None)
  }

  pub(crate) fn exec_native_getcwd(&mut self) -> Result<Option<types::Type>> {
    let cwd = std::env::current_dir()?.to_string_lossy().into_owned();

    let bytes: Vec<u8> = cwd.into_bytes();

    let array_ref = self.heap.alloc_array(
      "[B",
      bytes.iter().map(|b| types::Type::Byte(*b as i8)).collect(),
      bytes.len(),
    )?;

    self.push_stack(array_ref)?;

    Ok(None)
  }

  pub(crate) fn exec_native_identity_hash_code(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_ref()?;

    let obj = self.heap.get_instance(obj_ref)?;

    self.push_stack(types::Type::Integer(obj.get_hash_code() as i32))?;

    Ok(None)
  }

  pub(crate) fn exec_native_stat0(&mut self) -> Result<Option<types::Type>> {
    let attr_ref = self.pop_object_ref()?;

    let path_ptr = self.pop_stack()?.as_long()? as u64;

    // 1. Read the null-terminated path string from off-heap memory
    let path = self.nativememory.read_string(path_ptr)?;

    // 2. Perform stat
    match std::fs::metadata(&path) {
      Ok(meta) => {
        let file_obj = self.heap.get_obj_instance_mut(attr_ref)?;

        // Example: populate fields (names may vary by JDK)
        file_obj.put_field("st_mode", types::Type::Integer(meta.mode() as i32))?;

        file_obj.put_field("st_ino", types::Type::Long(meta.ino() as i64))?;

        file_obj.put_field("st_dev", types::Type::Long(meta.dev() as i64))?;

        file_obj.put_field("st_rdev", types::Type::Long(meta.rdev() as i64))?;

        file_obj.put_field("st_nlink", types::Type::Integer(meta.nlink() as i32))?;

        file_obj.put_field("st_uid", types::Type::Integer(meta.uid() as i32))?;

        file_obj.put_field("st_gid", types::Type::Integer(meta.gid() as i32))?;

        file_obj.put_field("st_size", types::Type::Long(meta.size() as i64))?;

        file_obj.put_field("st_atime_sec", types::Type::Long(meta.atime()))?;

        file_obj.put_field("st_atime_nsec", types::Type::Long(meta.atime_nsec()))?;

        file_obj.put_field("st_mtime_sec", types::Type::Long(meta.mtime()))?;

        file_obj.put_field("st_mtime_nsec", types::Type::Long(meta.mtime_nsec()))?;

        file_obj.put_field("st_ctime_sec", types::Type::Long(meta.ctime()))?;

        file_obj.put_field("st_ctime_nsec", types::Type::Long(meta.ctime_nsec()))?;

        self.push_stack(types::Type::Integer(0))?;
      }
      Err(_) => {
        // TODO: Map error to errno if desired
        self.push_stack(types::Type::Integer(-1))?;
      }
    };

    Ok(None)
  }

  pub(crate) fn exec_native_library_load(&mut self) -> Result<Option<types::Type>> {
    warn!("always positive to library load");

    let _throw = self.pop_ioperand()?;

    let _is_built_in = self.pop_ioperand()?;

    let _string = self.pop_ref()?;

    let _obj = self.pop_ref()?;

    self.push_stack(types::Type::Boolean(true))?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_native_map(&mut self) -> Result<Option<types::Type>> {
    let path_ref = self.pop_object_ref()?;

    let path_str = self.heap.get_string(path_ref)?;

    warn!("Could not map native image: {}", path_str);

    self.push_stack(types::Type::Null)?;

    //// Simulate reading the file (e.g., lib/modules)
    //match std::fs::read(&path_str) {
    //    Ok(data) => {
    //        let capacity = data.len();
    //        let buffer_ref = self.heap.alloc_direct_bytebuffer(&mut self.classes, data)?;
    //        self.push_stack(buffer_ref)?;
    //    }
    //    Err(e) => {
    //        warn!("Could not map native image: {}", e);
    //        self.push_stack(types::Type::Null)?;
    //    }
    //}
    Ok(None)
  }

  pub(crate) fn exec_native_open0(&mut self) -> Result<Option<types::Type>> {
    let mode = self.pop_ioperand()? as libc::mode_t;

    let flags = self.pop_ioperand()? as libc::c_int;

    let path_addr = self.pop_loperand()? as u64;

    let c_path = self.nativememory.read_string(path_addr)?;

    let path = Path::new(&c_path);

    if !path.exists() {
      panic!("file not found {:?}", path);
    }

    let cstr = CString::new(c_path.clone())?;

    let fd = unsafe { libc::open(cstr.as_ptr(), flags, mode) };

    self.push_stack(types::Type::Integer(fd))?;

    Ok(None)
  }

  pub(crate) fn exec_native_for_name0(&mut self) -> Result<Option<types::Type>> {
    let _caller_class = self.pop_stack()?;

    let _loader = self.pop_stack()?;

    let initialize = self.pop_ioperand()? != 0;

    let name_ref = self.pop_object_ref()?;

    let name = self.heap.get_string(name_ref)?;

    // Convert from "java.lang.String" to "java/lang/String"
    let internal_name = name.replace('.', "/");

    // force load class
    self.class_loader.load_class(&internal_name)?;

    // Initialize if requested
    if initialize {
      self.init_class(&internal_name)?;
    }

    // Get Class object reference
    let class_obj_ref = self
      .heap
      .get_class_instance(&mut self.class_loader, &internal_name)?
      .get_ref();

    self.push_stack(types::Type::ObjectRef(class_obj_ref))?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_declared_methods0(&mut self) -> Result<Option<types::Type>> {
    let public_only = self.pop_ioperand()? != 0;

    // Get the `Class` object (this)
    let class_ref = self.pop_object_ref()?;

    let class_instance = self.heap.get_obj_instance(class_ref)?;

    // Get the internal name of the class from the Class object
    let class_name = self
      .heap
      .get_string(class_instance.get_field("name")?.as_ref()?)?;

    let class_info = self.class_loader.get(&class_name)?;

    let methods = class_info.get_methods().clone();

    drop(class_info);

    // Create array of Method objects
    let mut method_objs = Vec::with_capacity(methods.len());

    for method in methods {
      if !public_only || method.is_public() {
        let method_obj =
          self
            .heap
            .alloc_reflect_method(&mut self.class_loader, &class_name, &method)?;

        method_objs.push(method_obj);
      }
    }

    let array_ref = self.heap.alloc_array(
      "Ljava/lang/reflect/Method;",
      method_objs.clone(),
      method_objs.len(),
    )?;

    self.push_stack(array_ref)?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_constant_pool(&mut self) -> Result<Option<types::Type>> {
    warn!("exec_native_get_constant_pool not fully implemeted");

    let class_obj_ref = self.pop_object_ref()?;

    let obj = self.heap.get_instance(class_obj_ref)?;

    let class_name = obj.get_class_field_type().to_owned();

    let class_obj_ref = self
      .heap
      .get_class_instance(&mut self.class_loader, &class_name)?
      .get_ref();

    // Simulate creating a new ConstantPool object
    let cp_obj = self
      .heap
      .alloc_obj(&mut self.class_loader, "jdk/internal/reflect/ConstantPool")?
      .as_ref()?;

    let cp_instance = self.heap.get_obj_instance_mut(cp_obj)?;

    // Link the real constant pool to this wrapper via a synthetic field
    cp_instance.new_field("internalClass", types::Type::ObjectRef(class_obj_ref))?;

    // Optionally store a raw constant pool or reference ID
    // e.g., cp_instance.new_field("index", Type::Integer(123))?;
    self.push_stack(types::Type::ObjectRef(cp_obj))?;

    Ok(None)
  }

  pub(crate) fn exec_native_invoke0(&mut self) -> Result<Option<types::Type>> {
    let args_array = self.pop_ref()?;

    let target_obj = self.pop_stack()?;

    let method_obj = self.pop_object_ref()?;

    let method_inst = self.heap.get_obj_instance(method_obj)?;

    let method_name = self
      .heap
      .get_string(method_inst.get_field("name")?.as_ref()?)?;

    let method_type = self
      .heap
      .get_string(method_inst.get_field("signature")?.as_ref()?)?;

    let class_ref = method_inst.get_field("clazz")?.as_ref()?;

    let class_inst = self.heap.get_obj_instance(class_ref)?; // java/lang/Class
    let class_inner_ref = class_inst.get_field("name")?.as_ref()?; // T in Class<T> as string
    let class_name = self.heap.get_string(class_inner_ref)?; // string
    let (method_class, _) =
      self
        .class_loader
        .get_method_by_name(&class_name, &method_name, &method_type)?;

    let array_args = self.heap.get_array_instance(args_array)?;

    let mut args = vec![];

    if target_obj != types::Type::Null {
      args.push(target_obj);
    }

    if let Some(_arg) = array_args.get_elements().iter().next() {
      // TODO: handle arguments
      panic!()
    }

    // Step 3: Push method frame and invoke
    let ret = self.call_and_resolve_method(&method_class, &method_name, &method_type, args)?;

    self.push_stack(ret)?;

    Ok(None)
  }

  pub(crate) fn exec_native_init_ids(&mut self) -> Result<Option<types::Type>> {
    warn!("IOUtil.initIDs: not fully implemented");

    let mut class = self.class_loader.get_mut("java/io/FileDescriptor")?;

    class.put_static_field("fd", types::Type::Integer(1))?;

    Ok(None)
  }

  pub(crate) fn exec_native_fileoutputstream_initids(&mut self) -> Result<Option<types::Type>> {
    warn!("FileOutputStream.initIDs: not fully implemented");

    let mut class = self.class_loader.get_mut("java/io/FileDescriptor")?;

    class.put_static_field("fd", types::Type::Integer(1))?;

    Ok(None)
  }

  pub(crate) fn exec_native_iov_max(&mut self) -> Result<Option<types::Type>> {
    let max_iov = 1024; // should be cross-platform
    self.push_stack(types::Type::Integer(max_iov))?;

    Ok(None)
  }

  pub(crate) fn exec_native_writev_max(&mut self) -> Result<Option<types::Type>> {
    let max_bytes: i64 = 2_147_483_647; // 2GB - 1, typical Linux max
    self.push_stack(types::Type::Long(max_bytes))?;

    Ok(None)
  }

  thread_local! {
      static NATIVE_THREAD_ID: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
  }

  pub(crate) fn exec_native_thread_init(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/NativeThread.init(): not fully implemented");

    Self::NATIVE_THREAD_ID.with(|id| {
      if id.get() == 0 {
        let new_id = self.alloc_new_native_tid();

        id.set(new_id);
      }
    });

    Ok(None)
  }

  pub(crate) fn exec_native_thread_current0(&mut self) -> Result<Option<types::Type>> {
    warn!("sun/nio/ch/NativeThread.current0(): not fully implemented");

    let tid = Self::NATIVE_THREAD_ID.with(|id| {
      if id.get() == 0 {
        let new_id = self.alloc_new_native_tid();

        id.set(new_id);
      }

      id.get()
    });

    self.push_stack(types::Type::Long(tid))?;

    Ok(None)
  }

  pub(crate) fn exec_native_pread0(&mut self) -> Result<Option<types::Type>> {
    let position = self.pop_stack()?.as_long()? as libc::off_t;

    let len = self.pop_stack()?.as_integer()? as usize;

    let address = self.pop_stack()?.as_long()? as u64;

    let fd_obj = self.pop_stack()?.as_ref()?;

    // Extract file descriptor from java/io/FileDescriptor.fd
    let fd_inst = self.heap.get_obj_instance(fd_obj)?;

    let raw_fd = fd_inst.get_field("fd")?.as_integer()? as RawFd;

    // Allocate temporary buffer for read
    let mut buffer = vec![0u8; len];

    let ret = unsafe {
      libc::pread(
        raw_fd,
        buffer.as_mut_ptr() as *mut libc::c_void,
        len,
        position,
      )
    };

    if ret < 0 {
      self.push_stack(types::Type::Integer(-1))?;

      return Ok(None);
    }

    let read_len = ret as usize;

    let dest: *mut u8 = address as *mut u8;

    self.nativememory.is_valid(address);

    for (i, val) in buffer.iter().enumerate().take(read_len) {
      let addr = dest.wrapping_add(i);

      unsafe {
        *addr = *val;
      }
    }

    self.push_stack(types::Type::Integer(read_len as i32))?;

    Ok(None)
  }

  pub(crate) fn exec_native_size0(&mut self) -> Result<Option<types::Type>> {
    let fd_obj = self.pop_stack()?.as_ref()?;

    let fd_inst = self.heap.get_obj_instance(fd_obj)?;

    let fd = fd_inst.get_field("fd")?.as_integer()? as RawFd;

    let mut stat_buf: libc::stat = unsafe { std::mem::zeroed() };

    let result = unsafe { libc::fstat(fd, &mut stat_buf) };

    if result != 0 {
      self.push_stack(types::Type::Long(-1))?;
    } else {
      self.push_stack(types::Type::Long(stat_buf.st_size))?;
    }

    Ok(None)
  }

  pub(crate) fn exec_native_allocation_granularity0(&mut self) -> Result<Option<types::Type>> {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) };

    let granularity = if page_size > 0 { page_size } else { 4096 };

    self.push_stack(types::Type::Long(granularity as i64))?;

    Ok(None)
  }

  pub(crate) fn exec_native_map0(&mut self) -> Result<Option<types::Type>> {
    let is_sync = self.pop_ioperand()? != 0;

    let size = self.pop_stack()?.as_long()? as usize;

    let position = self.pop_stack()?.as_long()? as libc::off_t;

    let prot_flags = self.pop_stack()?.as_integer()?; // 1=read, 2=write, 3=rw
    let fd_obj = self.pop_stack()?.as_ref()?;

    let fd_inst = self.heap.get_obj_instance(fd_obj)?;

    let raw_fd = fd_inst.get_field("fd")?.as_integer()? as RawFd;

    // TODO: why sometimes we dont have it?
    let mut prot = libc::PROT_READ;

    if prot_flags & 1 != 0 {
      prot |= libc::PROT_READ;
    }

    if prot_flags & 2 != 0 {
      prot |= libc::PROT_WRITE;
    }

    let flags = if is_sync {
      libc::MAP_SHARED
    } else {
      libc::MAP_PRIVATE
    };

    let addr = unsafe { libc::mmap(std::ptr::null_mut(), size, prot, flags, raw_fd, position) };

    if addr == libc::MAP_FAILED {
      return Err(eyre!("mmap failed"));
    }

    debug!(
      "MAP {} {} -> {:?} -> {:?}",
      raw_fd, size, addr as i64, prot_flags
    );

    self.nativememory.register(addr as u64, size as u64);

    // Return pointer address as long
    self.push_stack(types::Type::Long(addr as i64))?;

    Ok(None)
  }

  pub(crate) fn exec_native_class_is_interface(&mut self) -> Result<Option<types::Type>> {
    let class_obj_ref = self.pop_stack()?.as_ref()?;

    let class_name = self.heap.get_classname_from_class_obj(class_obj_ref)?;

    let is_interface = self.class_loader.get(&class_name)?.is_interface();

    self.push_stack(types::Type::Boolean(is_interface))?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_declared_constructors0(&mut self) -> Result<Option<types::Type>> {
    let public_only = self.pop_ioperand()? != 0;

    let class_ref = self.pop_stack()?.as_ref()?;

    // Resolve the class object
    let class_obj = self.heap.get_obj_instance(class_ref)?;

    let class_inner_ref = class_obj.get_field("name")?.as_ref()?; // T in Class<T> as string
    let class_name = self.heap.get_string(class_inner_ref)?; // string

    let mut constructors = vec![];

    // triple of (name, descriptor, access_flags)
    let mut method_names = vec![];

    let methods = self.class_loader.get(&class_name)?.get_methods().clone();

    for method in methods {
      if method.get_name() == "<init>" {
        if public_only && (!method.is_public()) {
          continue;
        }

        method_names.push((
          class_to_dotclass(method.get_name()),
          method.get_descriptor().to_string(),
          method.get_access_flags(),
        ));
      }
    }

    for method in method_names {
      let param_types = get_argument_classnames(&method.1);

      let mut param_types_args = vec![];

      for p in param_types {
        let p = &descriptor_to_classname(&p);

        let class = self.heap.get_class_instance(&mut self.class_loader, p)?;

        param_types_args.push(types::Type::ObjectRef(class.get_ref()));
      }

      let param_types_args_len = param_types_args.len();

      let param_array =
        self
          .heap
          .alloc_array("java/lang/Class", param_types_args, param_types_args_len)?;

      // Allocate java/lang/reflect/Constructor object
      let name_str = self.heap.alloc_string(&mut self.class_loader, &method.0)?;

      let sig_str = self.heap.alloc_string(&mut self.class_loader, &method.1)?;

      let ctor_ref = self
        .heap
        .alloc_obj(&mut self.class_loader, "java/lang/reflect/Constructor")?
        .as_ref()?;

      let ctor_inst = self.heap.get_obj_instance_mut(ctor_ref)?;

      ctor_inst.new_field("clazz", types::Type::ObjectRef(class_ref))?;

      ctor_inst.new_field("name", name_str)?;

      ctor_inst.new_field("signature", sig_str)?;

      ctor_inst.new_field("parameterTypes", param_array)?;

      ctor_inst.new_field("modifiers", types::Type::Integer(method.2 as i32))?;

      constructors.push(types::Type::ObjectRef(ctor_ref));
    }

    let constructors_len = constructors.len();

    let array = self.heap.alloc_array(
      "java/lang/reflect/Constructor",
      constructors,
      constructors_len,
    )?;

    self.push_stack(array)?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_modifiers(&mut self) -> Result<Option<types::Type>> {
    let class_ref = self.pop_stack()?.as_ref()?;

    let class_obj = self.heap.get_obj_instance(class_ref)?;

    let class_inner_ref = class_obj.get_field("name")?.as_ref()?; // T in Class<T> as string
    let class_name = self.heap.get_string(class_inner_ref)?; // string
    let access_flags = self.class_loader.get(&class_name)?.get_access_flags();

    self.push_stack(types::Type::Integer(access_flags as i32))?;

    Ok(None)
  }

  pub(crate) fn exec_native_new_instance0(&mut self) -> Result<Option<types::Type>> {
    let args_array_ref = self.pop_array_ref()?;

    let ctor_obj_ref = self.pop_object_ref()?;

    let ctor_obj = self.heap.get_obj_instance(ctor_obj_ref)?;

    let clazz_ref = ctor_obj.get_field("clazz")?.as_ref()?;

    let clazz_obj = self.heap.get_obj_instance(clazz_ref)?;

    let class_inner_ref = clazz_obj.get_field("name")?.as_ref()?; // T in Class<T> as string
    let clazz_name = self.heap.get_string(class_inner_ref)?;

    let method_name = self
      .heap
      .get_string(ctor_obj.get_field("name")?.as_ref()?)?; // should be "<init>"
    assert!(method_name == "<init>");

    let descriptor = self
      .heap
      .get_string(ctor_obj.get_field("signature")?.as_ref()?)?
      .to_string();

    let new_obj = self.heap.alloc_obj(&mut self.class_loader, &clazz_name)?;

    // Prepare arguments
    let mut args = vec![new_obj];

    let array = self.heap.get_array_instance(args_array_ref)?;

    for arg in array.get_elements() {
      match arg {
        types::Type::Null => args.push(types::Type::Null),
        _ => {
          let obj = self.heap.get_obj_instance(arg.as_ref()?)?;

          if obj.is_primitive() {
            let value = obj.get_field("value")?;

            if value.get_category() == 2 {
              args.push(value);
            }

            args.push(value);
          } else {
            args.push(types::Type::ObjectRef(obj.get_ref()));
          }
        }
      }
    }

    self.call_and_resolve_method(&clazz_name, &method_name, &descriptor, args)?;

    // Result of <init> is always void, but we return the new instance
    self.push_stack(new_obj)?;

    Ok(None)
  }

  pub(crate) fn exec_native_canonicalize0(&mut self) -> Result<Option<types::Type>> {
    let path_obj = self.pop_object_ref()?;

    let path = self.heap.get_string(path_obj)?;

    //Canonicalize the path
    let canonical = match std::fs::canonicalize(&path) {
      Ok(p) => p,
      Err(_) => std::path::PathBuf::from(path),
    };

    let canonical_str = canonical.to_string_lossy().to_string();

    let canonical_obj = self
      .heap
      .alloc_string(&mut self.class_loader, &canonical_str)?;

    self.push_stack(canonical_obj)?;

    Ok(None)
  }

  pub(crate) fn exec_native_is_assignable_from(&mut self) -> Result<Option<types::Type>> {
    let other_class_obj = self.pop_object_ref()?;

    let this_class_obj = self.pop_object_ref()?;

    let this_class = self.heap.get_obj_instance(this_class_obj)?;

    let other_class = self.heap.get_obj_instance(other_class_obj)?;

    let result = types::Type::check_type(
      &mut self.class_loader,
      this_class.get_classname(),
      other_class.get_classname(),
    )?;

    self.push_stack(types::Type::Boolean(result))?;

    Ok(None)
  }

  pub(crate) fn exec_native_get_superclass(&mut self) -> Result<Option<types::Type>> {
    let class_ref = self.pop_stack()?.as_ref()?;

    let class_inner = self
      .heap
      .get_class_from_class_obj(&mut self.class_loader, class_ref)?;

    if class_inner.get_name() == "java/lang/Object" {
      drop(class_inner);

      self.push_stack(types::Type::Null)?;
    } else {
      let super_class_name = class_inner.get_parent_name().to_owned();
      drop(class_inner);

      let super_class_obj_ref = self
        .heap
        .get_class_instance(&mut self.class_loader, &super_class_name)?
        .get_ref();

      self.push_stack(types::Type::ObjectRef(super_class_obj_ref))?;
    }

    Ok(None)
  }

  pub(crate) fn exec_native_define_module0(&mut self) -> Result<Option<types::Type>> {
    let packages_arr = self.pop_ref()?; // [Ljava/lang/Object;
    let version_obj = self.pop_ref()?; // version String (can be null)
    let name_obj = self.pop_object_ref()?; // name String
    let is_open = self.pop_ioperand()?; // boolean 1/0
    let module_obj = self.pop_object_ref()?; // this Module object
    let module_name = self.heap.get_string(name_obj)?;

    let version = if version_obj != 0 {
      Some(self.heap.get_string(version_obj)?)
    } else {
      None
    };

    let mut packages = Vec::new();

    if packages_arr != 0 {
      let array = self.heap.get_array_instance(packages_arr)?;

      for elem in array.get_elements() {
        let elem_ref = elem.as_ref()?;

        if elem_ref != 0 {
          let pkg_name = self.heap.get_string(elem_ref)?;

          packages.push(pkg_name);
        }
      }
    }

    self.class_loader.modulemanager.add(
      module_obj,
      &module_name.clone(),
      is_open != 0,
      version,
      module_name,
      packages,
    );

    Ok(None)
  }

  pub(crate) fn exec_native_long_bits_to_double(&mut self) -> Result<Option<types::Type>> {
    let bits = self.pop_stack()?.as_long()?;

    let value = f64::from_bits(bits as u64);

    self.push_stack(types::Type::Double(value))?;

    Ok(None)
  }

  pub(crate) fn exec_native_add_reads0(&mut self) -> Result<Option<types::Type>> {
    let target_module_ref = self.pop_ref()?; // other
    let source_module_ref = self.pop_ref()?; // 'this' module
    if source_module_ref == 0 || target_module_ref == 0 {
      // Should not crash on null
      return Ok(None);
    }

    let source_module = self.class_loader.modulemanager.get_mut(source_module_ref)?;

    source_module.add_read(target_module_ref);

    Ok(None)
  }

  pub(crate) fn exec_native_add_exports_to_all0(&mut self) -> Result<Option<types::Type>> {
    let package_name_ref = self.pop_object_ref()?; // String
    let module_ref = self.pop_object_ref()?; // Module
    let package_name = self.heap.get_string(package_name_ref)?;

    let module = self.class_loader.modulemanager.get_mut(module_ref)?;

    module.add_export_all(package_name);

    Ok(None)
  }

  pub(crate) fn exec_native_add_exports0(&mut self) -> Result<Option<types::Type>> {
    let target_ref = self.pop_ref()?; // Module (can be null)
    let package_name_ref = self.pop_object_ref()?; // String
    let source_ref = self.pop_object_ref()?; // Module
    assert!(target_ref != 0);

    let package_name = self.heap.get_string(package_name_ref)?;

    let module = self.class_loader.modulemanager.get_mut(source_ref)?;

    module.add_export_to_module(target_ref, package_name);

    Ok(None)
  }

  pub(crate) fn exec_native_fileoutputstream_write_bytes(&mut self) -> Result<Option<types::Type>> {
    let _append = self.pop_ioperand()? != 0; // boolean Z
    let len = self.pop_ioperand()?; // int
    let off = self.pop_ioperand()?; // int
    let byte_array_ref = self.pop_array_ref()?; // [B
    let this_ref = self.pop_object_ref()?; // FileOutputStream (this)
    let byte_array = self.heap.get_array_instance(byte_array_ref)?;

    let this = self.heap.get_obj_instance(this_ref)?;

    let fd_ref = this.get_field("fd")?.as_ref()?;

    let fd_obj = self.heap.get_obj_instance(fd_ref)?;

    let fd = fd_obj.get_field("fd")?.as_integer()?;

    let mut all_bytes = vec![];

    for b in byte_array.get_elements() {
      all_bytes.push(b.as_byte()? as u8);
    }

    if off < 0 || len < 0 || (off as usize + len as usize) > all_bytes.len() {
      return Err(eyre!(errors::JavaException::ArrayIndexOutOfBounds(
        (off + len) as usize,
        all_bytes.len()
      )));
    }

    let slice = &all_bytes[off as usize..(off as usize + len as usize)];

    let mut file = unsafe { File::from_raw_fd(fd) };

    file.write_all(slice)?;

    // to avoid closing the fd when `file` goes out of scope
    std::mem::forget(file);

    Ok(None)
  }

  pub(crate) fn exec_native_inetaddress_init(&mut self) -> Result<Option<types::Type>> {
    warn!("java/net/InetAddress.init() not implemented");

    Ok(None)
  }

  pub(crate) fn exec_native_is_ipv6_supported(&mut self) -> Result<Option<types::Type>> {
    warn!("java/net/InetAddress.isIPv6Supported() not implemented");

    let _ipv6 = Ipv6Addr::LOCALHOST;

    let supported = true;

    self.push_stack(types::Type::Boolean(supported))?;

    Ok(None)
  }

  pub(crate) fn exec_native_is_ipv4_available(&mut self) -> Result<Option<types::Type>> {
    warn!("java/net/InetAddress.isIPv4Available() not implemented");

    let available = true;

    self.push_stack(types::Type::Boolean(available))?;

    Ok(None)
  }

  pub(crate) fn exec_native_inet4address_init(&mut self) -> Result<Option<types::Type>> {
    warn!("java/net/Inet4Address.init() not implemented");

    Ok(None)
  }

  pub fn exec_native_get_declared_fields0(&mut self) -> Result<Option<types::Type>> {
    let public_only = self.pop_stack()?.as_bool()?;
    let class_obj = self.pop_object_ref()?; // this (java/lang/Class)

    let class = self
      .heap
      .get_class_from_class_obj(&mut self.class_loader, class_obj)?;

    let classname = class.get_name().to_owned();

    debug!("{}", classname);

    let fields = class
      .get_fields()
      .iter()
      .filter(|f| !public_only || f.is_public())
      .cloned()
      .collect::<Vec<_>>()
      .clone();

    drop(class);

    let mut elements = vec![];

    for field in &fields {
      let field_obj = self
        .heap
        .alloc_reflect_field(&mut self.class_loader, &classname, field)?;

      elements.push(field_obj);
    }

    let array_ref = self
      .heap
      .alloc_array("java/lang/reflect/Field", elements, 0)?;

    self.push_stack(array_ref)?;

    Ok(None)
  }

  pub(crate) fn exec_native_array_newarray(&mut self) -> Result<Option<types::Type>> {
    let size = self.pop_ioperand()? as usize;

    let comp_type_ref = self.pop_object_ref()?; // Ljava/lang/Class;
                                                //let comp_type_classname = self
                                                //  .heap
                                                //  .get_obj_instance(comp_type_ref)?
                                                //  .get_classname()
                                                //  .to_owned();
    let comp_type_classname = self.heap.get_classname_from_class_obj(comp_type_ref)?;

    let array_ref = self.heap.alloc_array(&comp_type_classname, vec![], size)?;
    self.push_stack(array_ref)?;

    Ok(None)
  }

  pub fn exec_native_string_intern(&mut self) -> Result<Option<types::Type>> {
    let str_ref = self.pop_object_ref()?; // the string instance

    let value = self.heap.get_string(str_ref)?;
    // as of now every string is interned so this is "useless" but in the future
    // a selection of what to intern might be done
    // => therefore better play safe here
    let new_str_ref = self.heap.alloc_string(&mut self.class_loader, &value)?;

    self.push_stack(new_str_ref)?;

    Ok(None)
  }
}
