use color_eyre::eyre::{eyre, Result};
use log::{debug, warn};

use crate::{
  runtime::{
    constants::*,
    errors,
    jvm::*,
    types::{self},
  },
  utils::{classname_to_descriptor, ju1, ju4},
};

impl JVM {
  pub(crate) fn native_dispatcher_java_lang_invoke_methodhandlenatives(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      ("init", "(Ljava/lang/invoke/MemberName;Ljava/lang/Object;)V") => {
        self.exec_native_method_handle_natives_init()
      }
      ("getNamedCon", "(I[Ljava/lang/Object;)I") => self.exec_native_get_named_con(),
      (
        "resolve",
        "(Ljava/lang/invoke/MemberName;Ljava/lang/Class;IZ)Ljava/lang/invoke/MemberName;",
      ) => self.exec_native_method_handle_natives_resolve(),
      ("getMemberVMInfo", "(Ljava/lang/invoke/MemberName;)Ljava/lang/Object;") => {
        self.exec_native_get_member_vm_info()
      }
      ("objectFieldOffset", "(Ljava/lang/invoke/MemberName;)J") => {
        self.exec_native_object_field_offset()
      }
      ("staticFieldBase", "(Ljava/lang/invoke/MemberName;)Ljava/lang/Object;") => {
        self.exec_native_static_field_base()
      }
      ("staticFieldOffset", "(Ljava/lang/invoke/MemberName;)J") => {
        self.exec_native_static_field_offset()
      }

      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/lang/invoke/MethodHandleNatives".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  fn exec_native_method_handle_natives_init(&mut self) -> Result<Option<types::Type>> {
    warn!("java/lang/invoke/MethodHandleNatives.init() not implemented");
    // TODO: `fill in vmtarget, vmindex while we have ctor in hand:`

    let ctor_ref = self.pop_object_ref()?;
    let this_ref = self.pop_object_ref()?;

    // TODO: is it allowed to init clazz here ?
    let ctor = self.heap.get_obj_instance(ctor_ref)?;
    let ctor_clazz_ref = ctor.get_field("clazz")?;

    let this = self.heap.get_obj_instance_mut(this_ref)?;
    this.put_field("clazz", ctor_clazz_ref)?;
    this.put_field(
      "flags",
      types::Type::Integer(MN_IS_CONSTRUCTOR | (REF_NEW_INVOKE_SPECIAL << MN_REFERENCE_KIND_SHIFT)),
    )?;

    Ok(None)
  }

  fn exec_native_get_named_con(&mut self) -> Result<Option<types::Type>> {
    warn!("java/lang/invoke/MethodHandleNatives.getNamedCon() not implemented");
    let _args_array_ref = self.pop_array_ref()?;

    let _kind = self.pop_ioperand()? as usize;

    let ret_value = types::Type::Integer(0);
    self.push_stack(ret_value)?;

    Ok(Some(ret_value))
  }

