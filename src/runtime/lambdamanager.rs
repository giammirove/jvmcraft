use crate::{runtime::jvm::*, utils::get_argument_classnames};
use color_eyre::eyre::{eyre, Result};
use log::{debug, warn};
use std::collections::HashMap;

use crate::{
  class_loader::{class_file::InvokeDynamicResolved, constant_pool::CpInfoInfoEnum},
  runtime::{errors, types},
  utils::{ju2, ju4},
};

#[derive(Debug)]
pub(crate) struct LambdaManager {
  // key: class | method name | bytecode offset
  // value: call site ref in the heap
  callsites: HashMap<String, ju4>,
}

impl LambdaManager {
  pub(crate) fn new() -> Self {
    LambdaManager {
      callsites: HashMap::new(),
    }
  }

  pub(crate) fn add_call_site(
    &mut self,
    classname: &str,
    method_name: &str,
    offset: usize,
    callsite_ref: ju4,
  ) {
    if self.get_call_site(classname, method_name, offset).is_some() {
      return;
    }

    let key = LambdaManager::gen_key(classname, method_name, offset);
    self.callsites.insert(key, callsite_ref);
  }

  pub(crate) fn get_call_site(
    &self,
    classname: &str,
    method_name: &str,
    offset: usize,
  ) -> Option<ju4> {
    let key = LambdaManager::gen_key(classname, method_name, offset);
    self.callsites.get(&key).copied()
  }

  fn gen_key(classname: &str, method_name: &str, offset: usize) -> String {
    format!("{}_{}_{}", classname, method_name, offset)
  }
}

impl JVM {
  fn create_method_handles_lookup(&mut self, class_name: &str) -> Result<types::Type> {
    let method_handles_lookup_ref = self
      .heap
      .alloc_obj(
        &mut self.class_loader,
        "java/lang/invoke/MethodHandles$Lookup",
      )?
      .as_ref()?;

    let current_class_ref = self
      .heap
      .get_class_instance(&mut self.class_loader, class_name)?
      .get_ref();

    let method_handles = self.heap.get_obj_instance_mut(method_handles_lookup_ref)?;
    method_handles.put_field("lookupClass", types::Type::ObjectRef(current_class_ref))?;
    let public_mod = 0x1;
    let private_mod = 0x2;
    let protected_mod = 0x4;
    let static_mod = 0x8;
    let module_mod = static_mod << 1;
    let unconditional_mod = static_mod << 2;
    let original_mod = static_mod << 3;
    method_handles.put_field(
      "allowedModes",
      types::Type::Integer(
        public_mod
          | private_mod
          | protected_mod
          | static_mod
          | module_mod
          | unconditional_mod
          | original_mod,
      ),
    )?;

    Ok(types::Type::ObjectRef(method_handles_lookup_ref))
  }

  fn resolve_bootstrap_argument(&mut self, classname: &str, index: ju2) -> Result<types::Type> {
    let cpinfo = self
      .class_loader
      .get(classname)?
      .resolve_index(index)?
      .clone();
    match cpinfo.get_info() {
      CpInfoInfoEnum::MethodHandle(_) => {
        let method_handle_resolved = self.class_loader.resolve_method_handle(classname, index)?;
        self.create_method_handle(classname, method_handle_resolved)
      }
      CpInfoInfoEnum::MethodType(_) => {
        let method_type_resolved = self.class_loader.resolve_method_type(classname, index)?;
        self.create_method_type(&method_type_resolved)
      }
      CpInfoInfoEnum::String(info) => {
        let string = self
          .class_loader
          .resolve_string(classname, info.get_string_index())?;
        self.heap.alloc_string(&mut self.class_loader, &string)
      }
      _ => Err(eyre!(errors::InternalError::General(format!(
        "Bootstrap argument not supported: {}",
        cpinfo.get_info()
      )))),
    }
  }

