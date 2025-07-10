use crate::{
  class_loader::class_file::MethodHandleResolved,
  notimpl,
  runtime::{
    constants::*,
    errors,
    jvm::*,
    types::{self},
  },
  utils::{
    classname_to_descriptor, descriptor_to_classname, get_parameters_type_descriptor,
    get_return_type_descriptor, ju1, ju2, ju4, parse_parameter_types,
  },
};
use color_eyre::eyre::{eyre, Result};
use log::{debug, error, warn};

impl JVM {
  pub(crate) fn resolve_method_type_descriptor(
    &mut self,
    method_type_ref: ju4,
  ) -> Result<types::Type> {
    self.call_and_resolve_method(
      "java/lang/invoke/MethodType",
      "descriptorString",
      "()Ljava/lang/String;",
      vec![types::Type::ObjectRef(method_type_ref)],
    )
  }

  pub(crate) fn resolve_member_name_type_descriptor(
    &mut self,
    member_name_ref: ju4,
  ) -> Result<types::Type> {
    self.call_and_resolve_method(
      "java/lang/invoke/MemberName",
      "getMethodDescriptor",
      "()Ljava/lang/String;",
      vec![types::Type::ObjectRef(member_name_ref)],
    )
  }

  pub(crate) fn get_member_name_index_and_flags(
    &mut self,
    classname: &str,
    name: &str,
    type_ref: ju4,
    ref_kind: ju1,
  ) -> Result<(i32, ju2)> {
    match ref_kind as i32 {
      REF_GET_FIELD | REF_PUT_FIELD | REF_GET_STATIC | REF_PUT_STATIC => {
        let type_obj_class =
          classname_to_descriptor(&self.heap.get_classname_from_class_obj(type_ref)?);
        let (_, _field, field_index) =
          self
            .class_loader
            .get_field_by_name_with_index(classname, name, &type_obj_class, 0)?;
        Ok((field_index, _field.get_access_flags()))
      }
      REF_INVOKE_VIRTUAL
      | REF_INVOKE_STATIC
      | REF_INVOKE_SPECIAL
      | REF_INVOKE_INTERFACE
      | REF_NEW_INVOKE_SPECIAL => {
        let type_obj = self.heap.get_obj_instance(type_ref)?;
        let method_type_str_ref = match type_obj.get_classname() {
          "java/lang/String" => type_ref,
          "java/lang/invoke/MethodType" => {
            self.resolve_method_type_descriptor(type_ref)?.as_ref()?
          }
          _ => panic!(),
        };
        let type_str = self.heap.get_string(method_type_str_ref)?;

        let (_, _method, method_index) = self
          .class_loader
          .get_any_method_by_name_with_index(classname, name, &type_str)?;
        Ok((method_index, _method.get_access_flags()))
      }
      _ => Err(eyre!("Unsupported reference kind: {}", ref_kind)),
    }
  }

  pub(crate) fn native_dispatcher_java_lang_invoke_methodhandle(
    &mut self,
    name: &str,
    type_str: &str,
  ) -> Result<Option<types::Type>> {
    match (name, type_str) {
      (
        "invokeBasic",
        _, // PolymorphicSignature
      ) => self.exec_native_invoke_basic(type_str),
      (
        "invokeExact",
        _, // PolymorphicSignature
      ) => self.exec_native_invoke_exact(type_str),
      (
        "invokeVirtual",
        _, // PolymorphicSignature
      ) => self.exec_native_invoke_virtual(type_str),

      _ => Err(eyre!(errors::InternalError::NativeNotImplemented(
        "java/lang/invoke/MethodHandle".to_string(),
        name.to_owned(),
        type_str.to_owned()
      ))),
    }
  }