  fn exec_native_method_handle_natives_resolve(&mut self) -> Result<Option<types::Type>> {
    warn!("java/lang/invoke/MethodHandleNatives.resolve() not fully implemented");

    let _speculative = self.pop_stack()?.as_bool()?;
    let _lookup_mode = self.pop_ioperand()?;
    let _caller_class_ref = self.pop_ref()?; // may be null
    let member_name_ref = self.pop_object_ref()?;

    let member_name_obj = self.heap.get_obj_instance(member_name_ref)?;

    // Extract info from MemberName
    let clazz_ref = member_name_obj.get_field("clazz")?.as_ref()?;
    let name_ref = member_name_obj.get_field("name")?.as_ref()?;
    let type_ref = member_name_obj.get_field("type")?.as_ref()?; // might be Class or MethodType
    let flags = member_name_obj.get_field("flags")?.as_integer()?;

    let clazz_name = self.heap.get_classname_from_class_obj(clazz_ref)?;
    let name = self.heap.get_string(name_ref)?;
    let ref_kind = (flags >> MN_REFERENCE_KIND_SHIFT) & MN_REFERENCE_KIND_MASK;

    let (member_index, member_modifiers) =
      match self.get_member_name_index_and_flags(&clazz_name, &name, type_ref, ref_kind as ju1) {
        Ok(res) => res,
        Err(_) => return Err(eyre!(errors::JavaException::LinkageError)),
      };

    let member_name_obj_mut = self.heap.get_obj_instance_mut(member_name_ref)?;
    member_name_obj_mut.put_field("clazz", types::Type::ObjectRef(clazz_ref))?;
    member_name_obj_mut.put_field("type", types::Type::ObjectRef(type_ref))?;
    // update flags with member information
    member_name_obj_mut.put_field(
      "flags",
      types::Type::Integer(flags | member_modifiers as i32),
    )?;
    warn!("java/lang/invoke/MethodHandleNatives.resolve() needs to set `method` field");
    member_name_obj_mut.put_field("method", types::Type::Null)?;
    // if null, this guy is resolved
    member_name_obj_mut.put_field("resolution", types::Type::Null)?;
    member_name_obj_mut.new_field("vmindex", types::Type::Long(member_index as i64))?;

    let ret_value = types::Type::ObjectRef(member_name_ref);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // this and the return should be similar, but they might differ while dealing with unresolved
  // methods => so I resolve it again to get all important informations
  fn exec_native_get_member_vm_info_method(
    &mut self,
    member_name_ref: ju4,
    clazz_ref: ju4,
    name_ref: ju4,
    type_ref: ju4,
    flags: i32,
    member_class_name: &str,
    member: &str,
  ) -> Result<Vec<types::Type>> {
    // this is MethodType
    // to be sure we respect jvm internals
    let method_type_str_ref = self.resolve_method_type_descriptor(type_ref)?.as_ref()?;

    let method_type_str_ref = self.heap.get_string(method_type_str_ref)?;

    debug!("{} {} {}", member_class_name, member, method_type_str_ref);

    // we dont need the method to be concrete
    let (_method_real_class_name, _method, method_index) = self
      .class_loader
      .get_any_method_by_name_with_index(member_class_name, member, &method_type_str_ref)?;

    let slot = self
      .heap
      .alloc_integer(&mut self.class_loader, method_index)?;

    let member_name_obj = self.heap.get_obj_instance_mut(member_name_ref)?;

    assert!(clazz_ref != 0);
    assert!(name_ref != 0);
    assert!(type_ref != 0);
    // update informations
    member_name_obj.put_field(
      "flags",
      types::Type::Integer(flags | _method.get_access_flags() as i32),
    )?;

    Ok(vec![slot, types::Type::ObjectRef(member_name_ref)])
  }

  fn exec_native_get_member_vm_info_field(
    &mut self,
    type_ref: ju4,
    member_class_name: &str,
    member: &str,
  ) -> Result<Vec<types::Type>> {
    // this is Class<T>
    let member_type = self.heap.get_classname_from_class_obj(type_ref)?;

    let (field_real_class_name, _, field_index) = self.class_loader.get_field_by_name_with_index(
      member_class_name,
      member,
      &classname_to_descriptor(&member_type),
      0,
    )?;

    let slot = self
      .heap
      .alloc_integer(&mut self.class_loader, field_index)?;

    let target_class = self
      .heap
      .get_class_instance(&mut self.class_loader, &field_real_class_name)?;
    assert!(target_class.get_classname() == "java/lang/Class");

    Ok(vec![slot, types::Type::ObjectRef(target_class.get_ref())])
  }

  fn exec_native_get_member_vm_info_constructor(
    &mut self,
    member_name_ref: ju4,
    clazz_ref: ju4,
    name_ref: ju4,
    type_ref: ju4,
    flags: i32,
    member_class_name: &str,
    member: &str,
  ) -> Result<Vec<types::Type>> {
    assert!(member == "<init>");

    // this is MethodType
    // to be sure we respect jvm internals
    let method_type_str_ref = self.resolve_method_type_descriptor(type_ref)?.as_ref()?;

    let method_type_str_ref = self.heap.get_string(method_type_str_ref)?;

    debug!("{} {} {}", member_class_name, member, method_type_str_ref);

    // we dont need the method to be concrete
    let (_method_real_class_name, _method, method_index) = self
      .class_loader
      .get_any_method_by_name_with_index(member_class_name, member, &method_type_str_ref)?;

    let slot = self
      .heap
      .alloc_integer(&mut self.class_loader, method_index)?;

    let member_name_obj = self.heap.get_obj_instance_mut(member_name_ref)?;

    assert!(clazz_ref != 0);
    assert!(name_ref != 0);
    assert!(type_ref != 0);
    // update informations
    member_name_obj.put_field(
      "flags",
      types::Type::Integer(flags | _method.get_access_flags() as i32),
    )?;

    Ok(vec![slot, types::Type::ObjectRef(member_name_ref)])
  }

  fn exec_native_get_member_vm_info(&mut self) -> Result<Option<types::Type>> {
    warn!(
      "java/lang/invoke/MethodHandleNatives.getMemberVMInfo only works with materialized methods"
    );
    let member_name_ref = self.pop_object_ref()?;

    let member_name_obj = self.heap.get_obj_instance(member_name_ref)?;

    let clazz_ref = member_name_obj.get_field("clazz")?.as_ref()?;

    let name_ref = member_name_obj.get_field("name")?.as_ref()?; // may be null if not yet materialized

    let type_ref = member_name_obj.get_field("type")?.as_ref()?; // may be null if not yet materialized

    let flags = member_name_obj.get_field("flags")?.as_integer()?;

    if name_ref == 0 {
      return Err(eyre!(errors::InternalError::General(
        "MemberName name is Null (probably not materialized)".to_string()
      )));
    }

    if type_ref == 0 {
      return Err(eyre!(errors::InternalError::General(
        "MemberName type is Null (probably not materialized)".to_string()
      )));
    }

    let member_class_name = self
      .heap
      .get_class_from_class_obj(&mut self.class_loader, clazz_ref)?
      .get_name()
      .to_owned();

    let member = self.heap.get_string(name_ref)?;

    debug!(
      "vm member info : {} {} {} {}",
      member_class_name, member, type_ref, flags
    );

    // see reflect.Modifier
    let vm_info_arguments = if Constants::is_method(flags) {
      self.exec_native_get_member_vm_info_method(
        member_name_ref,
        clazz_ref,
        name_ref,
        type_ref,
        flags,
        &member_class_name,
        &member,
      )?
    } else if Constants::is_field(flags) {
      self.exec_native_get_member_vm_info_field(type_ref, &member_class_name, &member)?
    } else if Constants::is_constructor(flags) {
      self.exec_native_get_member_vm_info_constructor(
        member_name_ref,
        clazz_ref,
        name_ref,
        type_ref,
        flags,
        &member_class_name,
        &member,
      )?
    } else {
      return Err(eyre!(errors::InternalError::General(format!(
        "MemberName flag not handled {}",
        flags
      ))));
    };

    let vm_info_array = self
      .heap
      .alloc_array("java/lang/Object", vm_info_arguments, 2)?;

    self.push_stack(vm_info_array)?;
    Ok(Some(vm_info_array))
  }

  // public native long objectFieldOffset(MemberName name);
  pub(crate) fn exec_native_object_field_offset(&mut self) -> Result<Option<types::Type>> {
    let field_member_ref = self.pop_object_ref()?; // java/lang/invoke/MemberName;

    let field_member_obj = self.heap.get_obj_instance(field_member_ref)?;

    let field_member_class_ref = field_member_obj.get_field("clazz")?.as_ref()?;
    let field_member_class_name = self
      .heap
      .get_classname_from_class_obj(field_member_class_ref)?;

    let field_member_name_ref = field_member_obj.get_field("name")?.as_ref()?;
    let field_member_name_str = self.heap.get_string(field_member_name_ref)?;

    let offset = self
      .class_loader
      .get_field_offset(&field_member_class_name, &field_member_name_str)?;

    debug!(
      "{:?} {:?} {:?}",
      field_member_class_name, field_member_name_str, offset
    );

    let ret_value = types::Type::Long(offset);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  pub(crate) fn exec_native_static_field_base(&mut self) -> Result<Option<types::Type>> {
    let field_member_ref = self.pop_object_ref()?; // java/lang/invoke/MemberName;

    let field_member_obj = self.heap.get_obj_instance(field_member_ref)?;

    let field_member_class_ref = field_member_obj.get_field("clazz")?.as_ref()?;
    let field_member_class_name = self
      .heap
      .get_classname_from_class_obj(field_member_class_ref)?;

    let field_member_name_ref = field_member_obj.get_field("name")?.as_ref()?;
    let field_member_name_str = self.heap.get_string(field_member_name_ref)?;

    let field_member_name_type_ref = field_member_obj.get_field("type")?.as_ref()?;
    let field_member_name_type = classname_to_descriptor(
      &self
        .heap
        .get_classname_from_class_obj(field_member_name_type_ref)?,
    );

    let (field_base_class_name, _, _) = self.class_loader.get_field_by_name_with_index(
      &field_member_class_name,
      &field_member_name_str,
      &field_member_name_type,
      0,
    )?;

    let field_base_class = self
      .heap
      .get_static_class_instance(&field_base_class_name)?
      .get_ref();

    let ret_value = types::Type::ObjectRef(field_base_class);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  pub(crate) fn exec_native_static_field_offset(&mut self) -> Result<Option<types::Type>> {
    let field_member_ref = self.pop_object_ref()?; // java/lang/invoke/MemberName;

    let field_member_obj = self.heap.get_obj_instance(field_member_ref)?;

    let field_member_class_ref = field_member_obj.get_field("clazz")?.as_ref()?;
    let field_member_class_name = self
      .heap
      .get_classname_from_class_obj(field_member_class_ref)?;

    let field_member_name_ref = field_member_obj.get_field("name")?.as_ref()?;
    let field_member_name_str = self.heap.get_string(field_member_name_ref)?;

    let offset = self
      .class_loader
      .get_field_offset(&field_member_class_name, &field_member_name_str)?;

    debug!(
      "{:?} {:?} {:?}",
      field_member_class_name, field_member_name_str, offset
    );

    let ret_value = types::Type::Long(offset);
    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }
}
