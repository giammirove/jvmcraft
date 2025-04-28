use crate::runtime::errors;
use crate::runtime::jvm::*;
use crate::runtime::types;
use color_eyre::eyre::{eyre, Result};
use log::warn;
use log::error;

impl JVM {

    /// Call a native function
    ///
    /// # Arguments
    ///
    /// * `class_name` - Class of the function to call
    /// * `name` - Name of the function to call
    /// * `type_str` - Type of the function to call
    ///
    pub(crate) fn call_native(
        &mut self,
        class_name: &str,
        name: &str,
        type_str: &str,
    ) -> Result<Option<types::Type>> {
        warn!("NEED TO ACCOUNT FOR CLASS NAME AND METHOD TYPE");
        // TODO: SOLVE based on class and type
        match (class_name, name, type_str) {
            (_, "registerNatives", _) => {
                warn!("TODO: {}", name)
            }
            (_, "desiredAssertionStatus0", _) => self.push_stack(types::Type::Integer(1))?,
            (_, "platformProperties", _) => {
                self.exec_native_platform_properties()?;
                warn!("TODO: {}", name)
            }
            ("jdk/internal/util/SystemProps$Raw", "vmProperties", "()[Ljava/lang/String;") => {
                self.exec_native_vm_properties()?;
                warn!("TODO: {}", name)
            }
            ("java/lang/Class", "getPrimitiveClass", "(Ljava/lang/String;)Ljava/lang/Class;") => {
                self.exec_native_get_primitive_class()?;
            }
            ("java/lang/Object", "hashCode", "()I") => {
                self.exec_native_hash_code()?;
            }
            (_, "getClass", _) => {
                self.exec_native_get_class()?;
            }
            (_, "arrayBaseOffset0", _) => {
                self.exec_native_array_base_offset0()?;
            }
            (_, "arrayIndexScale0", _) => {
                self.exec_native_array_index_scale0()?;
            }
            ("java/lang/Class", "initClassName", "()Ljava/lang/String;") => {
                self.exec_native_init_class_name()?;
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
            ("java/lang/Class", "isPrimitive", "()Z") => {
                self.exec_native_is_primitive()?;
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
            ("jdk/internal/misc/Unsafe", "fullFence", "()V") => {
                self.exec_native_full_fence()?;
            }
            ("java/lang/Runtime", "availableProcessors", "()I") => {
                self.exec_native_available_processors()?;
            }
            ("java/lang/Thread", "getNextThreadIdOffset", "()J") => {
                self.exec_native_get_next_thread_id_offset()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "objectFieldOffset1",
                "(Ljava/lang/Class;Ljava/lang/String;)J",
            ) => {
                self.exec_native_object_field_offset1()?;
            }
            ("jdk/internal/misc/Unsafe", "compareAndSetInt", "(Ljava/lang/Object;JII)Z") => {
                self.exec_native_compare_and_set_int()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "compareAndSetReference",
                "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
            ) => {
                self.exec_native_compare_and_set_reference()?;
            }
            ("jdk/internal/misc/Unsafe", "compareAndSetLong", "(Ljava/lang/Object;JJJ)Z") => {
                self.exec_native_compare_and_set_long()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "getReferenceVolatile",
                "(Ljava/lang/Object;J)Ljava/lang/Object;",
            ) => {
                self.exec_native_get_reference_volatile()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "putReferenceVolatile",
                "(Ljava/lang/Object;JLjava/lang/Object;)V",
            ) => {
                self.exec_native_put_reference_volatile()?;
            }
            ("java/io/FileDescriptor", "initIDs", "()V") => {
                self.exec_native_filedescriptor_init_ids()?;
            }
            ("java/io/FileDescriptor", "getHandle", "(I)J") => {
                self.exec_native_get_handle()?;
            }
            ("java/io/FileDescriptor", "getAppend", "(I)Z") => {
                self.exec_native_get_append()?;
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
            ("jdk/internal/misc/Unsafe", "getLongVolatile", "(Ljava/lang/Object;J)J") => {
                self.exec_unsafe_get_long_volatile()?;
            }
            ("jdk/internal/misc/Unsafe", "getIntVolatile", "(Ljava/lang/Object;J)I") => {
                self.exec_unsafe_get_int_volatile()?;
            }
            ("jdk/internal/reflect/Reflection", "getCallerClass", "()Ljava/lang/Class;") => {
                self.exec_native_reflection_get_caller_class()?;
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
            ("java/lang/Class", "isArray", "()Z") => {
                self.exec_native_is_array()?;
            }
            ("jdk/internal/misc/Unsafe", "ensureClassInitialized0", "(Ljava/lang/Class;)V") => {
                self.exec_native_ensure_class_initialized0()?;
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
            (
                "jdk/internal/misc/Unsafe",
                "getReference",
                "(Ljava/lang/Object;J)Ljava/lang/Object;",
            ) => {
                self.exec_native_unsafe_get_reference()?;
            }
            ("jdk/internal/misc/Unsafe", "allocateMemory0", "(J)J") => {
                self.exec_native_allocate_memory0()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "copyMemory0",
                "(Ljava/lang/Object;JLjava/lang/Object;JJ)V",
            ) => {
                self.exec_native_copy_memory0()?;
            }
            ("jdk/internal/misc/Unsafe", "putByte", "(Ljava/lang/Object;JB)V") => {
                self.exec_native_put_byte()?;
            }
            ("sun/nio/fs/UnixNativeDispatcher", "stat0", "(JLsun/nio/fs/UnixFileAttributes;)I") => {
                self.exec_native_stat0()?;
            }
            ("java/lang/ref/Reference", "refersTo0", "(Ljava/lang/Object;)Z") => {
                self.exec_native_refers_to0()?;
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
                "java/lang/Class",
                "forName0",
                "(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;",
            ) => {
                self.exec_native_for_name0()?;
            }
            ("java/lang/Class", "getDeclaredMethods0", "(Z)[Ljava/lang/reflect/Method;") => {
                self.exec_native_get_declared_methods0()?;
            }
            ("java/lang/Class", "getConstantPool", "()Ljdk/internal/reflect/ConstantPool;") => {
                self.exec_native_get_constant_pool()?;
            }
            ("jdk/internal/reflect/Reflection", "getClassAccessFlags", "(Ljava/lang/Class;)I") => {
                self.exec_native_get_class_access_flags()?;
            }
            (
              "jdk/internal/reflect/DirectMethodHandleAccessor$NativeAccessor",
              "invoke0",
              "(Ljava/lang/reflect/Method;Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;"
            ) => {
                self.exec_native_invoke0()?;
            }
            ("jdk/internal/misc/Unsafe", "setMemory0", "(Ljava/lang/Object;JJB)V") => {
                self.exec_native_set_memory0()?;
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
            (
                "sun/nio/ch/UnixFileDispatcherImpl",
                "pread0",
                "(Ljava/io/FileDescriptor;JIJ)I"
            ) => {
                self.exec_native_pread0()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "getInt",
                "(Ljava/lang/Object;J)I"
            ) => {
                self.exec_native_unsafe_get_int()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "getLong",
                "(Ljava/lang/Object;J)J"
            ) => {
                self.exec_native_unsafe_get_long()?;
            }
            (
                "jdk/internal/misc/Unsafe",
                "getByte",
                "(Ljava/lang/Object;J)B"
            ) => {
                self.exec_native_unsafe_get_byte()?;
            }
            (
                "sun/nio/ch/UnixFileDispatcherImpl",
                "size0",
                "(Ljava/io/FileDescriptor;)J"
            ) => {
                self.exec_native_size0()?;
            }
            (
                "sun/nio/ch/UnixFileDispatcherImpl",
                "allocationGranularity0",
                "()J"
            ) => {
                self.exec_native_allocation_granularity0()?;
            }
            (
                "sun/nio/ch/UnixFileDispatcherImpl",
                "map0",
                "(Ljava/io/FileDescriptor;IJJZ)J"
            ) => {
                self.exec_native_map0()?;
            }
            ("java/lang/Class", "isInterface", "()Z") => {
                self.exec_native_class_is_interface()?;
            }
            (
                "java/lang/Class",
                "getDeclaredConstructors0",
                "(Z)[Ljava/lang/reflect/Constructor;"
            ) => {
                self.exec_native_get_declared_constructors0()?;
            }
            (
                "java/lang/Class",
                "getModifiers",
                "()I"
            ) => {
                self.exec_native_get_modifiers()?;
            }
            (
                "jdk/internal/reflect/DirectConstructorHandleAccessor$NativeAccessor",
                "newInstance0",
                "(Ljava/lang/reflect/Constructor;[Ljava/lang/Object;)Ljava/lang/Object;"
            ) => {
                self.exec_native_new_instance0()?;
            }
            (
                "java/io/UnixFileSystem",
                "canonicalize0",
                "(Ljava/lang/String;)Ljava/lang/String;"
            ) => {
                self.exec_native_canonicalize0()?;
            }
            (
                "java/lang/Class",
                "isAssignableFrom",
                "(Ljava/lang/Class;)Z"
            ) => {
                self.exec_native_is_assignable_from()?;
            }
            (
                "java/lang/Class",
                "getSuperclass",
                "()Ljava/lang/Class;"
            ) => {
                self.exec_native_get_superclass()?;
            }
            (
                "java/lang/Module",
                "defineModule0",
                "(Ljava/lang/Module;ZLjava/lang/String;Ljava/lang/String;[Ljava/lang/Object;)V"
            ) => {
                self.exec_native_define_module0()?;
            }
            (
                "java/lang/Double",
                "longBitsToDouble",
                "(J)D"
            ) => {
                self.exec_native_long_bits_to_double()?; 
            }
            (
                "java/lang/Module",
                "addReads0",
                "(Ljava/lang/Module;Ljava/lang/Module;)V"
            ) => {
                self.exec_native_add_reads0()?;
            }
            (
                "java/lang/Module",
                "addExportsToAll0",
                "(Ljava/lang/Module;Ljava/lang/String;)V"
            ) => {
                self.exec_native_add_exports_to_all0()?;
            }
            (
                "java/lang/Module",
                "addExports0",
                "(Ljava/lang/Module;Ljava/lang/String;Ljava/lang/Module;)V"
            ) => {
                self.exec_native_add_exports0()?;
            }
            (
                "java/io/FileOutputStream",
                "initIDs",
                "()V"
            ) => {
                self.exec_native_fileoutputstream_initids()?;
            }
            (
                "java/io/FileOutputStream",
                "writeBytes",
                "([BIIZ)V"
            ) => {
                self.exec_native_fileoutputstream_write_bytes()?;
            }
            (
                "java/net/InetAddress",
                "init",
                "()V"
            ) => {
                self.exec_native_inetaddress_init()?;
            }
            ("java/net/InetAddress", "isIPv6Supported", "()Z") => {
                self.exec_native_is_ipv6_supported()?;
            }
            ("java/net/InetAddress", "isIPv4Available", "()Z") => {
                self.exec_native_is_ipv4_available()?;
            }
            ("java/net/Inet4Address", "init", "()V"
            ) => {
                self.exec_native_inet4address_init()?;
            }
            (
                "java/lang/invoke/MethodHandleNatives",
                "getNamedCon",
                "(I[Ljava/lang/Object;)I"
            ) => {
                self.exec_native_get_named_con()?;
            }
            (
                "java/lang/invoke/MethodHandleNatives",
                "resolve",
                "(Ljava/lang/invoke/MemberName;Ljava/lang/Class;IZ)Ljava/lang/invoke/MemberName;"
            ) => {
                self.exec_native_resolve_member_name()?;
            }
            (   "java/lang/invoke/MethodHandleNatives", 
                "getMemberVMInfo", 
                "(Ljava/lang/invoke/MemberName;)Ljava/lang/Object;"
            ) => {
                self.exec_native_get_member_vm_info()?;
            }
            _ => {
                error!("NATIVE NOT IMPL {} {} {}", class_name, name, type_str);
                return Err(eyre!(errors::RuntimeError::NotImplementedException));
            }
        }
        Ok(None)
    }
}