  // For simple cases (e.g. findStatic) the method can be direcly called
  // For more complex ones (e.g. adapters) a LambdaForm is generated
  fn resolve_direct_method_handle(
    &mut self,
    method_handle: &types::ObjectInstance,
  ) -> Result<(String, String, String, i32)> {
    let member_name_ref = method_handle.get_field("member")?.as_ref()?;
    let member_name = self.heap.get_obj_instance(member_name_ref)?;

    let method_name_ref = member_name.get_field("name")?.as_ref()?;
    let method_name = self.heap.get_string(method_name_ref)?;

    let method_clazz_ref = member_name.get_field("clazz")?.as_ref()?;
    let method_clazz_name = self.heap.get_classname_from_class_obj(method_clazz_ref)?;

    debug!(
      "DIRECT METHOD HANDLE {} {} {}",
      member_name, method_clazz_name, method_name
    );

    let method_flags = member_name.get_field("flags")?.as_integer()?;

    let method_type_str = if Constants::is_method(method_flags) {
      let method_type_ref = method_handle.get_field("type")?.as_ref()?;
      let method_type_str_ref = self
        .resolve_method_type_descriptor(method_type_ref)?
        .as_ref()?;

      self.heap.get_string(method_type_str_ref)?
    } else {
      panic!();
    };

    // it could refer to a method or a field (getter)

    // TODO: better way to get it ? maybe avoiding mutability on self

    Ok((
      method_clazz_name,
      method_name,
      method_type_str,
      method_flags,
    ))
  }

  fn call_direct_method_handle(
    &mut self,
    method_handle: &types::ObjectInstance,
    args: &mut Vec<types::Type>,
  ) -> Result<types::Type> {
    let member_name_ref = method_handle.get_field("member")?.as_ref()?;
    let member_name = self.heap.get_obj_instance(member_name_ref)?;

    let method_name_ref = member_name.get_field("name")?.as_ref()?;
    let method_name = self.heap.get_string(method_name_ref)?;

    let method_clazz_ref = member_name.get_field("clazz")?.as_ref()?;
    let method_clazz_name = self.heap.get_classname_from_class_obj(method_clazz_ref)?;

    let method_flags = member_name.get_field("flags")?.as_integer()?;

    debug!(
      "CALL DIRECT METHOD HANDLE {} {} {} {}",
      member_name, method_clazz_name, method_name, method_flags
    );

    // it could refer to a method or a field (getter)

    match method_flags {
      // treat as a method
      _ if Constants::is_method(method_flags) => {
        if JVM::is_method_handle_new_invoke_special(method_flags) {
          // we need to push this
          let this_ref = self
            .heap
            .alloc_obj(&mut self.class_loader, &method_clazz_name)?;
          // `args` is already ordered
          // replace the first element of `args` with a new object
          args[0] = this_ref;
        } else {
          // we have the method handle reference has first
          // element of `args`, but in this case
          // it is not expected
          args.remove(0);
        }

        let method_type_ref = method_handle.get_field("type")?.as_ref()?;
        let method_type_str_ref = self
          .resolve_method_type_descriptor(method_type_ref)?
          .as_ref()?;

        let method_type_str = self.heap.get_string(method_type_str_ref)?;

        // call the method
        let return_value = self.call_and_resolve_method(
          &method_clazz_name,
          &method_name,
          &method_type_str,
          args.to_vec(),
        )?;

        debug!("invoke exact return value : {}", return_value);

        // if new invoke special => return it self
        let ret_value = if JVM::is_method_handle_new_invoke_special(method_flags) {
          *args.first().unwrap()
        } else {
          return_value
        };
        Ok(ret_value)
      }
      _ if Constants::is_field(method_flags) => {
        error!("Field in DirectMethodHandle not supported yet");
        Err(eyre!(errors::InternalError::NotImplemented))
      }
      _ if Constants::is_constructor(method_flags) => {
        let this_ref = self
          .heap
          .alloc_obj(&mut self.class_loader, &method_clazz_name)?;
        // `args` is already ordered
        // replace the first element of `args` with a new object
        args[0] = this_ref;

        let method_type_ref = method_handle.get_field("type")?.as_ref()?;
        let method_type_str_ref = self
          .resolve_method_type_descriptor(method_type_ref)?
          .as_ref()?;

        let method_type_str = self.heap.get_string(method_type_str_ref)?;

        // the method type is not the one in the member name
        // since that one contains the return type as Ljava/lang/Object
        // while <init> has no return (V)

        // contains only the arguments of <init>
        let parameters_type_str = get_parameters_type_descriptor(&method_type_str);
        let init_method_type_str = format!("({})V", parameters_type_str);

        // call the method
        let return_value = self.call_and_resolve_method(
          &method_clazz_name,
          &method_name,
          &init_method_type_str,
          args.to_vec(),
        )?;

        debug!("invoke exact return value : {}", return_value);

        Ok(*args.first().unwrap())
      }
      _ => notimpl!(),
    }
  }

