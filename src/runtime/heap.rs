use crate::class_loader::{class_file, methods};
use crate::runtime::errors;
use crate::utils::*;
use crate::{class_loader::loader::ClassLoader, runtime::types};
use color_eyre::eyre::{eyre, OptionExt, Result};
use core::panic;
use log::{debug, warn};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use super::types::{ArrayInstance, Type};

#[derive(Debug)]
pub struct Heap {
    next_obj_ref: ju4, // 0 is used for Null
    //next_array_ref: ju4,
    heap: HashMap<ju4, types::Instance>,
    //arrays: HashMap<ju4, types::ArrayInstance>,
    classes: HashMap<String, ju4>,
    strings: HashMap<String, ju4>,
}

impl fmt::Display for Heap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Heap: ")?;
        for (k, v) in &self.heap {
            write!(
                f,
                "\n\t{} : {:?} {:?} {:?}",
                k,
                v.get_classname(),
                v.get_ref(),
                v.get_hash_code()
            )?;
            if let types::Instance::ObjectInstance(obj) = v {
                if v.get_classname() == "java/lang/String" {
                    let value = obj.get_field("value").unwrap();
                    if let types::Type::ArrayRef(array_ref) = value {
                        let array = self.get_array_instance(array_ref).unwrap();
                        write!(f, " -> {:?}", array.get_string())?;
                    }
                }
            }
        }
        //write!(f, "\nArrays : ")?;
        //for (k, v) in &self.arrays {
        //    write!(f, "\n\t{} : {:?}", k, v.get_classname())?;
        //}
        //write!(f, "\nClasses: ")?;
        //for (k, v) in &self.classes {
        //    write!(f, "\n\t{} : {}", k, v)?;
        //}
        //write!(f, "\nStrings: ")?;
        //for (k, v) in &self.strings {
        //    write!(f, "\n\t{} : {}", k, v)?;
        //}
        Ok(())
    }
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            next_obj_ref: 1,
            //next_array_ref: 0,
            heap: HashMap::new(),
            //arrays: HashMap::new(),
            classes: HashMap::new(),
            strings: HashMap::new(),
        }
    }

    /// Allocate a string
    ///
    /// # Arguments
    ///
    /// * `value` - String to allocate
    ///
    /// # Returns
    ///
    /// A `ju4` representing the object reference in the heap
    pub fn alloc_string(&mut self, loader: &mut ClassLoader, value: &str) -> Result<types::Type> {
        // strings are immutable
        if self.strings.contains_key(value) {
            debug!("ALREADY EXISTS {:?}", value);
            return Ok(types::Type::ObjectRef(
                *self
                    .strings
                    .get(value)
                    .ok_or_eyre(eyre!("something went wrong in alloc string"))?,
            ));
        }

        let arr: Vec<types::Type> = value.bytes().map(|c| types::Type::Byte(c as i8)).collect();
        let arr_len = arr.len();
        // TODO: is this the correct way to create a string ?
        // TODO: is alloc string used only to create a class obj ?
        let str_ref = self.alloc_array_primitive("B", arr, arr_len)?;

        let string_ref = self.alloc_obj(loader, "java/lang/String")?;
        let (string_obj, curr_ref) = match string_ref {
            types::Type::ObjectRef(obj_ref) => (self.get_obj_instance_mut(obj_ref)?, obj_ref),
            _ => panic!(),
        };
        string_obj.new_field("value", str_ref)?;
        self.strings.insert(value.to_string(), curr_ref);

        Ok(types::Type::ObjectRef(curr_ref))
    }

    pub fn get_curr_obj_ref(&mut self) -> ju4 {
        self.next_obj_ref
    }
    pub fn get_next_obj_ref(&mut self) -> ju4 {
        let curr_ref = self.next_obj_ref;
        self.next_obj_ref += 1;
        curr_ref
    }

    /// Allocate an object
    ///
    /// # Arguments
    ///
    /// * `loader` - ClassLoader used to resolve the class to instantiate
    /// * `class_name` - Class to instantiate
    ///
    /// # Returns
    ///
    /// A `ObjectRef` of the newly instantiated object
    pub fn alloc_obj(&mut self, loader: &mut ClassLoader, class_name: &str) -> Result<types::Type> {
        // TODO: not sure about how I create a new object
        let curr_ref = self.get_next_obj_ref();

        // TODO: resolve inheritance
        let class = loader.get(class_name)?;
        let mut new_obj = class.new_obj(curr_ref)?;

        let mut parent_class_name = class.get_name().to_owned();
        drop(class);

        let mut parents = vec![];

        // go up until hit the base
        while parent_class_name != "java/lang/Object" {
            parent_class_name = {
                let class = loader.get(&parent_class_name)?;
                class.get_parent_name().to_owned()
            };

            let parent_ref = self.alloc_obj(loader, &parent_class_name)?.as_ref()?;
            let parent_obj = self.get_obj_instance(parent_ref)?;
            parents.push(Arc::new(RwLock::new(parent_obj.clone())));
        }
        // we might be java/lang/Object
        if !parents.is_empty() {
            new_obj.set_parent(parents[0].clone());
            for i in 0..parents.len() - 1 {
                let cp = parents[i + 1].clone();
                parents[i].write().unwrap().set_parent(cp);
            }
        }

        self.heap
            .insert(curr_ref, types::Instance::ObjectInstance(new_obj));
        Ok(types::Type::ObjectRef(curr_ref))
    }

    pub fn get_component_type(
        &mut self,
        loader: &mut ClassLoader,
        class_name: &str,
    ) -> Result<types::Type> {
        let comp_type = types::Type::convert_array_descriptor_to_class_type(class_name)?;
        // TODO: force load it
        if !self.has_class_instance(&comp_type) {
            self.alloc_class_obj(loader, &comp_type)?;
        }
        if !types::Type::is_primitive(&comp_type) {
            let v = loader.get(&comp_type)?;
            drop(v);
        }
        let comp_class = self.get_class_instance(&comp_type)?;
        Ok(types::Type::ObjectRef(comp_class.get_obj_ref()))
    }

    /// Allocate a Class<T> object
    ///
    /// # Arguments
    ///
    /// * `loader` - ClassLoader used to resolve the class to instantiate
    /// * `class_name` - Class to instantiate
    ///
    /// # Returns
    ///
    /// A `ObjectRef` of the newly instantiated object
    pub fn alloc_class_obj(
        &mut self,
        loader: &mut ClassLoader,
        class_name: &str,
    ) -> Result<types::Type> {
        if self.classes.contains_key(class_name) {
            return Ok(types::Type::ObjectRef(
                *self
                    .classes
                    .get(class_name)
                    .ok_or_eyre(eyre!("something went wrong in alloc class obj"))?,
            ));
        }

        let mut fields = vec![];
        // adding component type information if array
        if class_name.starts_with("[") {
            let comp_type = self.get_component_type(loader, class_name)?;
            fields.push(("componentType", comp_type));
        }

        let string = self.alloc_string(loader, class_name)?;
        fields.push(("name", string));

        let obj_ref = self.alloc_obj(loader, "java/lang/Class")?.as_ref()?;

        let obj_mod = self.get_obj_instance_mut(obj_ref)?;
        for f in fields {
            obj_mod.new_field(f.0, f.1)?;
        }

        // TODO: Check which ClassLoader is loading it.
        // TODO: if part of the bootstrap classLoader is NULL
        obj_mod.put_field("classLoader", types::Type::Null)?;

        if !Type::is_primitive(class_name) && !class_name.starts_with("[") {
            let module = loader.modulemanager.get_module_by_class(class_name)?;
            obj_mod.put_field("module", types::Type::ObjectRef(module.get_obj_ref()))?;
        }

        self.classes.insert(class_name.to_string(), obj_ref);

        Ok(types::Type::ObjectRef(obj_ref))
    }

    /// Allocate a Class<T> object for a primitive type
    ///
    /// # Arguments
    ///
    /// * `loader` - ClassLoader used to resolve the class to instantiate
    /// * `class_name` - Class to instantiate
    ///
    /// # Returns
    ///
    /// A `ObjectRef` of the newly instantiated object
    pub fn alloc_primitive_class_obj(
        &mut self,
        loader: &mut ClassLoader,
        class_name: &str,
    ) -> Result<types::Type> {
        if self.classes.contains_key(class_name) {
            return Ok(types::Type::ObjectRef(
                *self
                    .classes
                    .get(class_name)
                    .ok_or_eyre(eyre!("something went wrong in alloc class obj"))?,
            ));
        }

        let name_ref = self.alloc_string(loader, class_name)?;

        let obj_ref = self.alloc_obj(loader, "java/lang/Class")?.as_ref()?;

        let obj_mod = self.get_obj_instance_mut(obj_ref)?;
        obj_mod.new_field("name", name_ref)?;

        // TODO: Check which ClassLoader is loading it.
        // TODO: if part of the bootstrap classLoader is NULL
        obj_mod.put_field("classLoader", types::Type::Null)?;
        // using something that is for sure in java.base
        let module = loader
            .modulemanager
            .get_module_by_class("java/lang/Integer")?;
        obj_mod.put_field("module", types::Type::ObjectRef(module.get_obj_ref()))?;

        self.classes.insert(class_name.to_string(), obj_ref);

        Ok(types::Type::ObjectRef(obj_ref))
    }

    /// Allocate an array of non primitive types
    ///
    /// # Arguments
    ///
    /// * `loader` - ClassLoader used to resolve the class to instantiate
    /// * `class_name` - Class of elements to instantiate
    ///
    /// # Returns
    ///
    /// A `ObjectRef` of the newly instantiated array
    pub fn alloc_array(
        &mut self,
        class_name: &str,
        array: Vec<types::Type>,
        size: usize, // size is used when array is empty -> fill in default values
    ) -> Result<types::Type> {
        let curr_array_ref = self.get_next_obj_ref();

        //let array_len = array.len();
        //let mut elements = array;
        //if array_len == 0 {
        //    for _ in 0..size {
        //        elements.push(types::Type::Null);
        //    }
        //}
        let array_len = array.len();
        let mut elements = array;
        if array_len == 0 {
            let def_value = get_default_value(class_name);
            for _ in 0..size {
                elements.push(def_value);
            }
        }
        let array = ArrayInstance::new(class_name, curr_array_ref, elements)?;

        //self.arrays.insert(curr_array_ref, array);
        self.heap
            .insert(curr_array_ref, types::Instance::ArrayInstance(array));

        Ok(types::Type::ArrayRef(curr_array_ref))
    }

    /// Allocate an array of primitive types
    ///
    /// # Arguments
    ///
    /// * `loader` - ClassLoader used to resolve the class to instantiate
    /// * `class_name` - Class of elements to instantiate
    ///
    /// # Returns
    ///
    /// A `ObjectRef` of the newly instantiated array
    pub fn alloc_array_primitive(
        &mut self,
        class_name: &str,
        array: Vec<types::Type>,
        size: usize, // size is used when array is empty -> fill in default values
    ) -> Result<types::Type> {
        let curr_array_ref = self.get_next_obj_ref();

        let array_len = array.len();
        let mut elements = array;
        if array_len == 0 {
            let def_value = get_default_value(class_name);
            for _ in 0..size {
                elements.push(def_value);
            }
        }
        let array = ArrayInstance::new(class_name, curr_array_ref, elements)?;

        //self.arrays.insert(curr_array_ref, array);
        self.heap
            .insert(curr_array_ref, types::Instance::ArrayInstance(array));

        Ok(types::Type::ArrayRef(curr_array_ref))
    }

    /// Allocate an multi dimenstional array
    ///
    /// # Arguments
    ///
    /// * `loader` - ClassLoader used to resolve the class to instantiate
    /// * `class_name` - Class of elements to instantiate
    ///
    /// # Returns
    ///
    /// A `ObjectRef` of the newly instantiated array
    pub fn alloc_multiarray(
        &mut self,
        class_name: &str,
        dims: &[usize], // size is used when array is empty -> fill in default values
    ) -> Result<types::Type> {
        let curr_array_ref = self.get_next_obj_ref();

        assert!(!dims.is_empty());

        if dims.len() == 1 {
            return self.alloc_array(class_name, vec![], dims[0]);
        }

        // multi dimension

        let first_dim = dims[0];
        let rest_dims = &dims[1..];

        // Allocate top-level array
        let mut elements = Vec::with_capacity(first_dim);

        for _ in 0..first_dim {
            // Recursive: allocate sub-arrays
            let element = self.alloc_multiarray(class_name, rest_dims)?;
            elements.push(element);
        }

        let array = ArrayInstance::new(class_name, curr_array_ref, elements)?;

        self.heap
            .insert(curr_array_ref, types::Instance::ArrayInstance(array));

        Ok(types::Type::ArrayRef(curr_array_ref))
    }

    pub fn get_instance(&self, obj_ref: ju4) -> Result<&types::Instance> {
        self.heap
            .get(&obj_ref)
            .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))
    }
    pub fn get_instance_mut(&mut self, obj_ref: ju4) -> Result<&mut types::Instance> {
        self.heap
            .get_mut(&obj_ref)
            .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))
    }
    pub fn get_obj_instance(&self, obj_ref: ju4) -> Result<&types::ObjectInstance> {
        let obj_enum = self
            .heap
            .get(&obj_ref)
            .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))?;
        if let types::Instance::ObjectInstance(obj) = obj_enum {
            return Ok(obj);
        }
        Err(eyre!(errors::RuntimeError::WrongInstance(
            "ObjectInstance",
            obj_enum.clone()
        )))
    }
    pub fn get_obj_instance_mut(&mut self, obj_ref: ju4) -> Result<&mut types::ObjectInstance> {
        let obj_enum = self
            .heap
            .get_mut(&obj_ref)
            .ok_or_eyre(format!("ObjRef not found in heap: {:?}", obj_ref))?;
        if let types::Instance::ObjectInstance(obj) = obj_enum {
            return Ok(obj);
        }
        Err(eyre!(errors::RuntimeError::WrongInstance(
            "ObjectInstance",
            obj_enum.clone()
        )))
    }

    pub fn get_array_instance(&self, obj_ref: ju4) -> Result<&types::ArrayInstance> {
        let obj_enum = self
            .heap
            .get(&obj_ref)
            .ok_or_eyre(format!("ArrayRef not found in heap: {:?}", obj_ref))?;
        if let types::Instance::ArrayInstance(obj) = obj_enum {
            return Ok(obj);
        }
        Err(eyre!(errors::RuntimeError::WrongInstance(
            "ArrayInstance",
            obj_enum.clone()
        )))
    }
    pub fn get_array_instance_mut(&mut self, obj_ref: ju4) -> Result<&mut types::ArrayInstance> {
        let obj_enum = self
            .heap
            .get_mut(&obj_ref)
            .ok_or_eyre(format!("ArrayRef not found in heap: {:?}", obj_ref))?;
        if let types::Instance::ArrayInstance(obj) = obj_enum {
            return Ok(obj);
        }
        Err(eyre!(errors::RuntimeError::WrongInstance(
            "ArrayInstance",
            obj_enum.clone()
        )))
    }

    pub fn get_string(&self, obj_ref: ju4) -> Result<String> {
        let obj = self.get_obj_instance(obj_ref)?;
        assert!(obj.get_classname() == "java/lang/String");
        let array_field = obj.get_field("value")?;
        if let types::Type::ArrayRef(array_ref) = array_field {
            let array = self.get_array_instance(array_ref)?;
            //// this is the name of T class in Class<T>
            let name = array.get_string()?;
            return Ok(name);
        }

        Err(eyre!(errors::RuntimeError::WrongType("Array", array_field)))
    }

    pub fn get_class_from_class_obj<'a>(
        &'a self,
        loader: &'a mut ClassLoader,
        class_ref: ju4,
    ) -> Result<RwLockReadGuard<'a, class_file::ClassFile>> {
        let class_obj = self.get_obj_instance(class_ref)?;
        let class_inner_name_ref = class_obj.get_field("name")?.as_ref()?; // T in Class<T> as string
        let class_inner_name = self.get_string(class_inner_name_ref)?;
        loader.get(&class_inner_name)
    }

    pub fn get_class_instance(&self, class_name: &str) -> Result<&types::ObjectInstance> {
        let class_ref = self
            .classes
            .get(class_name)
            .ok_or_eyre(format!("ClassObject not found in heap: {:?}", class_name))?;
        self.get_obj_instance(*class_ref)
    }
    pub fn has_class_instance(&self, class_name: &str) -> bool {
        self.get_class_instance(class_name).is_ok()
    }

    pub fn clone_object(&mut self, obj_ref: ju4) -> Result<types::Type> {
        let curr_ref = self.get_next_obj_ref();
        let instance = self.get_instance(obj_ref)?;
        match instance {
            types::Instance::ObjectInstance(obj) => {
                let cp = obj.clone();
                self.heap
                    .insert(curr_ref, types::Instance::ObjectInstance(cp));
                Ok(types::Type::ObjectRef(curr_ref))
            }
            types::Instance::ArrayInstance(arr) => {
                let cp = arr.clone();
                self.heap
                    .insert(curr_ref, types::Instance::ArrayInstance(cp));
                Ok(types::Type::ArrayRef(curr_ref))
            }
        }
    }

    pub fn alloc_reflect_method(
        &mut self,
        loader: &mut ClassLoader,
        declaring_class: &str,
        method: &methods::MethodInfo,
    ) -> Result<types::Type> {
        warn!("reflect method not fully implemented");
        // TODO: add propert types for arguments
        let param_types = method.get_descriptor();
        let signature = self.alloc_string(loader, method.get_descriptor())?;
        let ret_type = get_return_type_descriptor(param_types);
        if !self.has_class_instance(&ret_type) {
            debug!("ALLOC {}", ret_type);
            self.alloc_class_obj(loader, &ret_type)?;
        }
        let ret_class_obj_ref = self.get_class_instance(&ret_type)?.get_obj_ref();
        let param_types = get_argument_class_names(param_types).unwrap();
        let mut param_types_args = vec![];
        for p in param_types {
            debug!("{}", p);
            let p = &descriptor_to_class_name(&p);
            if !self.has_class_instance(p) {
                self.alloc_class_obj(loader, p)?;
            }
            let class = self.get_class_instance(p)?;
            param_types_args.push(types::Type::ObjectRef(class.get_obj_ref()));
        }
        let param_types_args_len = param_types_args.len();
        let param_array =
            self.alloc_array("[Ljava/lang/Class;", param_types_args, param_types_args_len)?;

        // Allocate a new java/lang/reflect/Method instance
        let method_ref = self.alloc_obj(loader, "java/lang/reflect/Method")?;

        // clazz: reference to Class object
        let class_obj_ref = self.get_class_instance(declaring_class)?.get_obj_ref();
        let name_string = self.alloc_string(loader, method.get_name())?;

        let method_instance = self.get_obj_instance_mut(method_ref.as_ref()?)?;
        method_instance.new_field("clazz", types::Type::ObjectRef(class_obj_ref))?;

        // name: java/lang/String
        method_instance.new_field("name", name_string)?;

        // modifiers: int
        method_instance.new_field(
            "modifiers",
            types::Type::Integer(*method.get_access_flags() as i32),
        )?;

        method_instance.new_field("parameterTypes", param_array)?;
        method_instance.new_field("signature", signature)?;

        // returnType: java/lang/Class
        method_instance.new_field("returnType", types::Type::ObjectRef(ret_class_obj_ref))?;

        Ok(method_ref)
    }

    pub(crate) fn alloc_integer(
        &mut self,
        loader: &mut ClassLoader,
        num: i32,
    ) -> Result<types::Type> {
        let obj_ref = self.alloc_obj(loader, "java/lang/Integer")?.as_ref()?;
        let obj = self.get_obj_instance_mut(obj_ref)?;
        obj.put_field("value", types::Type::Integer(num))?;
        Ok(types::Type::ObjectRef(obj_ref))
    }
}
