use color_eyre::eyre::{eyre, Result};
use log::warn;

use crate::runtime::{errors, jvm::*, types};

impl JVM {
  /// Call a native function
  ///
  /// # Arguments
  ///
  /// * `class_name` - Class of the function to call
  /// * `name` - Name of the function to call
  /// * `type_str` - Type of the function to call
  pub(crate) fn call_native(
    &mut self,
    class_name: &str,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    // TODO: create hierarchical dispatch
    match (class_name, name, type_str) {
      (_, "registerNatives", "()V") => {
        warn!("TODO: {} {} {}", class_name, name, type_str);
      }
      _ if class_name == "java/lang/Class" => {
        return self.native_dispatcher_java_lang_class(name, type_str);
      }
      _ if class_name == "java/lang/ClassLoader" => {
        return self.native_dispatcher_java_lang_classloader(name, type_str);
      }
      _ if class_name == "java/lang/invoke/MethodHandle" => {
        return self.native_dispatcher_java_lang_invoke_methodhandle(name, type_str);
      }
      _ if class_name == "java/lang/invoke/MethodHandleNatives" => {
        return self.native_dispatcher_java_lang_invoke_methodhandlenatives(name, type_str);
      }
      _ if class_name == "java/lang/invoke/VarHandle" => {
        return self.native_dispatcher_java_lang_invoke_varhandle(name, type_str);
      }
      _ if class_name == "jdk/internal/misc/Unsafe" => {
        return self.native_dispatcher_jdk_internal_misc_unsafe(name, type_str);
      }
      _ if class_name == "jdk/internal/reflect/Reflection" => {
        return self.native_dispatcher_jdk_internal_reflect_reflection(name, type_str);
      }
      _ if class_name == "jdk/net/LinuxSocketOptions" => {
        return self.native_dispatcher_jdk_net_linux(name, type_str);
      }
      _ if class_name == "sun/nio/ch/Net" => {
        return self.native_dispatcher_sun_nio_ch_net(name, type_str);
      }
      _ if class_name == "sun/nio/ch/SocketDispatcher" => {
        return self.native_dispatcher_sun_nio_ch_socketdispatcher(name, type_str);
      }
      _ if class_name == "sun/nio/ch/IOUtil" => {
        return self.native_dispatcher_sun_nio_ch_ioutil(name, type_str);
      }
      _ if class_name == "java/io/FileInputStream" => {
        return self.native_dispatcher_java_io_fileinputstream(name, type_str);
      }
      _ if class_name == "java/io/FileDescriptor" => {
        return self.native_dispatcher_java_io_filedescriptor(name, type_str);
      }
      _ if class_name == "java/lang/ref/Reference" => {
        return self.native_dispatcher_java_lang_ref_reference(name, type_str);
      }
      (_, "platformProperties", _) => {
        self.exec_native_platform_properties()?;
      }
      ("jdk/internal/util/SystemProps$Raw", "vmProperties", "()[Ljava/lang/String;") => {
        self.exec_native_vm_properties()?;
      }
      ("java/lang/Object", "hashCode", "()I") => {
        self.exec_native_hash_code()?;
      }
      (_, "getClass", _) => {
        self.exec_native_get_class()?;
      }
      ("java/lang/System", "arraycopy", "(Ljava/lang/Object;ILjava/lang/Object;II)V") => {
        self.exec_native_arraycopy()?;
      }
      (_, "floatToRawIntBits", _) => {
        self.exec_native_float_to_raw_int_bits()?;
      }
      (_, "floatToIntBits", _) => {
        self.exec_native_float_to_int_bits()?;
      }
      (_, "doubleToRawLongBits", _) => {
        self.exec_native_double_to_raw_long_bits()?;
      }
      (_, "doubleToLongBits", _) => {
        self.exec_native_double_to_long_bits()?;
      }
      (_, "fillInStackTrace", _) => {
        self.exec_native_fill_in_stack_trace()?;
      }
      (_, "initialize", _) => {
        self.exec_native_initialize()?;
      }
      ("java/lang/Runtime", "maxMemory", "()J") => {
        self.exec_native_max_memory()?;
      }
      ("jdk/internal/misc/CDS", "getCDSConfigStatus", "()I") => {
        self.exec_native_get_cds_config_status()?;
      }
      ("jdk/internal/misc/CDS", "initializeFromArchive", "(Ljava/lang/Class;)V") => {
        self.exec_native_initialize_from_archive()?;
      }
      ("jdk/internal/misc/CDS", "getRandomSeedForDumping", "()J") => {
        self.exec_native_get_random_seed_for_dumping()?;
      }
      ("java/lang/Runtime", "availableProcessors", "()I") => {
        self.exec_native_available_processors()?;
      }
      ("java/lang/Thread", "getNextThreadIdOffset", "()J") => {
        self.exec_native_get_next_thread_id_offset()?;
      }
      ("java/lang/System", "setIn0", "(Ljava/io/InputStream;)V") => {
        self.exec_native_set_in0()?;
      }
      ("java/lang/System", "setOut0", "(Ljava/io/PrintStream;)V") => {
        self.exec_native_set_out0()?;
      }
      ("java/lang/System", "setErr0", "(Ljava/io/PrintStream;)V") => {
        self.exec_native_set_err0()?;
      }
      ("java/lang/Object", "clone", "()Ljava/lang/Object;") => {
        self.exec_native_clone()?;
      }
      ("jdk/internal/misc/Signal", "findSignal0", "(Ljava/lang/String;)I") => {
        self.exec_native_find_signal0()?;
      }
      ("jdk/internal/misc/Signal", "handle0", "(IJ)J") => {
        self.exec_native_handle0()?;
      }
      ("java/lang/Thread", "currentThread", "()Ljava/lang/Thread;") => {
        self.exec_native_current_thread()?;
      }
      (
        "java/security/AccessController",
        "getStackAccessControlContext",
        "()Ljava/security/AccessControlContext;",
      ) => {
        self.exec_native_get_stack_access_control_context()?;
      }
      ("java/lang/Thread", "currentCarrierThread", "()Ljava/lang/Thread;") => {
        self.exec_native_get_current_carrier_thread()?;
      }
      ("java/lang/Thread", "setPriority0", "(I)V") => {
        self.exec_native_set_priority0()?;
      }
      ("java/lang/Thread", "start0", "()V") => {
        self.exec_native_start0()?;
      }
      ("java/lang/ref/Finalizer", "isFinalizationEnabled", "()Z") => {
        self.exec_native_is_finalization_enabled()?;
      }
      ("java/lang/Object", "notifyAll", "()V") => {
        self.exec_native_object_notify_all()?;
      }
      (
        "jdk/internal/loader/BootLoader",
        "setBootLoaderUnnamedModule0",
        "(Ljava/lang/Module;)V",
      ) => {
        self.exec_set_boot_loader_unnamed_module0()?;
      }
      ("java/lang/System", "mapLibraryName", "(Ljava/lang/String;)Ljava/lang/String;") => {
        self.exec_native_map_library_name()?;
      }
      (
        "jdk/internal/loader/NativeLibraries",
        "findBuiltinLib",
        "(Ljava/lang/String;)Ljava/lang/String;",
      ) => {
        self.exec_native_find_builtin_lib()?;
      }
      ("java/io/UnixFileSystem", "getBooleanAttributes0", "(Ljava/io/File;)I") => {
        self.exec_native_get_boolean_attributes0()?;
      }
      (
        "java/security/AccessController",
        "ensureMaterializedForStackWalk",
        "(Ljava/lang/Object;)V",
      ) => {
        self.exec_native_ensure_materialized_for_stack_walk()?;
      }
      ("sun/nio/fs/UnixNativeDispatcher", "init", "()I") => {
        self.exec_unix_native_dispatcher_init()?;
      }
      ("sun/nio/fs/UnixNativeDispatcher", "getcwd", "()[B") => {
        self.exec_native_getcwd()?;
      }
      ("java/lang/System", "identityHashCode", "(Ljava/lang/Object;)I") => {
        self.exec_native_identity_hash_code()?;
      }
      ("sun/nio/fs/UnixNativeDispatcher", "stat0", "(JLsun/nio/fs/UnixFileAttributes;)I") => {
        self.exec_native_stat0()?;
      }
      (
        "jdk/internal/loader/NativeLibraries",
        "load",
        "(Ljdk/internal/loader/NativeLibraries$NativeLibraryImpl;Ljava/lang/String;ZZ)Z",
      ) => {
        self.exec_native_library_load()?;
      }
      (
        "jdk/internal/jimage/NativeImageBuffer",
        "getNativeMap",
        "(Ljava/lang/String;)Ljava/nio/ByteBuffer;",
      ) => {
        self.exec_native_get_native_map()?;
      }
      ("sun/nio/fs/UnixNativeDispatcher", "open0", "(JII)I") => {
        self.exec_native_open0()?;
      }
      (
        "jdk/internal/reflect/DirectMethodHandleAccessor$NativeAccessor",
        "invoke0",
        "(Ljava/lang/reflect/Method;Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
      ) => {
        self.exec_native_invoke0()?;
      }
      ("sun/nio/ch/IOUtil", "initIDs", "()V") => {
        self.exec_native_init_ids()?;
      }
      ("sun/nio/ch/IOUtil", "iovMax", "()I") => {
        self.exec_native_iov_max()?;
      }
      ("sun/nio/ch/IOUtil", "writevMax", "()J") => {
        self.exec_native_writev_max()?;
      }
      ("sun/nio/ch/NativeThread", "init", "()V") => {
        self.exec_native_thread_init()?;
      }
      ("sun/nio/ch/NativeThread", "current0", "()J") => {
        self.exec_native_thread_current0()?;
      }
      ("sun/nio/ch/UnixFileDispatcherImpl", "pread0", "(Ljava/io/FileDescriptor;JIJ)I") => {
        self.exec_native_pread0()?;
      }
      ("sun/nio/ch/UnixFileDispatcherImpl", "size0", "(Ljava/io/FileDescriptor;)J") => {
        self.exec_native_size0()?;
      }
      ("sun/nio/ch/UnixFileDispatcherImpl", "allocationGranularity0", "()J") => {
        self.exec_native_allocation_granularity0()?;
      }
      ("sun/nio/ch/UnixFileDispatcherImpl", "map0", "(Ljava/io/FileDescriptor;IJJZ)J") => {
        self.exec_native_map0()?;
      }
      (
        "jdk/internal/reflect/DirectConstructorHandleAccessor$NativeAccessor",
        "newInstance0",
        "(Ljava/lang/reflect/Constructor;[Ljava/lang/Object;)Ljava/lang/Object;",
      ) => {
        self.exec_native_new_instance0()?;
      }
      ("java/io/UnixFileSystem", "canonicalize0", "(Ljava/lang/String;)Ljava/lang/String;") => {
        self.exec_native_canonicalize0()?;
      }
      (
        "java/lang/Module",
        "defineModule0",
        "(Ljava/lang/Module;ZLjava/lang/String;Ljava/lang/String;[Ljava/lang/Object;)V",
      ) => {
        self.exec_native_define_module0()?;
      }
      ("java/lang/Double", "longBitsToDouble", "(J)D") => {
        self.exec_native_long_bits_to_double()?;
      }
      ("java/lang/Module", "addReads0", "(Ljava/lang/Module;Ljava/lang/Module;)V") => {
        self.exec_native_add_reads0()?;
      }
      ("java/lang/Module", "addExportsToAll0", "(Ljava/lang/Module;Ljava/lang/String;)V") => {
        self.exec_native_add_exports_to_all0()?;
      }
      (
        "java/lang/Module",
        "addExports0",
        "(Ljava/lang/Module;Ljava/lang/String;Ljava/lang/Module;)V",
      ) => {
        self.exec_native_add_exports0()?;
      }
      ("java/io/FileOutputStream", "initIDs", "()V") => {
        self.exec_native_fileoutputstream_initids()?;
      }
      ("java/io/FileOutputStream", "writeBytes", "([BIIZ)V") => {
        self.exec_native_fileoutputstream_write_bytes()?;
      }
      ("java/net/InetAddress", "init", "()V") => {
        self.exec_native_inetaddress_init()?;
      }
      ("java/net/InetAddress", "isIPv6Supported", "()Z") => {
        self.exec_native_is_ipv6_supported()?;
      }
      ("java/net/InetAddress", "isIPv4Available", "()Z") => {
        self.exec_native_is_ipv4_available()?;
      }
      ("java/net/Inet4Address", "init", "()V") => {
        self.exec_native_inet4address_init()?;
      }
      ("java/lang/reflect/Array", "newArray", "(Ljava/lang/Class;I)Ljava/lang/Object;") => {
        self.exec_native_array_newarray()?;
      }
      ("java/lang/String", "intern", "()Ljava/lang/String;") => {
        self.exec_native_string_intern()?;
      }
      _ => {
        return Err(eyre!(errors::InternalError::NativeNotImplemented(
          format!("Unknown {}", class_name),
          name.to_owned(),
          type_str.to_owned()
        )));
      }
    }

    Ok(None)
  }
}