  fn call_bound_method_handle(
    &mut self,
    method_handle: &types::ObjectInstance,
    args: &mut [types::Type],
  ) -> Result<types::Type> {
    self.interpret_lambda_form(method_handle, args)
  }

  fn interpret_lambda_form(
    &mut self,
    method_handle: &types::ObjectInstance,
    args: &[types::Type],
  ) -> Result<types::Type> {
    // mini-interpreter for lambdaform

    let lambdaform_ref = method_handle.get_field("form")?.as_ref()?;
    let lambdaform = self.heap.get_obj_instance(lambdaform_ref)?;

    // 1. get the names
    let names_ref = lambdaform.get_field("names")?.as_ref()?;
    let names = self.heap.get_array_instance(names_ref)?.clone();

    // 2. prepare the frame
    let arity = lambdaform.get_field("arity")?.as_integer()? as usize;
    let mut guard_arity = arity;
    let mut values = vec![types::Type::None; names.len()];
    let mut category2_arguments = 0;
    let mut i = 0;
    let mut values_i = 0;
    while i < guard_arity {
      // name.index refers to the index in the arguments array
      // it does not know that in stack long/double occupy two slots
      values[values_i] = args[i];
      // get long/double
      if args[i].get_category() == 2 {
        category2_arguments += 1;
        // increment arity by one so that this cycle gather all args correctly
        guard_arity += 1;
        i += 1;
      }
      i += 1;
      values_i += 1;
    }
    // we double count them in the previous loop
    category2_arguments /= 2;
    debug!("ARITY {} - ARGS {}", arity, args.len());
    assert!(args.len() == guard_arity - category2_arguments);

    debug!("NAMES {}", names.len());
    debug!("OBJ {}", method_handle);
    debug!("VALUES {:?}", values);

    // 3. interpret
    for i in (arity..names.len()).step_by(1) {
      let name_ref = names.get(i)?.as_ref()?; // argL*
      let name = self.heap.get_obj_instance(name_ref)?;

      let arguments_ref = name.get_field("arguments")?.as_ref()?;
      let arguments = self.heap.get_array_instance(arguments_ref)?;

      let function_ref = name.get_field("function")?.as_ref()?;
      let function = self.heap.get_obj_instance(function_ref)?;

      let member_ref = function.get_field("member")?.as_ref()?;
      let member = self.heap.get_obj_instance(member_ref)?;

      let member_name_ref = member.get_field("name")?.as_ref()?;
      let member_name_str = self.heap.get_string(member_name_ref)?;

      let member_type_ref = member.get_field("type")?.as_ref()?;
      let member_type = self.heap.get_obj_instance(member_type_ref)?;

      let member_class_ref = member.get_field("clazz")?.as_ref()?;
      let member_class_name_str = self.heap.get_classname_from_class_obj(member_class_ref)?;

      let mut invoke_arguments_array = vec![];

      for arg in arguments.get_elements() {
        let new_arg = match *arg {
          types::Type::Integer(_) => panic!(),
          types::Type::ObjectRef(obj_ref) => {
            let obj = self.heap.get_obj_instance(obj_ref)?;
            // then it must be of class java/lang/invoke/LambdaForm$Name
            // and I use the `index` field to get the correct argument out of `values`
            let index = obj.get_field("index")?.as_integer()? as usize;
            debug!("index {} {:?}", index, values.get(index));
            *values.get(index).unwrap() // TODO: avoid unwrap
          }
          _ => panic!(),
        };
        invoke_arguments_array.push(new_arg);
        // dont care if long/double since `args` is already correctly formatted
        // e.g. long/double already occupy two slots
        // // in case the arg is long/double it counts as two arguments
        if new_arg.get_category() == 2 {
          invoke_arguments_array.push(new_arg);
        }
      }
      debug!(
        "NAME {} {}.{} {} -> {:?}",
        function.get_classname(),
        member_class_name_str,
        member_name_str,
        member_type,
        invoke_arguments_array
      );

      // TODO: there are synthetic methods to access fields
      // TODO: such as argL0, argL1, argL2, argL3, ...
      // TODO: I did not find a good solution for them
      // TODO: so for now I will treat them manually
      let member_name_str_ptr: &str = &member_name_str;
      let res = match member_name_str_ptr {
        _ if member_name_str_ptr.starts_with("arg") => {
          // im handling argL* manually, so the method handle is the first
          // element of the arguments
          let current_method_handle_ref = invoke_arguments_array[0].as_ref()?;
          let current_method_handle = self.heap.get_obj_instance(current_method_handle_ref)?;
          current_method_handle.get_field(&member_name_str)?
        }
        "invokeBasic" => {
          // this is most likely just a real method
          // for instance we could have a invokeBasic
          let mut args_type: String = "".to_owned();
          // skip the first argument since it is the `this`
          let mut i = 1;
          while i < invoke_arguments_array.len() {
            let arg = &invoke_arguments_array[i];
            debug!("Field of {} : {}", i, arg);

            let arg_type = arg.get_type(&self.heap);
            args_type = format!("{}{}", args_type, arg_type);

            // skip it if long/doable
            if arg.get_category() == 2 {
              i += 1;
            }

            i += 1; // Manual control over increment
          }
          let descriptor = format!("({})Ljava/lang/Object", args_type);
          self.call_and_resolve_method(
            &member_class_name_str,
            &member_name_str,
            &descriptor,
            invoke_arguments_array,
          )?
        }
        _ => panic!(),
      };
      values[i] = res;
      debug!("RES {} : {}", i, res);
    }

    let lambdaform = self.heap.get_obj_instance(lambdaform_ref)?;
    let result_index = lambdaform.get_field("result")?.as_integer()? as usize;
    debug!("LAMBDAFORM RESULT {}", values[result_index]);
    Ok(values[result_index])
  }