  pub(crate) fn exec_invokedynamic_newcallsite(
    &mut self,
    curr_class_name: &str,
    invoke_dynamic_resolved: &InvokeDynamicResolved,
  ) -> Result<ju4> {
    let method_lookup = self.create_method_handles_lookup(curr_class_name)?;

    // I need
    // - Lookup as MethodHandles.Lookup.
    // - Method Name as String
    // - Method Descriptor as MethodType
    // as minimum arguments for the bootstrap method
    //
    // the bootstrap method will return a callsite
    // the callsite must be associate with this invokedynamic

    let method_name_ref = self.heap.alloc_string(
      &mut self.class_loader,
      invoke_dynamic_resolved.get_method_name(),
    )?;

    let method_descriptor_ref =
      self.create_method_type(invoke_dynamic_resolved.get_method_type())?;

    // Next step is to parse the arguments defined in the Bootstrap Method

    let mut factory_args = vec![method_lookup, method_name_ref, method_descriptor_ref];
    for arg in invoke_dynamic_resolved.get_arguments() {
      factory_args.push(self.resolve_bootstrap_argument(curr_class_name, *arg)?);
    }

    let bootstrap_arg_classes =
      get_argument_classnames(invoke_dynamic_resolved.get_bootstrap_method_type());

    // for variadic (...Object) if is is empty, it is not written in the arguments
    // of the bootstrap method, but we still need it
    for _ in factory_args.len()..bootstrap_arg_classes.len() {
      // assuming it's the only case possible
      let empty_array = self.heap.alloc_array("java/lang/Object", vec![], 0)?;
      factory_args.push(empty_array);
    }

    // And then create the CallSite using the java/lang/invoke/LambdaMetafactory
    // just obtained

    debug!("{}", invoke_dynamic_resolved);
    debug!("FACTORY ARGS {:?}", factory_args);

    // TODO: call based on ref_kind
    let callsite_ref = self
      .call_and_resolve_method(
        invoke_dynamic_resolved.get_bootstrap_class_name(),
        invoke_dynamic_resolved.get_bootstrap_method_name(),
        invoke_dynamic_resolved.get_bootstrap_method_type(),
        factory_args,
      )?
      .as_ref()?;

    Ok(callsite_ref)
  }

  fn invoke_constant_callsite(
    &mut self,
    callsite_obj: &types::ObjectInstance,
  ) -> Result<types::Type> {
    let target_ref = callsite_obj.get_field("target")?.as_ref()?; // MethodHandle
    let target = self.heap.get_obj_instance(target_ref)?;

    let target_type_ref = target.get_field("type")?.as_ref()?; // MethodType
    let target_type = self.heap.get_obj_instance(target_type_ref)?;
    let ptypes_ref = target_type.get_field("ptypes")?.as_ref()?; // Array of Class<?>
    let ptypes_len = self
      .heap
      .get_array_instance(ptypes_ref)?
      .get_elements()
      .len();

    let target_classname = target.get_classname().to_owned();
    let mut invoke_type: String = "(".to_string();
    let mut args = vec![];

    for _ in 0..ptypes_len {
      args.push(self.pop_stack()?);
      invoke_type += "Ljava/lang/Object;";
    }
    args.push(types::Type::ObjectRef(target_ref));
    args.reverse();
    invoke_type += ")Ljava/lang/Object;";

    debug!(
      "CALLSITE {} {} {} {:?}",
      target_classname,
      self.heap.get_obj_instance(target_type_ref)?,
      invoke_type,
      args
    );

    let result_ref = self.call_method_handle(target_ref, &mut args)?;
    Ok(result_ref)
  }

  pub(crate) fn exec_invokedynamic(&mut self) -> Result<Option<types::Type>> {
    warn!("invokedynamic not fully implemented");

    let index = self.get_current_frame_mut()?.read_ju2()?;

    let _zero = self.get_current_frame_mut()?.read_ju2()?;

    let current_pc = self.get_current_frame_mut()?.get_pc();

    let curr_class_name = self.get_current_class()?.get_name().to_owned();

    let invoke_dynamic_resolved = self
      .class_loader
      .resolve_invokedynamic(&curr_class_name, index)?;

    debug!("{:?}", invoke_dynamic_resolved);

    // we need a factory
    // generate if it does not exist for this invokedynamic
    let callsite_ref = match self.lambdamanager.get_call_site(
      &curr_class_name,
      invoke_dynamic_resolved.get_method_name(),
      current_pc,
    ) {
      Some(v) => v,
      _ => {
        let callsite_ref =
          self.exec_invokedynamic_newcallsite(&curr_class_name, &invoke_dynamic_resolved)?;
        self.lambdamanager.add_call_site(
          &curr_class_name,
          invoke_dynamic_resolved.get_method_name(),
          current_pc,
          callsite_ref,
        );
        callsite_ref
      }
    };

    // use the factory to create the runnable instance
    let callsite = self.heap.get_obj_instance(callsite_ref)?.clone();
    let runnable_ref = match callsite.get_classname() {
      "java/lang/invoke/ConstantCallSite" => self.invoke_constant_callsite(&callsite)?,
      v => {
        return Err(eyre!(errors::InternalError::General(format!(
          "{} callsite not supported",
          v
        ))))
      }
    };

    // return the runnable instance
    self.push_stack(runnable_ref)?;
    Ok(None)
  }
}
