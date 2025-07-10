use color_eyre::eyre::{eyre, Result};
use log::{debug, warn};

use crate::{
  notimpl,
  runtime::{errors, jvm::*, types},
  utils::get_index_scale,
};

impl JVM {
  pub(crate) fn native_dispatcher_jdk_internal_misc_unsafe(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("arrayBaseOffset0", "(Ljava/lang/Class;)I") => self.exec_native_array_base_offset0(),
      ("arrayIndexScale0", "(Ljava/lang/Class;)I") => self.exec_native_array_index_scale0(),
      ("fullFence", "()V") => self.exec_native_full_fence(),
      ("objectFieldOffset1", "(Ljava/lang/Class;Ljava/lang/String;)J") => {
        self.exec_native_object_field_offset1()
      }
      ("compareAndSetReference", "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z") => {
        self.exec_native_compare_and_set_reference()
      }
      ("compareAndSetLong", "(Ljava/lang/Object;JJJ)Z") => self.exec_native_compare_and_set_long(),
      ("getReferenceVolatile", "(Ljava/lang/Object;J)Ljava/lang/Object;") => {
        self.exec_native_get_reference_volatile()
      }
      ("putReferenceVolatile", "(Ljava/lang/Object;JLjava/lang/Object;)V") => {
        self.exec_native_put_reference_volatile()
      }
      ("getLongVolatile", "(Ljava/lang/Object;J)J") => self.exec_unsafe_get_long_volatile(),
      ("getIntVolatile", "(Ljava/lang/Object;J)I") => self.exec_unsafe_get_int_volatile(),
      ("ensureClassInitialized0", "(Ljava/lang/Class;)V") => {
        self.exec_native_ensure_class_initialized0()
      }
      ("getReference", "(Ljava/lang/Object;J)Ljava/lang/Object;") => {
        self.exec_native_unsafe_get_reference()
      }
      ("putReference", "(Ljava/lang/Object;JLjava/lang/Object;)V") => {
        self.exec_native_unsafe_put_reference()
      }
      ("copyMemory0", "(Ljava/lang/Object;JLjava/lang/Object;JJ)V") => {
        self.exec_native_copy_memory0()
      }
      ("setMemory0", "(Ljava/lang/Object;JJB)V") => self.exec_native_set_memory0(),
      ("getInt", "(Ljava/lang/Object;J)I") => self.exec_native_unsafe_get_int(),
      ("getLong", "(Ljava/lang/Object;J)J") => self.exec_native_unsafe_get_long(),
      ("getByte", "(Ljava/lang/Object;J)B") => self.exec_native_unsafe_get_byte(),
      ("shouldBeInitialized0", "(Ljava/lang/Class;)Z") => self.exec_native_should_be_initialized0(),
      ("compareAndSetInt", "(Ljava/lang/Object;JII)Z") => self.exec_native_compare_and_set_int(),
      ("allocateMemory0", "(J)J") => self.exec_native_allocate_memory0(),
      ("putByte", "(Ljava/lang/Object;JB)V") => self.exec_native_put_byte(),
      ("freeMemory0", "(J)V") => self.exec_native_jdk_internal_misc_unsafe_free_memory0(),
      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "jdk/internal/misc/Unsafe".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  // private native int arrayBaseOffset0(Class<?> arrayClass);
  fn exec_native_array_base_offset0(&mut self) -> Result<Option<types::Type>> {
    let array_class = self.pop_stack()?;

    match array_class {
      types::Type::ObjectRef(obj_ref) => {
        let array_obj = self.heap.get_obj_instance(obj_ref)?;

        let raw_value_ref = array_obj.get_field("name")?;

        match raw_value_ref {
          types::Type::ObjectRef(value_ref) => {
            let value_obj = self.heap.get_obj_instance(value_ref)?;

            let value_field = value_obj.get_field("value")?;

            if let types::Type::ArrayRef(array_ref) = value_field {
              let array = self.heap.get_array_instance(array_ref)?;

              array.print();

              // TODO: Fixed now
              let ret_value = types::Type::Integer(0);
              self.push_stack(ret_value)?;
              return Ok(Some(ret_value));
            }
          }
          _ => notimpl!(),
        }
      }
      _ => notimpl!(),
    }

    notimpl!();
  }

  // private static native int arrayIndexScale0(Class clazz);
  fn exec_native_array_index_scale0(&mut self) -> Result<Option<types::Type>> {
    let array_class = self.pop_stack()?;

    match array_class {
      types::Type::ObjectRef(obj_ref) => {
        let array_obj = self.heap.get_obj_instance(obj_ref)?; // java.lang.Class
        let raw_value_ref = array_obj.get_field("name")?.as_ref()?;

        let inner_class_name = self.heap.get_string(raw_value_ref)?; // [Z, [B, etc
        let ret_value = types::Type::Integer(get_index_scale(&inner_class_name) as i32);
        self.push_stack(ret_value)?;
        Ok(Some(ret_value))
      }
      _ => notimpl!(),
    }
  }

  // public native void fullFence();
  fn exec_native_full_fence(&mut self) -> Result<Option<types::Type>> {
    warn!("Full Fence only works for single-threading");

    // No-op if single-threaded
    Ok(None)
  }

  // public native long objectFieldOffset1(Class class, String field);
  fn exec_native_object_field_offset1(&mut self) -> Result<Option<types::Type>> {
    let field_name = self.pop_string()?;

    let class_name = self.pop_class_name()?;

    let offset = self
      .class_loader
      .get_field_offset(&class_name, &field_name)?;

    debug!("{:?} {:?} {:?}", class_name, field_name, offset);

    let ret_value = types::Type::Long(offset);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // public native boolean compareAndSetReference(Object obj, long offset, Object expected, Object
  // newVal)
  fn exec_native_compare_and_set_reference(&mut self) -> Result<Option<types::Type>> {
    warn!("Compare and set reference only works for single-threading");

    let new_value = self.pop_stack()?;

    let expected_value = self.pop_ref()?;

    let offset = self.pop_loperand()?;

    let obj_ref = self.pop_ref()?;

    let obj = self.heap.get_instance_mut(obj_ref)?;

    let status = match obj {
      types::Instance::ObjectInstance(obj) => {
        let class_name = obj.get_classname();

        let field = self.class_loader.get_field_by_offset(class_name, offset)?;
        let field_name = field.get_name();

        let field = obj.get_field(field_name)?;

        let field_value = match field {
          types::Type::ObjectRef(val) => val,
          types::Type::ArrayRef(val) => val,
          types::Type::Null => 0,
          other => return Err(eyre!("Expected ref at offset, got {:?}", other)),
        };

        if field_value == expected_value {
          obj.put_field(field_name, new_value)?;

          true
        } else {
          false
        }
      }
      types::Instance::ArrayInstance(obj) => {
        let field = obj.get_with_index_scale(offset as usize)?;

        let field_value = match field {
          types::Type::ObjectRef(val) => *val,
          types::Type::ArrayRef(val) => *val,
          types::Type::Null => 0,
          other => return Err(eyre!("Expected ref at offset, got {:?}", other)),
        };

        if field_value == expected_value {
          obj.set_with_index_scale(offset as usize, new_value)?;

          true
        } else {
          false
        }
      }
    };

    let ret_value = types::Type::Boolean(status);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // public native boolean compareAndSetLong(Object obj, long offset, long expected, long newVal)
  fn exec_native_compare_and_set_long(&mut self) -> Result<Option<types::Type>> {
    warn!("Compare and set long only works for single-threading");

    let new_value = self.pop_loperand()?;

    let expected_value = self.pop_loperand()?;

    let offset = self.pop_loperand()?;

    let obj_ref = self.pop_ref()?;

    let status = if obj_ref == 0 {
      debug!("static field {}", offset);

      let field = self
        .class_loader
        .get_field_by_offset("java/lang/Thread", offset)?;
      let field_name = field.get_name();

      let mut class = self.class_loader.get_mut("java/lang/Thread")?;

      let field = class.get_static_field(field_name)?;

      let field_value = field.as_long()?;

      if field_value == expected_value {
        class.put_static_field(field_name, types::Type::Long(new_value))?;

        true
      } else {
        false
      }
    } else {
      let obj = self.heap.get_instance_mut(obj_ref)?;

      match obj {
        types::Instance::ObjectInstance(obj) => {
          let class_name = obj.get_classname();

          let field = self.class_loader.get_field_by_offset(class_name, offset)?;
          let field_name = field.get_name();

          let field = obj.get_field(field_name)?;

          let field_value = field.as_long()?;

          if field_value == expected_value {
            obj.put_field(field_name, types::Type::Long(new_value))?;

            true
          } else {
            false
          }
        }
        types::Instance::ArrayInstance(obj) => {
          let field = obj.get_with_index_scale(offset as usize)?;

          let field_value = field.as_long()?;

          if field_value == expected_value {
            obj.set_with_index_scale(offset as usize, types::Type::Long(new_value))?;

            true
          } else {
            false
          }
        }
      }
    };

    let ret_value = types::Type::Boolean(status);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // public native Object getReferenceVolatile(Object obj, long offset)
  fn exec_native_get_reference_volatile(&mut self) -> Result<Option<types::Type>> {
    warn!("get reference only works for single-threading");

    let offset = self.pop_loperand()?;

    let obj_ref = self.pop_ref()?;

    let obj = self.heap.get_instance(obj_ref)?;

    let ret_value = match obj {
      types::Instance::ObjectInstance(obj) => {
        let obj_classname = obj.get_classname();
        let field = self
          .class_loader
          .get_field_by_offset(obj_classname, offset)?;
        let field_name = field.get_name();

        let value = if field.is_static() {
          let class = self.class_loader.get(obj_classname)?;
          let field_type = class.get_static_field(field_name)?;
          *field_type
        } else {
          obj.get_field(field_name)?
        };

        match value {
          types::Type::ObjectRef(_) => value,
          _ => return Err(eyre!("Expected ObjectRef at offset, got {:?}", value)),
        }
      }
      types::Instance::ArrayInstance(obj) => {
        let value = obj.get_with_index_scale(offset as usize)?;
        *value
      }
    };

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // public native void putReferenceVolatile(Object obj, long offset, Object newValue)
  fn exec_native_put_reference_volatile(&mut self) -> Result<Option<types::Type>> {
    warn!("put reference only works for single-threading");

    let new_value = self.pop_stack()?;

    let offset = self.pop_loperand()?;

    let obj_ref = self.pop_ref()?;

    let obj = self.heap.get_instance_mut(obj_ref)?;

    match obj {
      types::Instance::ObjectInstance(obj) => {
        let obj_classname = obj.get_classname();
        let field = self
          .class_loader
          .get_field_by_offset(obj_classname, offset)?;
        let field_name = field.get_name();

        if field.is_static() {
          let mut class = self.class_loader.get_mut(obj_classname)?;
          class.put_static_field(field_name, new_value)?;
        } else {
          obj.put_field(field_name, new_value)?;
        }
      }
      types::Instance::ArrayInstance(obj) => {
        obj.set_with_index_scale(offset as usize, new_value)?;
      }
    };

    Ok(None)
  }

  fn exec_unsafe_get_long_volatile(&mut self) -> Result<Option<types::Type>> {
    // TODO:
    warn!("GetLongVolatile not fully implemented!");

    let offset = self.pop_loperand()?;

    let obj_ref = self.pop_ref()?;

    let field = self.get_volatile_field(offset, obj_ref)?;

    let field_value = field.as_long()?;

    let ret_value = types::Type::Long(field_value);
    self.push_stack(ret_value)?;
    Ok(None)
  }

  fn exec_unsafe_get_int_volatile(&mut self) -> Result<Option<types::Type>> {
    // TODO:
    warn!("GetIntVolatile not fully implemented!");

    let offset = self.pop_loperand()?;

    let obj_ref = self.pop_ref()?;

    let field = self.get_volatile_field(offset, obj_ref)?;

    let field_value = field.as_integer()?;

    let ret_value = types::Type::Integer(field_value);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_ensure_class_initialized0(&mut self) -> Result<Option<types::Type>> {
    let class_ref = self.pop_object_ref()?;

    let class_obj = self.heap.get_obj_instance(class_ref)?;

    let class_name_field = class_obj.get_field("name")?.as_ref()?;

    let class_name_str = self.heap.get_string(class_name_field)?;

    self.init_class(&class_name_str)?;

    Ok(None)
  }

  fn exec_native_unsafe_get_reference(&mut self) -> Result<Option<types::Type>> {
    warn!("jdk/internal/misc/Unsafe.getReference not properly implemented");
    let offset = self.pop_loperand()?;

    let obj = self.pop_stack()?;

    let _this = self.pop_stack()?;

    let ret_value = if obj == types::Type::Null {
      types::Type::Null
    } else {
      let obj_ref = obj.as_ref()?;

      let instance = self.heap.get_obj_instance(obj_ref)?;
      let instance_classname = instance.get_classname();

      let field = self
        .class_loader
        .get_field_by_offset(instance.get_classname(), offset)?;
      let field_name = field.get_name();

      if field.is_static() {
        let class = self.class_loader.get(instance_classname)?;
        let field_type = class.get_static_field(field_name)?;
        *field_type
      } else {
        instance.get_field(field_name)?
      }
    };

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_unsafe_put_reference(&mut self) -> Result<Option<types::Type>> {
    warn!("jdk/internal/misc/Unsafe.putReference not properly implemented");

    let value = self.pop_stack()?;

    let offset = self.pop_loperand()?;

    let base = self.pop_stack()?;

    let _this = self.pop_stack()?;

    if base == types::Type::Null {
      warn!("jdk/internal/misc/Unsafe.putReference not implemented for static");
      notimpl!()
    }

    let base_ref = base.as_ref()?;

    let instance = self.heap.get_obj_instance_mut(base_ref)?;
    let instance_classname = instance.get_classname();

    let field = self
      .class_loader
      .get_field_by_offset(instance_classname, offset)?;
    let field_name = field.get_name();

    // if the field is static
    if field.is_static() {
      let mut class = self.class_loader.get_mut(instance_classname)?;
      class.put_static_field(field_name, value)?;
    } else {
      instance.put_field(field_name, value)?;
    }

    Ok(None)
  }

  fn exec_native_copy_memory0(&mut self) -> Result<Option<types::Type>> {
    let size = self.pop_loperand()?;

    let dest_offset = self.pop_loperand()?;

    let dest_obj = self.pop_stack()?;

    let src_offset = self.pop_loperand()? as usize;

    let src_obj = self.pop_stack()?;

    // Get source pointer
    if let types::Type::ArrayRef(src_ref) = src_obj {
      let src = self.heap.get_array_instance(src_ref)?;

      if let types::Type::ArrayRef(dst_ref) = dest_obj {
        let _dest = self.heap.get_array_instance(dst_ref)?;

        panic!()
      } else if dest_obj == types::Type::Null {
        self.nativememory.is_valid(dest_offset as u64);

        let dest: *mut u8 = dest_offset as *mut u8;

        for i in 0..(size as usize) {
          let val = src.get(src_offset + i)?.as_byte()?;

          let addr = dest.wrapping_add(i);

          unsafe {
            *addr = val as u8;
          }
        }
      } else {
        return Err(eyre!("Invalid srcBase type"));
      };
    } else if src_obj == types::Type::Null {
      panic!()
    } else {
      return Err(eyre!("Invalid srcBase type"));
    };

    Ok(None)
  }

  fn exec_native_set_memory0(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?.as_byte()? as u8;

    let size = self.pop_stack()?.as_long()? as usize;

    let offset = self.pop_stack()?.as_long()? as usize;

    let base = self.pop_stack()?; // can be null
    match base {
      types::Type::Null => {
        // Off-heap memory
        self.nativememory.is_valid(offset as u64);

        let dest: *mut u8 = offset as *mut u8;

        for i in 0..size {
          let addr = dest.wrapping_add(i);

          unsafe {
            *addr = value;
          }
        }
      }
      types::Type::ArrayRef(array_ref) => {
        let array = self.heap.get_array_instance_mut(array_ref)?;

        let array_len = array.len();

        for i in 0..array_len {
          array.set(i, types::Type::Byte(value as i8))?;
        }
      }
      _ => panic!(),
    }

    Ok(None)
  }

  fn exec_native_unsafe_get_int(&mut self) -> Result<Option<types::Type>> {
    // this offset is computed based on index scale (works for real array but not for my array)
    let offset = self.pop_stack()?.as_long()? as usize;

    let obj = self.pop_stack()?;

    let ret = match obj {
      types::Type::ArrayRef(obj_ref) => {
        let src = self.heap.get_array_instance(obj_ref)?;

        types::Type::Integer(src.get_with_index_scale(offset)?.as_integer()?)
      }
      types::Type::Null => {
        dbg!();

        let src: *const i32 = offset as *const i32;

        debug!("{:?} {:?} {:?}", src, src.is_null(), src.is_aligned());

        self.nativememory.is_valid(offset as u64);

        types::Type::Integer(unsafe { *src })
      }
      _ => return Err(eyre!("Invalid srcBase type")),
    };

    self.push_stack(ret)?;
    Ok(Some(ret))
  }

  fn exec_native_unsafe_get_long(&mut self) -> Result<Option<types::Type>> {
    // this offset is computed based on index scale (works for real array but not for my array)
    let offset = self.pop_stack()?.as_long()? as usize;

    let obj = self.pop_stack()?;

    let ret = match obj {
      types::Type::ArrayRef(obj_ref) => {
        let src = self.heap.get_array_instance(obj_ref)?;

        types::Type::Long(src.get_with_index_scale(offset)?.as_long()?)
      }
      types::Type::Null => {
        self.nativememory.is_valid(offset as u64);

        let src: *mut u64 = offset as *mut u64;

        types::Type::Long(unsafe { *src as i64 })
      }
      _ => return Err(eyre!("Invalid srcBase type")),
    };

    self.push_stack(ret)?;
    Ok(Some(ret))
  }

  fn exec_native_unsafe_get_byte(&mut self) -> Result<Option<types::Type>> {
    // this offset is computed based on index scale (works for real array but not for my array)
    let offset = self.pop_stack()?.as_long()? as usize;

    let obj = self.pop_stack()?;

    let ret = match obj {
      types::Type::ArrayRef(obj_ref) => {
        let src = self.heap.get_array_instance(obj_ref)?;

        types::Type::Byte(src.get_with_index_scale(offset)?.as_byte()?)
      }
      types::Type::Null => {
        self.nativememory.is_valid(offset as u64);

        let src: *mut u8 = offset as *mut u8;

        types::Type::Byte(unsafe { *src as i8 })
      }
      _ => return Err(eyre!("Invalid srcBase type")),
    };

    self.push_stack(ret)?;
    Ok(Some(ret))
  }

  pub fn exec_native_should_be_initialized0(&mut self) -> Result<Option<types::Type>> {
    let class_ref = self.pop_object_ref()?;

    let class = self
      .heap
      .get_class_from_class_obj(&mut self.class_loader, class_ref)?;

    let should_init = !class.get_init();
    drop(class);

    let ret_value = types::Type::Boolean(should_init);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // public native boolean compareAndSetInt(Object obj, long offset, int expected, int newVal)
  fn exec_native_compare_and_set_int(&mut self) -> Result<Option<types::Type>> {
    warn!("Compare and set int only works for single-threading");

    let new_value = self.pop_ioperand()?;

    let expected_value = self.pop_ioperand()?;

    let offset = self.pop_loperand()?;

    let obj_ref = self.pop_ref()?;

    let status = if obj_ref == 0 {
      debug!("static field {}", offset);
      let field = self
        .class_loader
        .get_field_by_offset("java/lang/Thread", offset)?;
      let field_name = field.get_name();

      let mut class = self.class_loader.get_mut("java/lang/Thread")?;

      let field = class.get_static_field(field_name)?;

      let field_value = field.as_integer()?;

      if field_value == expected_value {
        class.put_static_field(field_name, types::Type::Integer(new_value))?;

        true
      } else {
        false
      }
    } else {
      let obj = self.heap.get_instance_mut(obj_ref)?;

      match obj {
        types::Instance::ObjectInstance(obj) => {
          let class_name = obj.get_classname();

          let field = self.class_loader.get_field_by_offset(class_name, offset)?;
          let field_name = field.get_name();

          let field = obj.get_field(field_name)?;

          let field_value = field.as_integer()?;

          if field_value == expected_value {
            obj.put_field(field_name, types::Type::Integer(new_value))?;

            true
          } else {
            false
          }
        }
        types::Instance::ArrayInstance(obj) => {
          let field = obj.get_with_index_scale(offset as usize)?;

          let field_value = field.as_integer()?;

          if field_value == expected_value {
            obj.set_with_index_scale(offset as usize, types::Type::Integer(new_value))?;

            true
          } else {
            false
          }
        }
      }
    };

    let ret_value = types::Type::Boolean(status);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_allocate_memory0(&mut self) -> Result<Option<types::Type>> {
    let size = self.pop_loperand()?;
    let address = self.nativememory.alloc(size as u64)?;

    let ret_value = types::Type::Long(address as i64);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  fn exec_native_put_byte(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?.as_byte()?;

    let offset = self.pop_stack()?.as_long()? as usize;

    let obj = self.pop_stack()?;

    if let types::Type::ArrayRef(obj_ref) = obj {
      let src = self.heap.get_array_instance_mut(obj_ref)?;

      src.set_with_index_scale(offset, types::Type::Byte(value))?;
    } else if obj == types::Type::Null {
      self.nativememory.is_valid(offset as u64);

      let src: *mut u8 = offset as *mut u8;

      unsafe {
        *src = value as u8;
      }
    } else {
      return Err(eyre!("Invalid srcBase type"));
    };

    Ok(None)
  }

  fn exec_native_jdk_internal_misc_unsafe_free_memory0(&mut self) -> Result<Option<types::Type>> {
    let address = self.pop_stack()?.as_long()?;

    unsafe {
      libc::free(address as *mut libc::c_void);
    }

    Ok(None)
  }
}