  fn resolve_bound_method_handle(
    &mut self,
    method_handle: &types::ObjectInstance,
  ) -> Result<(String, String, String, i32)> {
    // BoundMethodHandle has the arguments
    // LambdaForm tells how to use the arguments
    // get the lambda form and invokeBasic using argL0, argL1, ...

    let arg_l0_ref = method_handle.get_field("argL0")?.as_ref()?;
    let arg_l0 = self.heap.get_obj_instance(arg_l0_ref)?.clone();
    assert!(arg_l0.get_classname() == "java/lang/invoke/DirectMethodHandle");

    let args = vec![types::Type::None, types::Type::None, types::Type::None];
    self.interpret_lambda_form(method_handle, &args)?;

    panic!();
  }

  /// Resolve info of MethodHandle object
  ///
  /// # Arguments
  ///
  /// * `method_handle_ref` - MethodHandle heap ref
  ///
  /// # Returns
  ///
  /// (String, String, String)
  /// as (classname, method_name, method_type)
  /// It is performed in this way to be safe (but more expensive)
  pub(crate) fn resolve_method_handle(
    &mut self,
    method_handle_ref: ju4,
  ) -> Result<(String, String, String, i32)> {
    let method_handle = self.heap.get_obj_instance(method_handle_ref)?.clone();

    debug!("RESOLVING METHOD HANDLE {}", method_handle);

    if types::Type::check_type(
      &mut self.class_loader,
      "java/lang/invoke/DirectMethodHandle",
      method_handle.get_classname(),
    )? {
      return self.resolve_direct_method_handle(&method_handle);
    }

    if types::Type::check_type(
      &mut self.class_loader,
      "java/lang/invoke/BoundMethodHandle",
      method_handle.get_classname(),
    )? {
      return self.resolve_bound_method_handle(&method_handle);
    }

    error!(
      "method handle not handled {}",
      method_handle.get_classname()
    );

    panic!()
  }

