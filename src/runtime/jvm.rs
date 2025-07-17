use crate::{
  class_loader::{
    class_file,
    constant_pool::{self, CpInfoInfoEnum},
    loader::ClassLoader,
  },
  notimpl,
  runtime::{
    errors, frame, heap,
    lambdamanager::LambdaManager,
    nativememory::NativeMemory,
    opcode,
    types::{self},
  },
  utils::*,
};
use color_eyre::eyre::{eyre, OptionExt, Result};
use core::panic;
use log::{debug, error, info, warn};
use std::{
  borrow::BorrowMut,
  sync::{Arc, Mutex, RwLockReadGuard},
};
use tracing_subscriber::{filter, reload, Registry};

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct JVM {
  pub(crate) frames: Vec<frame::Frame>,
  pub(crate) heap: heap::Heap,
  pub(crate) class_loader: ClassLoader,
  pub(crate) nativememory: NativeMemory,
  pub(crate) lambdamanager: LambdaManager,

  pub(crate) counter: u64, // number of instructions executed

  // Threads
  current_thread: types::Type,
  next_thread_id: i64,

  // Boot Loader
  boot_loader_unnamed_module: Option<ju4>,
  // Logging
  logging_handle: Option<Arc<Mutex<reload::Handle<filter::LevelFilter, Registry>>>>,
}

impl JVM {
  #[allow(dead_code)]
  pub(crate) fn mock(user_dir: &str, class_names: Vec<String>, bootstrap: bool) -> Result<JVM> {
    let mut classes = ClassLoader::new();

    for class_name in class_names {
      classes.add(class_name)?;
    }

    let mut jvm = JVM {
      heap: heap::Heap::new(),
      frames: vec![],
      class_loader: classes,
      nativememory: NativeMemory::new(),
      lambdamanager: LambdaManager::new(),

      counter: 0,
      current_thread: types::Type::None,
      next_thread_id: 0,
      boot_loader_unnamed_module: None,
      logging_handle: None,
    };

    if bootstrap {
      jvm.bootstrap(user_dir)?;
    } else {
      jvm.bootstrap_mock(user_dir)?;
    }

    Ok(jvm)
  }

  pub fn build(user_dir: &str, class_names: Vec<String>) -> Result<JVM> {
    let mut classes = ClassLoader::new();

    for class_name in class_names {
      classes.add(class_name)?;
    }

    let mut jvm = JVM {
      heap: heap::Heap::new(),
      frames: vec![],
      class_loader: classes,
      nativememory: NativeMemory::new(),
      lambdamanager: LambdaManager::new(),

      counter: 0,
      current_thread: types::Type::None,
      next_thread_id: 0,
      boot_loader_unnamed_module: None,
      logging_handle: None,
    };

    jvm.bootstrap(user_dir)?;

    Ok(jvm)
  }

  pub(crate) fn set_logging_handle(
    &mut self,
    handle: Arc<Mutex<reload::Handle<filter::LevelFilter, Registry>>>,
  ) {
    self.logging_handle = Some(handle);
  }

  fn init_phase_1(&mut self) -> Result<()> {
    debug!("[-] Init Phase 1");

    self.call_and_resolve_method("java/lang/System", "initPhase1", "()V", vec![])?;

    debug!("[-] Init Phase 1 Done");

    Ok(())
  }

  fn init_phase_2(&mut self) -> Result<()> {
    debug!("[-] Init Phase 2");

    self.call_and_resolve_method(
      "java/lang/System",
      "initPhase2",
      "(ZZ)I",
      vec![types::Type::Boolean(false), types::Type::Boolean(false)],
    )?;

    debug!("[-] Init Phase 2 Done");

    Ok(())
  }

  fn init_phase_3(&mut self) -> Result<()> {
    debug!("[-] Init Phase 3");

    self.call_and_resolve_method("java/lang/System", "initPhase3", "()V", vec![])?;

    debug!("[-] Init Phase 3 Done");

    Ok(())
  }