  /// Resolve info of MethodHandle object
  ///
  /// # Arguments
  ///
  /// * `method_handle_ref` - MethodHandle heap ref
  /// * `args` - MethodHandle arguments (already correctly ordered)
  ///   the first element of `args` is the the method handle reference
  ///
  /// # Returns
  ///
  /// types::Type
  /// It is performed in this way to be safe (but more expensive)
  pub(crate) fn call_method_handle(
    &mut self,
    method_handle_ref: ju4,
    args: &mut Vec<types::Type>,
  ) -> Result<types::Type> {
    let method_handle = self.heap.get_obj_instance(method_handle_ref)?.clone();
    debug!("CALLING METHOD HANDLE {}", method_handle);

    if types::Type::check_type(
      &mut self.class_loader,
      "java/lang/invoke/DirectMethodHandle",
      method_handle.get_classname(),
    )? {
      return self.call_direct_method_handle(&method_handle, args);
    }

    if types::Type::check_type(
      &mut self.class_loader,
      "java/lang/invoke/BoundMethodHandle",
      method_handle.get_classname(),
    )? {
      return self.call_bound_method_handle(&method_handle, args);
    }

    error!(
      "method handle not handled {}",
      method_handle.get_classname()
    );

    panic!()
  }

  fn create_member_name(
    &mut self,
    classname: &str,
    name: &str,
    type_ref: ju4,
    flags: i32,
  ) -> Result<types::Type> {
    let class_ref = self
      .heap
      .get_class_instance(&mut self.class_loader, classname)?
      .get_ref();
    let name_ref = self.heap.alloc_string(&mut self.class_loader, name)?;

    let member_name_ref = self
      .heap
      .alloc_obj(&mut self.class_loader, "java/lang/invoke/MemberName")?
      .as_ref()?;
    let member_name = self.heap.get_obj_instance_mut(member_name_ref)?;

    member_name.put_field("clazz", types::Type::ObjectRef(class_ref))?;
    member_name.put_field("name", name_ref)?;
    member_name.put_field("type", types::Type::ObjectRef(type_ref))?;
    member_name.put_field("flags", types::Type::Integer(flags))?;

    Ok(types::Type::ObjectRef(member_name_ref))
  }

  fn create_member_name_from_method_handle(
    &mut self,
    method_handle: MethodHandleResolved,
  ) -> Result<types::Type> {
    warn!("Flag field creating for MemberName from MethodHandle is possibly wrong");
    let member_type_ref = self
      .heap
      .alloc_string(&mut self.class_loader, method_handle.get_method_type())?
      .as_ref()?;
    let (_, member_modifier) = self.get_member_name_index_and_flags(
      method_handle.get_classname(),
      method_handle.get_method_name(),
      member_type_ref,
      method_handle.get_ref_kind(),
    )?;

    let flags = member_modifier as i32
      | MN_IS_METHOD
      | (method_handle.get_ref_kind() as i32 & MN_REFERENCE_KIND_MASK) << MN_REFERENCE_KIND_SHIFT;

    self.create_member_name(
      method_handle.get_classname(),
      method_handle.get_method_name(),
      member_type_ref,
      flags,
    )
  }

  fn create_direct_method_handle(
    &mut self,
    caller_class: &str,
    method_handle_resolved: MethodHandleResolved,
  ) -> Result<types::Type> {
    debug!("Creating direct method handle");
    debug!("{}", method_handle_resolved);

    let ref_kind = method_handle_resolved.get_ref_kind();
    let refc_ref = self
      .heap
      .get_class_instance(
        &mut self.class_loader,
        method_handle_resolved.get_classname(),
      )?
      .get_ref();

    let member_name_ref = self.create_member_name_from_method_handle(method_handle_resolved)?;

    let caller_class_ref = self
      .heap
      .get_class_instance(&mut self.class_loader, caller_class)?
      .get_ref();

    let method_handle_ref = self.call_and_resolve_method(
      "java/lang/invoke/DirectMethodHandle",
      "make",
      "(BLjava/lang/Class;Ljava/lang/invoke/MemberName;Ljava/lang/Class;)Ljava/lang/invoke/DirectMethodHandle;",
      vec![types::Type::Byte(ref_kind as i8), types::Type::ObjectRef(refc_ref), member_name_ref, types::Type::ObjectRef(caller_class_ref)])?.as_ref()?;

    Ok(types::Type::ObjectRef(method_handle_ref))
  }

  pub(crate) fn create_method_handle(
    &mut self,
    caller_class: &str,
    method_handle_resolved: MethodHandleResolved,
  ) -> Result<types::Type> {
    debug!("Creating method handle");
    self.create_direct_method_handle(caller_class, method_handle_resolved)
  }

  pub(crate) fn create_method_type(&mut self, method_type: &str) -> Result<types::Type> {
    debug!("Creating method type");

    let return_type_str = get_return_type_descriptor(method_type);
    let parameter_type_str = get_parameters_type_descriptor(method_type);
    let parameter_types_str = parse_parameter_types(parameter_type_str);

    debug!("MethodType Rtype {}", return_type_str);

    let return_type_class_ref = self
      .heap
      .get_class_instance(&mut self.class_loader, &return_type_str)?
      .get_ref();

    let mut parameter_type_class_refs = vec![];
    for pt in parameter_types_str {
      parameter_type_class_refs.push(types::Type::ObjectRef(
        self
          .heap
          .get_class_instance(&mut self.class_loader, &descriptor_to_classname(pt))?
          .get_ref(),
      ));
    }

    let method_parameters_array_ref =
      self
        .heap
        .alloc_array("java/lang/Class", parameter_type_class_refs, 0)?;

    let method_type_ref = self.call_and_resolve_method(
      "java/lang/invoke/MethodType",
      "makeImpl",
      "(Ljava/lang/Class;[Ljava/lang/Class;Z)Ljava/lang/invoke/MethodType;",
      vec![
        types::Type::ObjectRef(return_type_class_ref),
        method_parameters_array_ref,
        types::Type::Boolean(true),
      ],
    )?;

    Ok(method_type_ref)
  }

  // TODO: consider different invoke kind
  // ASSUMPTION: the method in MethodHandle is already materialized
  pub fn exec_native_invoke_basic(&mut self, arg_type: &str) -> Result<Option<types::Type>> {
    warn!("java/lang/invoke/MethodHandle.invokeBasic() not fully implemented");

    // TODO: proper implement this
    self.exec_native_invoke_exact(arg_type)
  }

  fn is_method_handle_new_invoke_special(flags: i32) -> bool {
    (flags >> MN_REFERENCE_KIND_SHIFT) & REF_NEW_INVOKE_SPECIAL != 0
  }

  // TODO: consider different invoke kind
  // TODO: add type checking (no type coercion or conversion)
  // ASSUMPTION: the method in MethodHandle is already materialized
  pub fn exec_native_invoke_exact(&mut self, arg_type: &str) -> Result<Option<types::Type>> {
    warn!("java/lang/invoke/MethodHandle.invokeExact() not fully implemented");

    let params_types = parse_parameter_types(get_parameters_type_descriptor(arg_type));
    debug!("from {:?} to {:?}", arg_type, params_types);

    let mut args = vec![];
    for _ in 0..params_types.len() {
      let arg = self.pop_stack()?;
      args.push(arg);
      if arg.get_category() == 2 {
        args.push(arg);
      }
    }

    let method_handle_ref = self.pop_object_ref()?; // This is the MethodHandle instance
    args.push(types::Type::ObjectRef(method_handle_ref));

    debug!(
      "{} / {}",
      self.get_current_frame()?.get_pc(),
      self.get_current_frame()?.get_code_length()
    );
    debug!("{}", self.get_current_frame()?);

    args.reverse();

    debug!("{:#?}", args);
    debug!("{:#?}", arg_type);
    debug!("{:#?}", params_types);

    let ret_value = self.call_method_handle(method_handle_ref, &mut args)?;

    debug!("invoke exact return value : {}", ret_value);

    self.push_stack(ret_value)?;
    Ok(Some(ret_value))
  }

  // TODO: consider different invoke kind
  // TODO: add type checking (no type coercion or conversion)
  // ASSUMPTION: the method in MethodHandle is already materialized
  pub fn exec_native_invoke_virtual(&mut self, arg_type: &str) -> Result<Option<types::Type>> {
    warn!("java/lang/invoke/MethodHandle.invokeVirtual() not fully implemented");

    // TODO: proper implement this
    self.exec_native_invoke_exact(arg_type)
  }
}