  fn bootstrap_primitive_types(&mut self) -> Result<()> {
    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "C")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "Z")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "B")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "S")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "I")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "L")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "F")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "D")?;

    self
      .heap
      .alloc_primitive_class_obj(&mut self.class_loader, "V")?;

    Ok(())
  }

  fn bootstrap_user_module(&mut self, module_name: &str, user_dir: &str) -> Result<()> {
    // predicting the object reference
    let module_java_base_ref = self.heap.get_curr_obj_ref();

    // MUST load the modules before anything else (used during class resolution !)
    let packages = get_classnames_in_module(user_dir);

    self.class_loader.modulemanager.add(
      module_java_base_ref,
      module_name,
      true,
      None,
      user_dir.to_owned(),
      packages.clone(),
    );

    // this is the object instance of the module for java.base
    self
      .heap
      .alloc_obj(&mut self.class_loader, "java/lang/Module")?
      .as_ref()?;

    let module_descriptor = self.alloc_module_descriptor(user_dir, "", "", vec![])?;

    let module_java_base = self.heap.get_obj_instance_mut(module_java_base_ref)?;

    module_java_base.put_field("descriptor", module_descriptor)?;

    Ok(())
  }

  fn load_module(&mut self, module_name: &str) -> Result<()> {
    // predicting the object reference
    let module_ref = self.heap.get_curr_obj_ref();

    let path = format!("{}/{}", get_env("JMODS", ""), module_name);

    // MUST load the modules before anything else (used during class resolution !)
    let packages = if module_name == "unnamed" {
      vec![]
    } else {
      get_classnames_in_module(&path)
    };

    self.class_loader.modulemanager.add(
      module_ref,
      module_name,
      true,
      None,
      path.clone(),
      packages.clone(),
    );

    // this is the object instance of the module for java.base
    self
      .heap
      .alloc_obj(&mut self.class_loader, "java/lang/Module")?
      .as_ref()?;

    // setup primitive types since they are needed
    self.bootstrap_primitive_types()?;

    // need to create the module descriptor and associate it to the module
    //let module_descriptor = self.alloc_module_descriptor("java.base", "", "", packages)?;
    // TODO: passing the packages means executing A LOT of instructions >= 2M
    let module_descriptor = self.alloc_module_descriptor(module_name, "", "", vec![])?;

    let module_java_base = self.heap.get_obj_instance_mut(module_ref)?;

    module_java_base.put_field("descriptor", module_descriptor)?;

    Ok(())
  }

  // setup bootstrap class loader and relatives modules (java.base, etc)
  fn bootstrap_java_base(&mut self) -> Result<()> {
    self.load_module("java.base")
  }

  // TODO: any module but java.base should be lazily loaded
  fn bootstrap_jdk_net(&mut self) -> Result<()> {
    self.load_module("jdk.net")
  }

  fn bootstrap(&mut self, user_dir: &str) -> Result<()> {
    debug!("[-] Bootstrap");

    self.bootstrap_java_base()?;
    // TODO: this should be called only if needed
    self.bootstrap_jdk_net()?;

    // create unnamed module, must be after java base init
    self.load_module("unnamed")?;

    self.bootstrap_user_module("usermodule", user_dir)?;

    self.init_class("java/lang/System")?;

    self.current_thread = self.create_main_thread("MainThread")?;

    self.init_class("java/lang/Thread")?;

    // TODO: for some reason it is never used in initPhase1 :(
    self.init_class("java/lang/ref/Reference")?;

    self.init_class("java/lang/reflect/AccessibleObject")?;

    // initializeSystemClass (should be called after register natives)
    // Initialize the system class. Called after thread initialization.
    // initPhase1
    self.init_phase_1()?;

    // initPhase2
    // TODO: is it needed ?
    self.init_phase_2()?;

    // initPhase3
    // TODO: is it needed ?
    //self.init_phase_3()?;

    Ok(())
  }

  fn bootstrap_mock(&mut self, user_dir: &str) -> Result<()> {
    debug!("[-] Bootstrap Mock");

    self.bootstrap_java_base()?;

    self.bootstrap_user_module("usermodule", user_dir)?;

    Ok(())
  }

  pub(crate) fn _get_counter(&self) -> u64 {
    self.counter
  }

  pub(crate) fn get_current_frame(&self) -> Result<&frame::Frame> {
    self
      .frames
      .last()
      .ok_or_eyre(errors::InternalError::FrameNotFound)
  }

  pub(crate) fn get_current_frame_mut(&mut self) -> Result<&mut frame::Frame> {
    self
      .frames
      .last_mut()
      .ok_or_eyre(errors::InternalError::FrameNotFound)
  }

  fn create_main_thread(&mut self, name: &str) -> Result<types::Type> {
    let thread_group_obj = self
      .heap
      .alloc_obj(&mut self.class_loader, "java/lang/ThreadGroup")?;

    self.call_and_resolve_method(
      "java/lang/ThreadGroup",
      "<init>",
      "()V",
      vec![thread_group_obj],
    )?;

    let name_string = self.heap.alloc_string(&mut self.class_loader, name)?;

    let thread_obj = self
      .heap
      .alloc_obj(&mut self.class_loader, "java/lang/Thread")?;

    self.current_thread = thread_obj;

    self
      .class_loader
      .get_mut("java/lang/Thread")?
      .put_static_field("nextThreadID", types::Type::Long(self.next_thread_id))?;

    self
      .class_loader
      .get_mut("java/lang/Thread")?
      .new_field("nextThreadID", "J")?;

    self.next_thread_id += 1;

    self.call_and_resolve_method(
      "java/lang/Thread",
      "<init>",
      "(Ljava/lang/ThreadGroup;Ljava/lang/String;)V",
      vec![thread_obj, thread_group_obj, name_string],
    )?;

    Ok(thread_obj)
  }

  fn alloc_set(&mut self, elements: Vec<types::Type>) -> Result<types::Type> {
    // create the object
    let set_ref = self
      .heap
      .alloc_obj(&mut self.class_loader, "java/util/HashSet")?
      .as_ref()?;

    // init
    self.call_and_resolve_method(
      "java/util/HashSet",
      "<init>",
      "(I)V",
      vec![
        types::Type::ObjectRef(set_ref),
        types::Type::Integer(elements.len() as i32),
      ],
    )?;

    // populate the set
    for e in elements {
      self.call_and_resolve_method(
        "java/util/HashSet",
        "add",
        "(Ljava/lang/Object;)Z",
        vec![types::Type::ObjectRef(set_ref), e],
      )?;
    }

    Ok(types::Type::ObjectRef(set_ref))
  }

  pub fn alloc_module_descriptor(
    &mut self,
    name: &str,
    version: &str,
    main_class: &str,
    packages: Vec<String>,
  ) -> Result<types::Type> {
    let name_obj = self
      .heap
      .alloc_string(&mut self.class_loader, &class_to_dotclass(name))?;

    let version_obj = self.heap.alloc_string(&mut self.class_loader, version)?;

    let main_class_obj = self.heap.alloc_string(&mut self.class_loader, main_class)?;

    let modifiers = self.alloc_set(vec![])?;

    let requires = self.alloc_set(vec![])?;

    let exports = self.alloc_set(vec![])?;

    let opens = self.alloc_set(vec![])?;

    let uses = self.alloc_set(vec![])?;

    let provides = self.alloc_set(vec![])?;

    let mut packages_vec = vec![];

    for p in packages {
      let p_ref = self.heap.alloc_string(&mut self.class_loader, &p)?;

      packages_vec.push(p_ref);
    }

    let packages = self.alloc_set(packages_vec)?;

    let descriptor_ref = self
      .heap
      .alloc_obj(&mut self.class_loader, "java/lang/module/ModuleDescriptor")?
      .as_ref()?;

    let descriptor = self.heap.get_obj_instance_mut(descriptor_ref)?;

    descriptor.put_field("name", name_obj)?;

    descriptor.put_field("rawVersionString", version_obj)?;

    descriptor.put_field("modifiers", modifiers)?;

    descriptor.put_field("requires", requires)?;

    descriptor.put_field("exports", exports)?;

    descriptor.put_field("opens", opens)?;

    descriptor.put_field("uses", uses)?;

    descriptor.put_field("provides", provides)?;

    descriptor.put_field("packages", packages)?;

    descriptor.put_field("mainClass", main_class_obj)?;

    Ok(types::Type::ObjectRef(descriptor_ref))
  }

  pub(crate) fn alloc_new_native_tid(&mut self) -> i64 {
    let id = self.next_thread_id;

    self.next_thread_id += 1;

    id
  }

  pub(crate) fn get_current_thread_obj(&self) -> types::Type {
    self.current_thread
  }

  pub(crate) fn set_boot_loader_unnamed_module(&mut self, module_ref: ju4) {
    self.boot_loader_unnamed_module = Some(module_ref)
  }

  pub(crate) fn handle_step_error(
    &mut self,
    err: color_eyre::eyre::Report,
    stop_at: usize,
  ) -> Result<Option<types::Type>> {
    match err {
      _ if err.downcast_ref::<errors::InternalError>().is_some() => {
        let my_error = err.downcast_ref::<errors::InternalError>().unwrap();

        match my_error {
          errors::InternalError::FrameNotFound => {
            // done here
            Ok(None)
          }
          e => {
            self.show_frames();
            panic!("{}", e)
          }
        }
      }
      _ if err.downcast_ref::<errors::JavaException>().is_some() => {
        let exception = err.downcast_ref::<errors::JavaException>().unwrap();
        let exec_classname = errors::JavaException::convert_java_exception_to_classname(exception);
        // create new exception
        let exec_ref = self
          .heap
          .alloc_obj(&mut self.class_loader, exec_classname)?
          .as_ref()?;
        // and handle it
        let handled = self.handle_java_exception(exec_ref, stop_at)?;
        if handled.is_none() {
          Err(err)
        } else {
          Ok(handled)
        }
      }
      _ => {
        warn!("Exception not recognized");
        self.show_frames();
        panic!("{}", err)
      }
    }
  }

  pub fn run(&mut self) -> Result<()> {
    loop {
      match self.step() {
        Ok(v) => {
          if let Some(v) = v {
            debug!("NEW RET VALUE {}", v);
          }
        }
        Err(err) => {
          if self.handle_step_error(err, 0).is_err() {
            debug!("Something bad happened");
            break;
          }
        }
      }
    }

    Ok(())
  }

  pub(crate) fn push_stack(&mut self, value: types::Type) -> Result<()> {
    self.get_current_frame_mut()?.push_stack(value);
    if value.get_category() == 2 {
      self.get_current_frame_mut()?.push_stack(value);
    }

    Ok(())
  }

  pub(crate) fn pop_stack(&mut self) -> Result<types::Type> {
    let val = self.get_current_frame_mut()?.pop_stack()?;
    if val.get_category() == 2 {
      self.get_current_frame_mut()?.pop_stack()?;
    }
    Ok(val)
  }

  pub(crate) fn pop_string(&mut self) -> Result<String> {
    let string_ref = self.pop_object_ref()?;

    let field = self.heap.get_obj_instance(string_ref)?;

    assert!(field.is_string());

    let value_field = field.get_field("value")?;

    if let types::Type::ArrayRef(array_ref) = value_field {
      let array = self.heap.get_array_instance(array_ref)?;

      let field_name = array.get_string()?;

      return Ok(field_name);
    }

    Err(eyre!(errors::InternalError::General(
      "failed pop string from stack".to_string()
    )))
  }

  pub(crate) fn pop_class_name(&mut self) -> Result<String> {
    let class_ref = self.pop_object_ref()?;

    let class_name = self.heap.get_classname_from_class_obj(class_ref)?;

    Ok(class_name)
  }

  pub(crate) fn pop_array_ref(&mut self) -> Result<ju4> {
    let popped = self.get_current_frame_mut()?.pop_stack()?;

    if let types::Type::ArrayRef(obj_ref) = popped {
      return Ok(obj_ref);
    }

    Err(eyre!(errors::InternalError::General(
      "failed pop array ref from stack".to_string()
    )))
  }

  pub(crate) fn pop_object_ref(&mut self) -> Result<ju4> {
    let popped = self.get_current_frame_mut()?.pop_stack()?;

    if let types::Type::ObjectRef(obj_ref) = popped {
      return Ok(obj_ref);
    }

    Err(eyre!(errors::InternalError::General(
      "failed pop object ref from stack".to_string()
    )))
  }

  pub(crate) fn pop_ref(&mut self) -> Result<ju4> {
    let popped = self.get_current_frame_mut()?.pop_stack()?;

    let obj_ref = match popped {
      types::Type::ObjectRef(obj_ref) => obj_ref,
      types::Type::ArrayRef(array_ref) => array_ref,
      types::Type::Null => 0,
      _ => {
        return Err(eyre!(errors::InternalError::General(
          "failed pop object ref from stack".to_string()
        )))
      }
    };

    Ok(obj_ref)
  }

  pub(crate) fn jump_to(&mut self, pc: usize) -> Result<()> {
    let frame = self.get_current_frame_mut()?;
    frame.jump_to(pc);
    Ok(())
  }

  pub(crate) fn jump_by(&mut self, offset: i16) -> Result<()> {
    let frame = self.get_current_frame()?;

    if frame.can_jump_by(offset) {
      debug!("        [~] Jumping by {}", offset,);

      let frame = self.get_current_frame_mut()?;

      frame.jump_by(offset);
    } else {
      debug!(
        "        [~] NOT Jumping by {} from {}",
        offset,
        frame.get_pc()
      );

      debug!(
        "        [~] NOT Jumping by {} from {}",
        frame.get_code_length(),
        (frame.get_pc() as i32 + offset as i32) % frame.get_code_length() as i32
      );

      panic!();
    }

    Ok(())
  }

  // Short
  fn exec_sipush(&mut self) -> Result<Option<types::Type>> {
    let byte1 = self.get_current_frame_mut()?.read_ju1()?;

    let byte2 = self.get_current_frame_mut()?.read_ju1()?;

    let value = sign_extend16(((byte1 as u16) << 8) | (byte2 as u16)); // Convert to signed byte, then to i32
    self.push_stack(types::Type::Short(value as i16))?;

    Ok(None)
  }

  fn exec_saload(&mut self) -> Result<Option<types::Type>> {
    let index = self.pop_stack()?;

    let array_ref = self.pop_stack()?;

    match (array_ref, index) {
      (types::Type::ArrayRef(array_ref), types::Type::Integer(i)) => {
        let array = self.heap.get_array_instance(array_ref)?;

        let element = *array.get(i as usize)?;

        let value = match element {
          types::Type::Integer(int) => int as i16,
          types::Type::Short(int) => int,
          types::Type::Byte(int) => int as i16,
          types::Type::Character(int) => int as i16,
          types::Type::Boolean(int) => int as i16,
          v => return Err(eyre!(format!("element is not short {}", v))),
        };

        self.push_stack(types::Type::Short(value))?;

        Ok(None)
      }
      v => Err(eyre!(format!("ArrayRef not in the stack {:?}", v))),
    }
  }

  fn exec_sastore(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_ioperand()?;

    let index = self.pop_ioperand()?;

    let array_ref = self.pop_stack()?;

    let array_ref = match array_ref {
      types::Type::ArrayRef(array_ref) => array_ref,
      _ => {
        return Err(eyre!(format!(
          "ArrayRef not in the stack : {} {}",
          array_ref, index
        )))
      }
    };

    let array = self.heap.get_array_instance_mut(array_ref)?;

    if array.get_classname() != "[S" {
      return Err(eyre!(format!("Input should be of type short {}", array)));
    }

    array.set(index as usize, types::Type::Short(value as i16))?;

    Ok(None)
  }

  // Byte
  fn exec_baload(&mut self) -> Result<Option<types::Type>> {
    let index = self.pop_ioperand()?;

    let array_ref = self.pop_array_ref()?;

    let array = self.heap.get_array_instance(array_ref)?;

    let element = *array.get(index as usize)?;

    let value = element.as_byte()?;

    self.push_stack(types::Type::Byte(value))?;

    Ok(None)
  }

  fn exec_bastore(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_ioperand()?;

    let index = self.pop_ioperand()?;

    let array_ref = self.pop_stack()?;

    let array_ref = match array_ref {
      types::Type::ArrayRef(array_ref) => array_ref,
      _ => {
        return Err(eyre!(format!(
          "ArrayRef not in the stack : {} {}",
          array_ref, index
        )))
      }
    };

    let array = self.heap.get_array_instance_mut(array_ref)?;

    if array.get_classname() != "[B" {
      return Err(eyre!(format!(
        "Input should be of type java/lang/Byte {}",
        array
      )));
    }

    array.set(index as usize, types::Type::Byte(value as i8))?;

    Ok(None)
  }

  // Char - Exec
  fn exec_caload(&mut self) -> Result<Option<types::Type>> {
    let index = self.pop_ioperand()?;

    let array_ref = self.pop_array_ref()?;

    let array = self.heap.get_array_instance(array_ref)?;

    let element = *array.get(index as usize)?;

    let value = match element {
      types::Type::Integer(int) => int as i8,
      types::Type::Short(int) => int as i8,
      types::Type::Byte(int) => int,
      types::Type::Character(int) => int,
      types::Type::Boolean(int) => int as i8,
      v => return Err(eyre!(format!("element is not character {}", v))),
    };

    self.push_stack(types::Type::Character(value))?;

    Ok(None)
  }

  fn exec_aaload(&mut self) -> Result<Option<types::Type>> {
    let index = self.pop_ioperand()?;

    let array_ref = self.pop_array_ref()?;

    let array = self.heap.get_array_instance(array_ref)?;

    debug!("        [~] AALOAD {} at {} -> {}", array_ref, index, array);

    let value = array.get(index as usize)?;

    if let types::Type::ObjectRef(obj_ref) = value {
      let obj = self.heap.get_obj_instance(*obj_ref)?;

      debug!("        [~] AALOAD {}", obj);
    }

    self.push_stack(*array.get(index as usize)?)?;

    Ok(None)
  }

  fn exec_aastore(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?;

    let index = self.pop_ioperand()? as usize;

    let array_ref = self.pop_stack()?;

    let array = match array_ref {
      types::Type::ArrayRef(array_ref) => self.heap.get_array_instance(array_ref)?,
      _ => return Err(eyre!(errors::JavaException::NullPointer)),
    };

    debug!(
      "        [~] AASTORE setting {} in {} {} at {}",
      value,
      array_ref,
      array.get_classname(),
      index
    );

    // array element class name
    let expected_class_name = array.get_element_classname();

    let value = match value {
      types::Type::Null => value,
      types::Type::ArrayRef(obj_ref) | types::Type::ObjectRef(obj_ref) => {
        let obj = self.heap.get_instance(obj_ref)?;

        debug!(
          "        [~] AASTORE {} <: {}",
          expected_class_name,
          obj.get_classname()
        );

        debug!(
          "        [~] AASTORE array is  {} -> {}",
          array, expected_class_name
        );
        debug!(
          "        [~] AASTORE value is  {} -> {}",
          obj,
          obj.get_classname()
        );

        // check if `arr[index] = value` is legal
        if types::Type::check_type(
          &mut self.class_loader,
          expected_class_name,
          obj.get_classname(),
        )? {
          value
        } else {
          return Err(eyre!(errors::InternalError::WrongClass(
            expected_class_name.to_owned(),
            obj.get_classname().to_owned()
          )));
        }
      }
      _ => {
        return Err(eyre!(errors::InternalError::WrongType(
          "ArrayRef/Null",
          value
        )))
      }
    };

    let array = match array_ref {
      types::Type::ArrayRef(array_ref) => self.heap.get_array_instance_mut(array_ref)?,
      _ => return Err(eyre!(errors::JavaException::NullPointer)),
    };

    array.set(index, value)?;

    Ok(None)
  }

  // https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-6.html#jvms-6.5.ldc
  fn exec_ldc(&mut self) -> Result<Option<types::Type>> {
    let value: types::Type = {
      let info = {
        let index = self.get_current_frame_mut()?.read_ju1()? as ju2;

        let current_class = self.get_current_class()?;

        let item = current_class.resolve_index(index)?;

        item.get_info().clone()
      };

      match info {
        constant_pool::CpInfoInfoEnum::Integer(int) => types::Type::Integer(int.int()),
        constant_pool::CpInfoInfoEnum::Float(fl) => types::Type::Float(fl.float()),
        constant_pool::CpInfoInfoEnum::String(st) => {
          let string = self
            .get_current_class()?
            .resolve_name(st.get_string_index())?;

          self.heap.alloc_string(&mut self.class_loader, &string)?
        }
        constant_pool::CpInfoInfoEnum::Long(_) | constant_pool::CpInfoInfoEnum::Double(_) => {
          return Err(eyre!("long/double not allowed in LDC"))
        }
        constant_pool::CpInfoInfoEnum::Class(cl) => {
          let class_name = self
            .get_current_class()?
            .resolve_name(cl.get_name_index())?;

          self
            .heap
            .alloc_class_obj(&mut self.class_loader, &class_name)?
        }
        _ => notimpl!(format!("ldc not implemented: {}", info)),
      }
    };

    self.push_stack(value)?;

    Ok(None)
  }

  fn exec_ldcw(&mut self) -> Result<Option<types::Type>> {
    let value: types::Type = {
      let info = {
        // wide index
        let index = self.get_current_frame_mut()?.read_ju2()?;

        let current_class = self.get_current_class()?;

        let item = current_class.resolve_index(index)?;

        item.get_info().clone()
      };

      match info {
        constant_pool::CpInfoInfoEnum::Integer(int) => types::Type::Integer(int.int()),
        constant_pool::CpInfoInfoEnum::Float(fl) => types::Type::Float(fl.float()),
        constant_pool::CpInfoInfoEnum::String(st) => {
          let string = self
            .get_current_class()?
            .resolve_name(st.get_string_index())?;

          self.heap.alloc_string(&mut self.class_loader, &string)?
        }
        constant_pool::CpInfoInfoEnum::Long(_) | constant_pool::CpInfoInfoEnum::Double(_) => {
          return Err(eyre!("long/double not implemented in LDCW"))
        }
        constant_pool::CpInfoInfoEnum::Class(cl) => {
          let class_name = self
            .get_current_class()?
            .resolve_name(cl.get_name_index())?;

          self
            .heap
            .alloc_class_obj(&mut self.class_loader, &class_name)?
        }
        _ => notimpl!(format!("ldc not implemented: {}", info)),
      }
    };

    self.push_stack(value)?;

    Ok(None)
  }

  fn exec_getstatic(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let (class_name, field_name, _field_type) = self
      .class_loader
      .resolve_field_ref(&curr_class_name, index)?;

    self.init_class(&class_name)?;

    // TODO: check it is static and field type
    let field = self
      .class_loader
      .get_static_field(&class_name, &field_name)?;

    if let types::Type::ObjectRef(obj_ref) = field {
      let obj = self.heap.get_obj_instance(obj_ref)?;

      debug!("        [~] GETSTATIC {} : {}", field_name, obj);
    }

    self.push_stack(field)?;

    Ok(None)
  }

  fn exec_putstatic(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let (class_name, field_name, field_type) = self
      .class_loader
      .resolve_field_ref(&curr_class_name, index)?;

    self.init_class(&class_name)?;

    // TODO: check it is static and field type
    let mut value = self.pop_stack()?;

    debug!(
      "Put static in {} {} {} = {}",
      class_name, field_name, field_type, value
    );

    // Custom fields
    let name: &str = &class_name;

    let fname: &str = &field_name;

    if name == "jdk/internal/misc/UnsafeConstants" {
      match fname {
        // 64-bit
        "ADDRESS_SIZE0" => value = types::Type::Integer(8),
        "PAGE_SIZE" => value = types::Type::Integer(4096),
        "BIG_ENDIAN" => value = types::Type::Boolean(false),
        _ => {}
      }
    }

    self
      .class_loader
      .get_mut(&class_name)?
      .borrow_mut()
      .put_static_field(&field_name, value)?;

    //self.push_stack(&name as u32);
    Ok(None)
  }

  // https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-6.html#jvms-6.5.invokestatic
  fn exec_invokestatic(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let (class_name, method_name, method_type) = self
      .class_loader
      .resolve_method_ref(&curr_class_name, index)?;

    if !self.heap.has_class_instance(&class_name) {
      self
        .heap
        .alloc_class_obj(&mut self.class_loader, &class_name)?;

      self.init_class(&class_name)?;
    }

    // TODO: skip for now -> handle later
    if self
      .class_loader
      .is_method_native(&class_name, &method_name, &method_type)?
    {
      debug!("[!] {} is native static in {}", method_name, class_name);

      self.call_native(&class_name, &method_name, &method_type)?;

      return Ok(None);
    }

    debug!(
      "        [~] InvokeStatic {} {} {}",
      class_name, method_name, method_type
    );

    // TODO: check it is static
    let arg_count = JVM::parse_argument_count(&method_type)?;

    // If the method is not native, the nargs argument values are popped from the operand stack.
    let mut args = self.pop_arguments(arg_count, &class_name, false)?;

    let (method_class, method) =
      self
        .class_loader
        .get_method_by_name(&class_name, &method_name, &method_type)?;

    let max_locals = method
      .get_code()
      .ok_or_eyre(eyre!(errors::InternalError::CodeNotFound(
        class_name,
        method_name.clone(),
        method_type.clone()
      )))?
      .get_max_locals();

    for _ in 0..(max_locals) {
      args.push(types::Type::None);
    }

    debug!(
      "        [~] InvokeStatic {} {} {} {:?} ({})",
      method_class, method_name, method_type, args, arg_count
    );

    self.push_frame_from_class(&method_class, &method_name, &method_type, args)?;

    Ok(None)
  }

  // General
  fn exec_pop2(&mut self) -> Result<Option<types::Type>> {
    let value1 = self.pop_stack()?;

    if value1.get_category() == 1 {
      let value2 = self.pop_stack()?;

      assert!(value2.get_category() == 1);
    }

    Ok(None)
  }

  fn exec_new(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let class_name = self.get_current_class()?.resolve_class_name(index)?;

    let obj = self.heap.alloc_obj(&mut self.class_loader, &class_name)?;

    debug!("        [~] New {}", class_name);

    self.push_stack(obj)?;

    Ok(None)
  }

  fn exec_newarray(&mut self) -> Result<Option<types::Type>> {
    let atype = self.get_current_frame_mut()?.read_ju1()? as ju2;

    let size = self.pop_ioperand()?;

    let class_name = match atype {
      4 => "Z",
      5 => "C",
      6 => "F",
      7 => "D",
      8 => "B",
      9 => "S",
      10 => "I",
      11 => "J",
      _ => return Err(eyre!(format!("Array type not recognized {}", atype))),
    };

    let array_ref = self
      .heap
      .alloc_array_primitive(class_name, vec![], size as usize)?;

    self.push_stack(array_ref)?;

    Ok(None)
  }

  fn exec_dup(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_stack()?;

    self.push_stack(value)?;

    self.push_stack(value)?;

    Ok(None)
  }

  fn exec_dupx1(&mut self) -> Result<Option<types::Type>> {
    let value1 = self.pop_stack()?;

    let value2 = self.pop_stack()?;

    assert!(value1.get_category() == 1 && value2.get_category() == 1);

    self.push_stack(value1)?;

    self.push_stack(value2)?;

    self.push_stack(value1)?;

    Ok(None)
  }

  fn exec_dupx2(&mut self) -> Result<Option<types::Type>> {
    let value1 = self.pop_stack()?;

    let value2 = self.pop_stack()?;

    if value1.get_category() == 1 && value2.get_category() == 2 {
      self.push_stack(value1)?;

      self.push_stack(value2)?;

      self.push_stack(value1)?;
    } else if value1.get_category() == 1 && value2.get_category() == 1 {
      let value3 = self.pop_stack()?;

      assert!(value3.get_category() == 1);

      self.push_stack(value1)?;

      self.push_stack(value3)?;

      self.push_stack(value2)?;

      self.push_stack(value1)?;
    }

    Ok(None)
  }

  fn exec_dup2(&mut self) -> Result<Option<types::Type>> {
    let value1 = self.pop_stack()?;

    if value1.get_category() == 2 {
      self.push_stack(value1)?;
      self.push_stack(value1)?;
    } else if value1.get_category() == 1 {
      let value2 = self.pop_stack()?;
      assert!(value2.get_category() == 1);

      self.push_stack(value2)?;
      self.push_stack(value1)?;
      self.push_stack(value2)?;
      self.push_stack(value1)?;
    }

    Ok(None)
  }

  fn exec_bipush(&mut self) -> Result<Option<types::Type>> {
    let byte = self.get_current_frame_mut()?.read_ju1()?;

    let value = sign_extend8(byte); // Convert to signed byte, then to i32
    self.push_stack(types::Type::Integer(value))?;

    Ok(None)
  }

  fn exec_ifnull(&mut self) -> Result<Option<types::Type>> {
    let offset = self.get_current_frame_mut()?.read_ju2()? as i16;

    let value = self.pop_stack()?;

    if value == types::Type::Null {
      self.jump_by(offset)?;
    }

    Ok(None)
  }

  fn exec_ifnonnull(&mut self) -> Result<Option<types::Type>> {
    let offset = self.get_current_frame_mut()?.read_ju2()? as i16;

    let value = self.pop_stack()?;

    match value {
      types::Type::Null => {}
      _ => {
        self.jump_by(offset)?;
      }
    };

    Ok(None)
  }

  fn exec_goto(&mut self) -> Result<Option<types::Type>> {
    let offset = self.get_current_frame_mut()?.read_ju2()? as i16;

    self.jump_by(offset)?;

    Ok(None)
  }

  pub(crate) fn parse_argument_count(descriptor: &str) -> Result<usize> {
    let mut count = 0;

    let mut chars = descriptor.chars();

    if chars.next() != Some('(') {
      return Err(eyre!("Invalid descriptor"));
    }

    let mut in_class = false;

    while let Some(ch) = chars.next() {
      match ch {
        ')' => break,
        'L' => {
          in_class = true;

          for c in chars.by_ref() {
            if c == ';' {
              in_class = false;

              break;
            }
          }

          count += 1;
        }
        '[' => continue, // skip array marker
        'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' => count += 1,
        _ if in_class => continue,
        _ => return Err(eyre!("Unexpected char in descriptor: {}", ch)),
      }
    }

    Ok(count)
  }

  fn pop_arguments(
    &mut self,
    arg_count: usize,
    class_name: &str,
    not_static: bool,
  ) -> Result<Vec<types::Type>> {
    let mut args = Vec::with_capacity(arg_count + (not_static as usize));

    for _ in 0..arg_count {
      // TODO: check argument type
      let arg = self.pop_stack()?;

      match arg {
        // two of them becase we use 32 bits and long/double are 64 bits
        types::Type::Long(_) | types::Type::Double(_) => {
          args.push(arg);
        }
        types::Type::ArrayRef(array_ref) => {
          let array = self.heap.get_array_instance(array_ref)?;
          debug!("Argument {} {}", array, array_ref);
        }
        types::Type::ObjectRef(obj_ref) => {
          let obj = self.heap.get_obj_instance(obj_ref)?;

          if obj.get_classname() == "java/lang/String" {
            let array_field = obj.get_field("value")?;

            if let types::Type::ArrayRef(array_ref) = array_field {
              let array = self.heap.get_array_instance(array_ref)?;

              //// this is the name of T class in Class<T>
              let name = array.get_string()?;

              debug!("Argument {} {} {}", obj_ref, name, array);
            } else {
              debug!("Argument {} {}", obj_ref, obj);
            }
          } else if obj.get_classname() == "java/lang/Class" {
            match self.heap.get_classname_from_class_obj(obj.get_ref()) {
              Ok(v) => {
                debug!("Argument class {} {}", obj_ref, v);
              }
              _ => {
                debug!("Argument {}", obj)
              }
            }
          } else {
            debug!("Argument {}", obj)
          }
        }
        _ => {
          debug!("Argument {}", arg)
        }
      }

      args.push(arg);
    }

    if not_static {
      match self.pop_stack()? {
        a @ (types::Type::ObjectRef(obj_ref) | types::Type::ArrayRef(obj_ref)) => {
          let obj = self.heap.get_instance(obj_ref)?;

          debug!("This : {}", obj);

          if !types::Type::check_type(
            &mut self.class_loader,
            class_name,
            obj.get_class_field_type(),
          )? {
            return Err(eyre!(errors::InternalError::WrongClass(
              class_name.to_string(),
              obj.get_class_field_type().to_string()
            )));
          }

          args.push(a);
        }
        types::Type::Null => return Err(eyre!(errors::JavaException::NullPointer)),
        v => return Err(eyre!(errors::InternalError::WrongType("ObjectRef", v))),
      };
    }

    // convert stack to local order
    args.reverse();

    Ok(args)
  }

  fn exec_invokevirtual(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let (class_name, method_name, method_type) = self
      .class_loader
      .resolve_method_ref(&curr_class_name, index)?;

    // TODO: skip for now -> handle later
    debug!(
      "        [~] InvokeVirtual {} {} {} from {}",
      class_name, method_name, method_type, curr_class_name
    );

    let arg_count = JVM::parse_argument_count(&method_type)?;

    let mut args = self.pop_arguments(arg_count, &class_name, true)?;

    let caller = args
      .first()
      .ok_or_eyre(eyre!(errors::InternalError::General(
        "Caller not found in InvokeVirtual".to_string()
      )))?;

    let caller_class = match caller {
      types::Type::ObjectRef(obj_ref) | types::Type::ArrayRef(obj_ref) => self
        .heap
        .get_instance(*obj_ref)?
        .get_classname()
        .to_string(),
      _ => panic!("not ref {}", caller),
    };

    debug!(
      "        [~] InvokeVirtual {} {} {} {}",
      caller_class, method_name, method_type, arg_count
    );

    let (method_class, method) =
      self
        .class_loader
        .get_method_by_name(&caller_class, &method_name, &method_type)?;

    if method.is_native() {
      debug!(
        "[!] {} {} is native virtual in {}",
        method_name, method_type, method_class
      );

      // restore stack
      self.restore_stack(args)?;

      self.call_native(&method_class, &method_name, &method_type)?;

      return Ok(None);
    }

    let max_locals = method
      .get_code()
      .ok_or_eyre(eyre!(errors::InternalError::CodeNotFound(
        class_name,
        method_name.clone(),
        method_type.clone()
      )))?
      .get_max_locals();

    for _ in 0..(max_locals - arg_count as u16) {
      args.push(types::Type::None);
    }

    self.push_frame_from_class(&method_class, &method_name, &method_type, args)?;

    Ok(None)
  }

  // https://docs.oracle.com/javase/specs/jvms/se17/html/jvms-4.html#jvms-4.10.1.9.invokespecial
  // compile time
  fn exec_invokespecial(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let (class_name, method_name, method_type) = self
      .class_loader
      .resolve_method_ref(&curr_class_name, index)?;

    if !self.heap.has_class_instance(&class_name) {
      self
        .heap
        .alloc_class_obj(&mut self.class_loader, &class_name)?;
    }

    // TODO: skip for now -> handle later
    if self
      .class_loader
      .is_method_native(&class_name, &method_name, &method_type)?
    {
      debug!("[!] {} is native in {}", method_name, class_name);

      self.call_native(&class_name, &method_name, &method_type)?;

      return Ok(None);
    }

    let max_locals =
      self
        .class_loader
        .get_method_max_locals_by_name(&class_name, &method_name, &method_type)?;

    let arg_count = JVM::parse_argument_count(&method_type)?;

    let mut args = self.pop_arguments(arg_count, &class_name, true)?;

    // TODO: set default value per type ?
    for _ in 0..max_locals {
      args.push(types::Type::None);
    }

    debug!(
      "        [~] InvokeSpecial {} {} {} {:?}",
      class_name, method_name, method_type, args
    );

    self.push_frame_from_class(&class_name, &method_name, &method_type, args)?;

    Ok(None)
  }

  fn exec_invokeinterface(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    // The redundancy is historical -> not needed
    let _count = self.get_current_frame_mut()?.read_ju1()?;

    // The value of the fourth operand byte must always be zero.
    let _zero = self.get_current_frame_mut()?.read_ju1()?;

    assert!(_zero == 0);

    let curr_class_name = self.get_current_class()?.get_name().to_owned();
    debug!("{} {}", curr_class_name, index);

    let (class_name, method_name, method_type) = self
      .class_loader
      .resolve_method_ref(&curr_class_name, index)?;

    debug!(
      "INVOKEINTERFACE {} {} {}",
      class_name, method_name, method_type
    );

    if !self.heap.has_class_instance(&class_name) {
      self
        .heap
        .alloc_class_obj(&mut self.class_loader, &class_name)?;
    }

    if method_name == "synchronized" {
      panic!("Not handled synchronized yet")
    }

    // TODO:  if the resolved method is static, the invokeinterface instruction throws an
    // IncompatibleClassChangeError.
    let arg_count = JVM::parse_argument_count(&method_type)?;

    // The objectref must be of type reference and must be followed on the operand stack by
    // nargs argument values, where the number, type, and order of the values must be consistent
    // with the descriptor of the resolved interface method.
    let mut args = self.pop_arguments(arg_count, &class_name, true)?;

    let caller = args
      .first()
      .ok_or_eyre(eyre!(errors::InternalError::General(
        "Caller not found in InvokeInterface".to_string()
      )))?;

    let caller_class = if let types::Type::ObjectRef(caller_ref) = caller {
      self
        .heap
        .get_obj_instance(*caller_ref)?
        .get_classname()
        .to_owned()
    } else {
      panic!("not ref")
    };

    // function of interface might have been defined using the "default" keywork
    // => no implementation found in the caller but in the interface class
    let (method_class, method) =
      self
        .class_loader
        .get_method_by_name(&caller_class, &method_name, &method_type)?;

    if method.is_native() {
      debug!(
        "[!] {} {} is native virtual in {}",
        method_name, method_type, method_class
      );

      // restore stack
      self.restore_stack(args)?;

      self.call_native(&method_class, &method_name, &method_type)?;

      return Ok(None);
    }

    debug!(
      "        [~] InvokeInterface {} {} {} {} {:?}",
      method_class, class_name, method_name, method_type, args
    );

    let max_locals = method
      .get_code()
      .ok_or_eyre(eyre!(errors::InternalError::CodeNotFound(
        class_name,
        method_name.clone(),
        method_type.clone()
      )))?
      .get_max_locals();

    for _ in 0..std::cmp::max(max_locals, 4) {
      args.push(types::Type::None);
    }

    self.push_frame_from_class(&caller_class, &method_name, &method_type, args)?;

    Ok(None)
  }

  fn exec_getfield(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let (class_name, field_name, field_type) = self
      .class_loader
      .resolve_field_ref(&curr_class_name, index)?;

    // TODO: check the type
    //let field_name = self.classes.get(&class_name)?.resolve_name(field_index)?;
    //let field_name = self.get_current_class()?.resolve_name(field_index)?;
    let obj = self.pop_stack()?;

    debug!(
      "        [~] Get field {} {} {} {}",
      class_name, field_name, field_type, obj
    );

    let value = match obj {
      types::Type::ObjectRef(obj_ref) => self
        .heap
        .get_obj_instance(obj_ref)?
        .get_field(&field_name)?,
      types::Type::Null => return Err(eyre!(errors::JavaException::NullPointer)),
      _ => return Err(eyre!("not object ref: {}", obj)),
    };

    debug!(
      "        [~] Get field {} {} {} {}",
      class_name, field_name, field_type, value
    );

    self.push_stack(value)?;

    Ok(None)
  }

  fn exec_putfield(&mut self) -> Result<Option<types::Type>> {
    let index = self.get_current_frame_mut()?.read_ju2()?;

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let (class_name, field_name, field_type) = self
      .class_loader
      .resolve_field_ref(&curr_class_name, index)?;

    let value = self.pop_stack()?;

    let objref = self.pop_stack()?;

    debug!(
      "        [~] Put field {} {} in {}",
      field_name, field_type, class_name
    );

    // objrect.field = value
    match objref {
      types::Type::ObjectRef(reference) => {
        let value_type = value.get_type(&self.heap);

        let objinstance = self.heap.get_obj_instance_mut(reference)?;

        if types::Type::check_type(&mut self.class_loader, &field_type, &value_type)? {
          objinstance.put_field(&field_name, value)?;
        } else {
          return Err(eyre!(
            "Putfield wrong type in {} : {} <!: {}",
            objinstance.get_classname(),
            value_type,
            field_type
          ));
        }
      }
      _ => return Err(eyre!("Not a Object Reference: {}", objref)),
    }

    Ok(None)
  }

  fn exec_checkcast(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_stack()?;

    let index = self.get_current_frame_mut()?.read_ju2()?;

    let obj_class_name = match obj_ref {
      types::Type::Null => {
        // If objectref is null, then the operand stack is unchanged.
        self.push_stack(obj_ref)?;

        return Ok(None);
      }
      types::Type::ObjectRef(obj_ref) | types::Type::ArrayRef(obj_ref) => self
        .heap
        .get_instance(obj_ref)?
        .get_class_field_type()
        .to_string(),
      _ => return Err(eyre!("Not a Reference: {}", obj_ref)),
    };

    debug!("{} {}", obj_ref, obj_class_name);

    let info = {
      let class = self.get_current_class()?;

      let item = class.resolve_index(index)?;

      item.get_info().clone()
    };

    match info {
      CpInfoInfoEnum::Class(class_info) => {
        let class_name = self
          .get_current_class()?
          .resolve_name(class_info.get_name_index())?;

        //  If objectref is a value of the type given by the resolved class, array, or
        // interface type, the operand stack is unchanged.
        if types::Type::check_type(&mut self.class_loader, &class_name, &obj_class_name)? {
          self.push_stack(obj_ref)?;

          return Ok(None);
        }

        debug!("{} {}", class_name, obj_class_name);
      }
      CpInfoInfoEnum::Interfaceref(_) => {
        notimpl!("Interface in CHECKCAST")
      }
      _ => return Err(eyre!("Not a Object Reference: {}", obj_class_name)),
    };

    notimpl!()
  }

  pub(crate) fn exec_instanceof(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_stack()?;

    let index = self.get_current_frame_mut()?.read_ju2()?;

    let obj_ref = match obj_ref {
      types::Type::Null => {
        // If objectref is null, the instanceof instruction pushes an int result of 0 onto
        // the operand stack.
        self.push_stack(types::Type::Integer(0))?;

        return Ok(None);
      }
      types::Type::ObjectRef(obj_ref) | types::Type::ArrayRef(obj_ref) => obj_ref,
      _ => return Err(eyre!("Not a Object Reference: {}", obj_ref)),
    };

    // resolve like in checkcast
    let topush = {
      let obj_class_name = self.heap.get_instance(obj_ref)?.get_classname().to_string();

      let class = self.get_current_class()?;

      let item = class.resolve_index(index)?;

      match &item.get_info() {
        CpInfoInfoEnum::Class(cl_index) => {
          let class_name = class.resolve_name(cl_index.get_name_index())?;

          drop(class);

          debug!("        INSTANCEOF {} {}", class_name, obj_class_name);

          if types::Type::check_type(&mut self.class_loader, &class_name, &obj_class_name)? {
            types::Type::Integer(1)
          } else {
            types::Type::Integer(0)
          }
        }
        CpInfoInfoEnum::Interfaceref(_) => {
          notimpl!()
        }
        _ => return Err(eyre!("Not a Object Reference: {}", obj_ref)),
      }
    };

    self.push_stack(topush)?;

    Ok(None)
  }

  fn exec_monitorenter(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_stack()?;

    let obj_ref = match obj_ref {
      types::Type::ObjectRef(obj_ref) => obj_ref,
      types::Type::Null => return Err(eyre!(errors::JavaException::NullPointer)),
      _ => return Err(eyre!("Not a Object Reference: {}", obj_ref)),
    };

    let obj = self.heap.get_obj_instance_mut(obj_ref)?;

    obj.monitorenter();

    Ok(None)
  }

  fn exec_monitorexit(&mut self) -> Result<Option<types::Type>> {
    let obj_ref = self.pop_stack()?;

    let obj_ref = match obj_ref {
      types::Type::ObjectRef(obj_ref) => obj_ref,
      types::Type::Null => return Err(eyre!(errors::JavaException::NullPointer)),
      _ => return Err(eyre!("Not a Object Reference: {}", obj_ref)),
    };

    let obj = self.heap.get_obj_instance_mut(obj_ref)?;

    obj.monitorexit();

    Ok(None)
  }

  fn handle_java_exception(
    &mut self,
    exec_ref: ju4,
    stop_at: usize,
  ) -> Result<Option<types::Type>> {
    let class_name = self
      .heap
      .get_obj_instance(exec_ref)?
      .get_classname()
      .to_string();
    while self.frames.len() >= stop_at {
      let method_name = self.get_current_frame()?.get_method_name().to_string();
      let method_type = self.get_current_frame()?.get_method_type().to_string();
      let method_class = self.get_current_frame()?.get_classname().to_string();

      let (_, method) =
        self
          .class_loader
          .get_method_by_name(&method_class, &method_name, &method_type)?;

      if let Some(method_code) = method.get_code() {
        let pc = self.get_current_frame()?.get_pc() as ju2;
        let check = method_code.check_exception(&mut self.class_loader, pc, &class_name)?;
        if let Some(handler_pc) = check {
          self.push_stack(types::Type::ObjectRef(exec_ref))?;
          // jump to handler pc
          self.jump_to(handler_pc as usize)?;
          debug!(
            "Exception Handler at {} {} {}:{}",
            method_class, method_name, method_type, handler_pc
          );
          return Ok(Some(types::Type::ObjectRef(exec_ref)));
        }
      };

      debug!(
        "Poping frame {} {} {}",
        method_class, method_name, method_type
      );
      self.pop_frame();
    }

    debug!("Exception not handled yet {}", class_name);

    // not handled
    Ok(None)
  }

  fn exec_athrow(&mut self) -> Result<Option<types::Type>> {
    warn!("ATHROW NOT IMPLEMENTED YET");

    let exec_ref = self.pop_stack()?.as_ref()?;
    let exec_obj = self.heap.get_obj_instance(exec_ref)?;
    let exec_classname = exec_obj.get_classname();

    let detail_message_ref = exec_obj.get_field("detailMessage")?.as_ref()?;
    let detail_message = if detail_message_ref == 0 {
      "No Message".to_string()
    } else {
      self.heap.get_string(detail_message_ref)?
    };
    debug!("THROWING {} -> '{}'", exec_classname, detail_message);

    self.show_frames();

    // make this propagate until main loop

    Err(eyre!(
      errors::JavaException::convert_classname_to_java_exception(exec_classname, detail_message)
    ))
  }

  // Access jump table by index and jump
  fn exec_tableswitch(&mut self) -> Result<Option<types::Type>> {
    let curr_pc = self.get_current_frame()?.get_pc();

    let padding = (4 - (curr_pc % 4)) % 4;

    // 0 to 3 byte pad
    for _ in 0..padding {
      self.get_current_frame_mut()?.read_ju1()?;
    }

    let default = self.get_current_frame_mut()?.read_ju4()? as i32;

    let low = self.get_current_frame_mut()?.read_ju4()? as i32;

    let high = self.get_current_frame_mut()?.read_ju4()? as i32;

    let index = self.pop_ioperand()?;

    if low >= high {
      return Err(eyre!(errors::InternalError::General(
        "low should be less than high in tableswitch".to_string()
      )));
    }

    debug!(
      "        [~] TABLESWITCH P({}) D({}) L({}) H({}) I({})",
      padding, default, low, high, index
    );

    let mut jumptable = vec![];

    // the high - low + 1 signed 32-bit offsets are treated as a 0-based jump table
    for _ in 0..(high - low + 1) {
      let jump = self.get_current_frame_mut()?.read_ju4()? as i32;

      jumptable.push(jump);
    }

    let offset = if index < low || index > high {
      // then a target address is calculated by adding default to the address of the opcode of
      // this tableswitch instruction
      default
    } else {
      let index = (index - low) as usize;

      // the offset at position index - low of the jump table is extracted
      // the target address is calculated by adding that offset to the address of the opcode
      // of this tableswitch instruction
      if index >= jumptable.len() {
        return Err(eyre!(errors::JavaException::ArrayIndexOutOfBounds(
          index,
          jumptable.len()
        )));
      }

      jumptable[index]
    };

    self.jump_by(offset as i16)?;

    Ok(None)
  }

  fn exec_lookupswitch(&mut self) -> Result<Option<types::Type>> {
    let curr_pc = self.get_current_frame()?.get_pc();

    let padding = (4 - (curr_pc % 4)) % 4;

    // 0 to 3 byte pad
    for _ in 0..padding {
      self.get_current_frame_mut()?.read_ju1()?;
    }

    let default = self.get_current_frame_mut()?.read_ju4()? as i32;

    let npairs = self.get_current_frame_mut()?.read_ju4()? as i32;

    let key = self.pop_ioperand()?;

    if npairs < 0 {
      return Err(eyre!(errors::InternalError::General(
        "npairs must be >= 0".to_string()
      )));
    }

    debug!(
      "        [~] LOOKUPSWITCH P({}) D({}) P({}) K({})",
      padding, default, npairs, key
    );

    // then high - low + 1 signed 32-bits offsets
    let mut jumptable = vec![];

    for _ in 0..(npairs) {
      let matc = self.get_current_frame_mut()?.read_ju4()? as i32;

      let offset = self.get_current_frame_mut()?.read_ju4()? as i32;

      jumptable.push((matc, offset));
    }

    jumptable.sort_by_key(|pair| pair.0);

    let offset = 'found: {
      for (m, o) in jumptable {
        if key == m {
          break 'found o;
        }
      }

      default
    };

    self.jump_by(offset as i16)?;

    Ok(None)
  }

  // Array
  fn exec_anewarray(&mut self) -> Result<Option<types::Type>> {
    let info = {
      let index = self.get_current_frame_mut()?.read_ju2()? as ju2;

      let current_class = self.get_current_class()?;

      let item = current_class.resolve_index(index)?;

      item.get_info().clone()
    };

    let size = self.pop_ioperand()?;

    let class_name = match info {
      CpInfoInfoEnum::Class(cl) => self
        .get_current_class()?
        .resolve_name(cl.get_name_index())?
        .clone(),
      CpInfoInfoEnum::Interfaceref(_) => {
        notimpl!()
      }
      _ => {
        return Err(eyre!(format!(
          "Array elements must be either Classes or Interfaces: {}",
          info
        )))
      }
    };

    let array_ref = self.heap.alloc_array(&class_name, vec![], size as usize)?;

    self.push_stack(array_ref)?;

    Ok(None)
  }

  fn exec_arraylength(&mut self) -> Result<Option<types::Type>> {
    let array_ref = self.pop_array_ref()?;

    let array = self.heap.get_array_instance(array_ref)?;

    debug!("        [~] ARRAYLENGTH {} {}", array_ref, array.len());

    self.push_stack(types::Type::Integer(array.len() as i32))?;
    Ok(None)
  }

  fn exec_castore(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_ioperand()?;

    let index = self.pop_ioperand()?;

    let array_ref = self.pop_stack()?;

    let array_ref = match array_ref {
      types::Type::ArrayRef(array_ref) => array_ref,
      _ => {
        return Err(eyre!(format!(
          "ArrayRef not in the stack : {} {}",
          array_ref, index
        )))
      }
    };

    let array = self.heap.get_array_instance_mut(array_ref)?;

    if array.get_classname() != "[C" {
      return Err(eyre!(format!(
        "Input should be of type character {}",
        array
      )));
    }

    array.set(index as usize, types::Type::Character(value as i8))?;

    Ok(None)
  }

  fn exec_multianewarray(&mut self) -> Result<Option<types::Type>> {
    let info = {
      let index = self.get_current_frame_mut()?.read_ju2()?;

      let current_class = self.get_current_class()?;

      let item = current_class.resolve_index(index)?;

      item.get_info().clone()
    };

    let num_dimensions = self.get_current_frame_mut()?.read_ju1()?;

    let class_name = match info {
      CpInfoInfoEnum::Class(cl) => self
        .get_current_class()?
        .resolve_name(cl.get_name_index())?
        .clone(),
      CpInfoInfoEnum::Interfaceref(_) => {
        notimpl!()
      }
      _ => {
        return Err(eyre!(format!(
          "Array elements must be either Classes or Interfaces: {}",
          info
        )))
      }
    };

    let mut dimensions = Vec::new();

    for _ in 0..num_dimensions {
      let dim = self.pop_ioperand()?;

      if dim < 0 {
        return Err(eyre!("NegativeArraySizeException"));
      }

      dimensions.push(dim as usize);
    }

    dimensions.reverse();

    let array_ref = self.heap.alloc_multiarray(&class_name, &dimensions)?;

    self.push_stack(array_ref)?;

    Ok(None)
  }

  pub(crate) fn get_current_class(&mut self) -> Result<RwLockReadGuard<class_file::ClassFile>> {
    let current_class = self.get_current_frame()?.get_classname().to_string();

    let class = self.class_loader.get(&current_class)?;

    Ok(class)
  }

  pub(crate) fn get_class_instance_mut(
    &mut self,
    classname: &str,
  ) -> Result<&mut types::ObjectInstance> {
    if !self.heap.has_class_instance(classname) {
      self
        .heap
        .alloc_class_obj(&mut self.class_loader, classname)?;
    }

    self.heap.get_class_instance_mut(classname)
  }

  pub(crate) fn restore_stack(&mut self, args: Vec<types::Type>) -> Result<()> {
    debug!("Restoring stack");
    let mut i = 0;

    while i < args.len() {
      self.push_stack(args[i])?;

      // skip since `push_stack` already take care of category 2 values
      if args[i].get_category() == 2 {
        i += 1;
      }

      i += 1;
    }
    Ok(())
  }

  pub(crate) fn call_and_resolve_method(
    &mut self,
    class_name: &str,
    method_name: &str,
    descriptor: &str,
    args: Vec<types::Type>,
  ) -> Result<types::Type> {
    info!(
      "[-] Resolving {} {} {} {:?}",
      class_name, method_name, descriptor, args
    );

    let mut returned = types::Type::None;

    let (method_class, method) =
      self
        .class_loader
        .get_method_by_name(class_name, method_name, descriptor)?;

    if method.is_native() {
      debug!(
        "[!] {} {} is native call and resolve in {}",
        method_name, descriptor, class_name
      );

      // restore stack so that the call native works with a fake but
      // correct stack state
      self.restore_stack(args)?;

      returned = self
        .call_native(&method_class, method_name, descriptor)?
        .unwrap_or(types::Type::None);
    } else {
      self.push_frame_from_class(class_name, method_name, descriptor, args)?;

      let cpframe = self.frames.last().unwrap().clone();

      let num_frames = self.frames.len();

      // TODO: this does not feel right
      // TODO: if we push something, wont it go into the previous stack ?
      // TODO: ergo making it dirty with wrong values
      while self.frames.len() >= num_frames && *self.frames.get(num_frames - 1).unwrap() == cpframe
      {
        match self.step() {
          Ok(v) => {
            if let Some(v) = v {
              debug!("NEW RET VALUE {}", v);

              returned = v;
            }
          }
          Err(err) => {
            match self.handle_step_error(err, num_frames) {
              // can not handle the error => pass the error to the upper handler
              Err(err) => return Err(err),
              // can handle the error => continue
              _ => {}
            }
          }
        }
      }
    }

    // clean stack from dirty returns
    if returned != types::Type::None && !self.frames.is_empty() {
      self.pop_stack()?;
    }

    info!("[!] Resolved {} {} {}", class_name, method_name, returned);

    Ok(returned)
  }

  pub(crate) fn init_class(&mut self, class_name: &str) -> Result<()> {
    // init parent first
    let class = self.class_loader.get(class_name)?;

    if class.get_init() {
      return Ok(());
    }

    if class.has_parent() {
      let parent_name = class.get_parent_name().to_owned();

      drop(class);

      self.init_class(&parent_name)?;
    } else {
      drop(class);
    }

    if self.class_loader.get(class_name)?.get_init() {
      return Ok(());
    }

    debug!("[-] Init class {}", class_name);

    self.class_loader.get_mut(class_name)?.init();

    // Create class object before resolve clinit because it might be used in clinit
    self
      .heap
      .alloc_class_obj(&mut self.class_loader, class_name)?;

    if self
      .class_loader
      .get(class_name)?
      .has_function("<clinit>", "()V")
    {
      self.call_and_resolve_method(class_name, "<clinit>", "()V", vec![])?;
    }

    debug!("[-] Init class {} Done", class_name);

    Ok(())
  }

  fn show_local_in_frame(&self, local: &types::Type) {
    match *local {
      types::Type::ArrayRef(array_ref) => {
        let obj = self.heap.get_array_instance(array_ref).unwrap();
        error!("\t\t  {} -> {}", local, obj);
        error!("\t\t  Elements:");
        for element in obj.get_elements() {
          self.show_local_in_frame(element);
        }
        error!("\t\t  ----------------------------------");
      }
      types::Type::ObjectRef(obj_ref) => {
        let obj = self.heap.get_obj_instance(obj_ref).unwrap();
        if obj.is_string() {
          let str = self
            .heap
            .get_string(obj_ref)
            .unwrap_or("null string".to_string());
          error!("\t\t  {} -> String '{}' ({})", local, str, str.len());
        } else if obj.get_classname() == "java/lang/invoke/MemberName" {
          let member_name_name_ref = obj.get_field("name").unwrap().as_ref().unwrap();
          let member_name_name = self
            .heap
            .get_string(member_name_name_ref)
            .unwrap_or("null string".to_string());
          error!("\t\t  {} -> MemberName {}", local, member_name_name);
        } else if obj.get_classname() == "java/lang/Class" {
          let classname = self.heap.get_classname_from_class_obj(obj_ref).unwrap();
          error!("\t\t  {} -> Class<{}>", local, classname);
        } else {
          error!("\t\t  {} -> {}", local, obj);
        }
      }
      _ => {
        error!("\t\t  {}", local);
      }
    }
  }

  pub(crate) fn show_frames(&self) {
    error!("==================================================================");

    error!("Stack Trace");

    for c in &self.frames {
      error!(
        "{}.{} {}",
        c.get_classname(),
        c.get_method_name(),
        c.get_method_type()
      );

      for l in c.get_locals() {
        if *l == types::Type::None {
          break;
        }

        self.show_local_in_frame(l);
      }
    }

    error!("==================================================================");
  }

  pub(crate) fn pop_frame(&mut self) {
    self.frames.pop();
  }

  pub(crate) fn push_frame_from_class(
    &mut self,
    class_name: &str,
    method_name: &str,
    method_type: &str,
    args: Vec<types::Type>,
  ) -> Result<()> {
    let (method_class, method) =
      self
        .class_loader
        .get_method_code_by_name(class_name, method_name, method_type)?;

    let max_locals = method.get_max_locals();

    let mut nargs = args.clone();

    let arg_classes = get_argument_classnames(method_type);

    // add remaining arguments
    for i in nargs.len()..arg_classes.len() {
      nargs.push(get_default_value(arg_classes.get(i).unwrap()))
    }

    // TODO: set default value per type ?
    for _ in 0..max_locals {
      nargs.push(types::Type::None);
    }

    info!(
      "[-] Push frame from class {} for {} {} ({:?})",
      class_name, method_name, method_type, args
    );

    let frame = frame::Frame::new(
      method_class.to_string(),
      method_name.to_string(),
      method_type.to_string(),
      method.get_code_vec().clone(),
      nargs,
    );

    self.frames.push(frame);
    // self.show_frames();

    Ok(())
  }

  pub(crate) fn step(&mut self) -> Result<Option<types::Type>> {
    let counter = self.counter;

    self.counter += 1;

    let class_name = self.get_current_class()?.get_name().to_owned();

    let current_frame = self.get_current_frame_mut()?;

    let op = current_frame.read_current_opcode()?;

    debug!(
      "    [{}] [{}] Exec: {} in {}.{} {}",
      counter,
      current_frame.get_pc() - 1,
      opcode::OpCode::from_byte(op),
      class_name,
      current_frame.get_method_name(),
      current_frame.get_method_type(),
    );

    let res = match opcode::OpCode::from_byte(op) {
      // Integer
      opcode::OpCode::ILOAD => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_iload(index.into())
      }
      opcode::OpCode::ILOAD0 => self.exec_iload(0),
      opcode::OpCode::ILOAD1 => self.exec_iload(1),
      opcode::OpCode::ILOAD2 => self.exec_iload(2),
      opcode::OpCode::ILOAD3 => self.exec_iload(3),
      opcode::OpCode::ISTORE => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_istore(index.into())
      }
      opcode::OpCode::ISTORE0 => self.exec_istore(0),
      opcode::OpCode::ISTORE1 => self.exec_istore(1),
      opcode::OpCode::ISTORE2 => self.exec_istore(2),
      opcode::OpCode::ISTORE3 => self.exec_istore(3),
      opcode::OpCode::IASTORE => self.exec_iastore(),
      opcode::OpCode::IALOAD => self.exec_iaload(),

      o @ (opcode::OpCode::IFEQ
      | opcode::OpCode::IFNE
      | opcode::OpCode::IFLT
      | opcode::OpCode::IFLE
      | opcode::OpCode::IFGT
      | opcode::OpCode::IFGE) => self.exec_if(o),

      o @ (opcode::OpCode::IFICMPEQ
      | opcode::OpCode::IFICMPNE
      | opcode::OpCode::IFICMPLT
      | opcode::OpCode::IFICMPLE
      | opcode::OpCode::IFICMPGT
      | opcode::OpCode::IFICMPGE) => self.exec_if_icmp(o),

      opcode::OpCode::IADD => self.exec_iadd(),
      opcode::OpCode::ISUB => self.exec_isub(),
      opcode::OpCode::IDIV => self.exec_idiv(),
      opcode::OpCode::IMUL => self.exec_imul(),
      opcode::OpCode::INEG => self.exec_ineg(),
      opcode::OpCode::IREM => self.exec_irem(),

      opcode::OpCode::IINC => self.exec_iinc(),
      opcode::OpCode::IXOR => self.exec_ixor(),
      opcode::OpCode::IOR => self.exec_ior(),
      opcode::OpCode::IAND => self.exec_iand(),

      opcode::OpCode::IRETURN => self.exec_ireturn(),
      opcode::OpCode::ICONSTM1 => self.exec_iconst(-1),
      opcode::OpCode::ICONST0 => self.exec_iconst(0),
      opcode::OpCode::ICONST1 => self.exec_iconst(1),
      opcode::OpCode::ICONST2 => self.exec_iconst(2),
      opcode::OpCode::ICONST3 => self.exec_iconst(3),
      opcode::OpCode::ICONST4 => self.exec_iconst(4),
      opcode::OpCode::ICONST5 => self.exec_iconst(5),

      opcode::OpCode::IUSHR => self.exec_iushr(),
      opcode::OpCode::ISHL => self.exec_ishl(),
      opcode::OpCode::ISHR => self.exec_ishr(),
      opcode::OpCode::I2F => self.exec_i2f(),
      opcode::OpCode::I2D => self.exec_i2d(),
      opcode::OpCode::I2L => self.exec_i2l(),
      opcode::OpCode::I2S => self.exec_i2s(),
      opcode::OpCode::I2C => self.exec_i2c(),
      opcode::OpCode::I2B => self.exec_i2b(),

      // Short
      opcode::OpCode::SIPUSH => self.exec_sipush(),
      opcode::OpCode::SALOAD => self.exec_saload(),
      opcode::OpCode::SASTORE => self.exec_sastore(),

      // Byte
      opcode::OpCode::BALOAD => self.exec_baload(),
      opcode::OpCode::BASTORE => self.exec_bastore(),

      // Long
      opcode::OpCode::LLOAD => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_lload(index.into())
      }
      opcode::OpCode::LLOAD0 => self.exec_lload(0),
      opcode::OpCode::LLOAD1 => self.exec_lload(1),
      opcode::OpCode::LLOAD2 => self.exec_lload(2),
      opcode::OpCode::LLOAD3 => self.exec_lload(3),

      opcode::OpCode::LSTORE => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_lstore(index.into())
      }
      opcode::OpCode::LSTORE0 => self.exec_lstore(0),
      opcode::OpCode::LSTORE1 => self.exec_lstore(1),
      opcode::OpCode::LSTORE2 => self.exec_lstore(2),
      opcode::OpCode::LSTORE3 => self.exec_lstore(3),

      opcode::OpCode::LADD => self.exec_ladd(),
      opcode::OpCode::LSUB => self.exec_lsub(),
      opcode::OpCode::LDIV => self.exec_ldiv(),
      opcode::OpCode::LMUL => self.exec_lmul(),
      opcode::OpCode::LXOR => self.exec_lxor(),
      opcode::OpCode::LOR => self.exec_lor(),
      opcode::OpCode::LREM => self.exec_lrem(),
      opcode::OpCode::LAND => self.exec_land(),
      opcode::OpCode::LNEG => self.exec_lneg(),

      opcode::OpCode::LCONST0 => self.exec_lconst(0),
      opcode::OpCode::LCONST1 => self.exec_lconst(1),
      opcode::OpCode::L2I => self.exec_l2i(),
      opcode::OpCode::L2D => self.exec_l2d(),
      opcode::OpCode::L2F => self.exec_l2f(),
      opcode::OpCode::LSHL => self.exec_lshl(),
      opcode::OpCode::LSHR => self.exec_lshr(),
      opcode::OpCode::LUSHR => self.exec_lushr(),
      opcode::OpCode::LCMP => self.exec_lcmp(),
      opcode::OpCode::LASTORE => self.exec_lastore(),
      opcode::OpCode::LALOAD => self.exec_laload(),
      opcode::OpCode::LRETURN => self.exec_lreturn(),

      opcode::OpCode::LDC2W => {
        let index = self.get_current_frame_mut()?.read_ju2()?;

        self.exec_ldc2w(index)
      }

      // Double
      opcode::OpCode::DLOAD => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_dload(index.into())
      }
      opcode::OpCode::DLOAD0 => self.exec_dload(0),
      opcode::OpCode::DLOAD1 => self.exec_dload(1),
      opcode::OpCode::DLOAD2 => self.exec_dload(2),
      opcode::OpCode::DLOAD3 => self.exec_dload(3),
      opcode::OpCode::DSTORE => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_dstore(index.into())
      }
      opcode::OpCode::DSTORE0 => self.exec_dstore(0),
      opcode::OpCode::DSTORE1 => self.exec_dstore(1),
      opcode::OpCode::DSTORE2 => self.exec_dstore(2),
      opcode::OpCode::DSTORE3 => self.exec_dstore(3),
      o @ (opcode::OpCode::DCMPG | opcode::OpCode::DCMPL) => self.exec_dcmp(o),
      opcode::OpCode::DCONST0 => self.exec_dconst(0.0),
      opcode::OpCode::DCONST1 => self.exec_dconst(1.0),
      opcode::OpCode::DADD => self.exec_dadd(),
      opcode::OpCode::DSUB => self.exec_dsub(),
      opcode::OpCode::DDIV => self.exec_ddiv(),
      opcode::OpCode::DMUL => self.exec_dmul(),
      opcode::OpCode::DNEG => self.exec_dneg(),
      opcode::OpCode::D2I => self.exec_d2i(),
      opcode::OpCode::D2L => self.exec_d2l(),
      opcode::OpCode::DRETURN => self.exec_dreturn(),

      // Float
      opcode::OpCode::FLOAD => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_fload(index.into())
      }
      opcode::OpCode::FLOAD0 => self.exec_fload(0),
      opcode::OpCode::FLOAD1 => self.exec_fload(1),
      opcode::OpCode::FLOAD2 => self.exec_fload(2),
      opcode::OpCode::FLOAD3 => self.exec_fload(3),
      opcode::OpCode::FADD => self.exec_fadd(),
      opcode::OpCode::FSUB => self.exec_fsub(),
      opcode::OpCode::FDIV => self.exec_fdiv(),
      opcode::OpCode::FMUL => self.exec_fmul(),
      opcode::OpCode::FNEG => self.exec_fneg(),
      opcode::OpCode::FRETURN => self.exec_freturn(),
      opcode::OpCode::FSTORE => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_fstore(index.into())
      }
      opcode::OpCode::FSTORE0 => self.exec_fstore(0),
      opcode::OpCode::FSTORE1 => self.exec_fstore(1),
      opcode::OpCode::FSTORE2 => self.exec_fstore(2),
      opcode::OpCode::FSTORE3 => self.exec_fstore(3),
      opcode::OpCode::FCONST0 => self.exec_fconst(0.0),
      opcode::OpCode::FCONST1 => self.exec_fconst(1.0),
      opcode::OpCode::FCONST2 => self.exec_fconst(2.0),

      opcode::OpCode::F2D => self.exec_f2d(),
      opcode::OpCode::F2L => self.exec_f2l(),
      opcode::OpCode::F2I => self.exec_f2i(),
      o @ (opcode::OpCode::FCMPG | opcode::OpCode::FCMPL) => self.exec_fcmp(o),

      // Char
      opcode::OpCode::CALOAD => self.exec_caload(),

      // Reference
      opcode::OpCode::ACONSTNULL => self.exec_aconstnull(),
      opcode::OpCode::ALOAD => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_aload(index.into())
      }
      opcode::OpCode::ALOAD0 => self.exec_aload(0),
      opcode::OpCode::ALOAD1 => self.exec_aload(1),
      opcode::OpCode::ALOAD2 => self.exec_aload(2),
      opcode::OpCode::ALOAD3 => self.exec_aload(3),
      opcode::OpCode::ASTORE => {
        let index = self.get_current_frame_mut()?.read_ju1()?;

        self.exec_astore(index as usize)
      }
      opcode::OpCode::ASTORE0 => self.exec_astore(0),
      opcode::OpCode::ASTORE1 => self.exec_astore(1),
      opcode::OpCode::ASTORE2 => self.exec_astore(2),
      opcode::OpCode::ASTORE3 => self.exec_astore(3),

      o @ (opcode::OpCode::IFACMPEQ | opcode::OpCode::IFACMPNE) => self.exec_if_acmp(o),
      opcode::OpCode::ARETURN => self.exec_areturn(),

      // LDC
      opcode::OpCode::LDC => self.exec_ldc(),
      opcode::OpCode::LDCW => self.exec_ldcw(),

      // Static
      opcode::OpCode::GETSTATIC => self.exec_getstatic(),
      opcode::OpCode::PUTSTATIC => self.exec_putstatic(),
      opcode::OpCode::INVOKESTATIC => self.exec_invokestatic(),

      // Array
      opcode::OpCode::NEWARRAY => self.exec_newarray(),
      opcode::OpCode::ANEWARRAY => self.exec_anewarray(),
      opcode::OpCode::MULTIANEWARRAY => self.exec_multianewarray(),
      opcode::OpCode::ARRAYLENGTH => self.exec_arraylength(),
      opcode::OpCode::CASTORE => self.exec_castore(),
      opcode::OpCode::AALOAD => self.exec_aaload(),
      opcode::OpCode::AASTORE => self.exec_aastore(),

      // General
      opcode::OpCode::NOP => Ok(None),
      opcode::OpCode::POP => {
        self.pop_stack()?;

        Ok(None)
      }
      opcode::OpCode::POP2 => self.exec_pop2(),
      opcode::OpCode::NEW => self.exec_new(),
      opcode::OpCode::DUP => self.exec_dup(),
      opcode::OpCode::DUPX1 => self.exec_dupx1(),
      opcode::OpCode::DUPX2 => self.exec_dupx2(),
      opcode::OpCode::DUP2 => self.exec_dup2(),
      opcode::OpCode::BIPUSH => self.exec_bipush(),
      opcode::OpCode::IFNULL => self.exec_ifnull(),
      opcode::OpCode::IFNONNULL => self.exec_ifnonnull(),
      opcode::OpCode::GOTO => self.exec_goto(),
      opcode::OpCode::INVOKEVIRTUAL => self.exec_invokevirtual(),
      opcode::OpCode::INVOKESPECIAL => self.exec_invokespecial(),
      opcode::OpCode::INVOKEINTERFACE => self.exec_invokeinterface(),
      opcode::OpCode::INVOKEDYNAMIC => self.exec_invokedynamic(),
      opcode::OpCode::GETFIELD => self.exec_getfield(),
      opcode::OpCode::PUTFIELD => self.exec_putfield(),
      opcode::OpCode::CHECKCAST => self.exec_checkcast(),
      opcode::OpCode::INSTANCEOF => self.exec_instanceof(),
      opcode::OpCode::MONITORENTER => self.exec_monitorexit(),
      opcode::OpCode::MONITOREXIT => self.exec_monitorenter(),
      opcode::OpCode::ATHROW => self.exec_athrow(),
      opcode::OpCode::TABLESWITCH => self.exec_tableswitch(),
      opcode::OpCode::LOOKUPSWITCH => self.exec_lookupswitch(),

      opcode::OpCode::RETURN => {
        self.pop_frame();

        info!("        RETURN Void");

        debug!("-----------------------------------------------------------");

        Ok(Some(types::Type::None)) // null is a proper value -> none is more like void
      }
      _ => {
        debug!("not implemented {}", op);

        return Err(eyre!(errors::InternalError::NotImplemented));
      }
    };

    // if self.counter > 5_800_000 && self.counter < 5_801_000 {
    //   if let Some(logging_handle) = &self.logging_handle {
    //     let reload_handle = logging_handle.lock().unwrap();
    //     reload_handle.modify(|filter| *filter = filter::LevelFilter::DEBUG)?;
    //   }
    // }

    // DEBUG
    if !self.frames.is_empty()
      && ((self.get_current_frame()?.get_method_name() == "")
        || (self
          .get_current_frame()?
          .get_classname()
          .contains("parseRecipe")))
    {
      self.show_frames();
      pause();
    }

    res
  }
}
