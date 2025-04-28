use crate::class_file;
use crate::class_loader::attributes;
use crate::runtime::modulemanager::ModuleManager;
use crate::runtime::{errors, types};
use crate::utils::ju2;
use color_eyre::eyre::{eyre, OptionExt, Result};
use log::{debug, warn};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use super::methods::MethodInfo;

#[derive(Debug)]
pub struct ClassLoader {
    pub(crate) modulemanager: ModuleManager,
    classes: HashMap<String, Arc<RwLock<class_file::ClassFile>>>,
}

impl ClassLoader {
    pub fn new() -> Self {
        ClassLoader {
            modulemanager: ModuleManager::new(),
            classes: HashMap::new(),
        }
    }

    pub fn add(&mut self, class_name: String) -> Result<()> {
        self.classes.insert(
            class_name.to_string(),
            class_file::ClassFile::parse(class_name + ".class")?,
        );
        Ok(())
    }

    pub(crate) fn load_class(&mut self, name: &str) -> Result<()> {
        let class = if name.starts_with("[") {
            class_file::ClassFile::create_array(name.to_string())?
        } else {
            // find module for the class
            let path = {
                let module = self.modulemanager.get_module_by_class(name)?;
                format!("{}/{}.class", module.get_name(), name)
            };
            class_file::ClassFile::parse(path)?
        };
        self.classes.insert(name.to_string(), class);
        Ok(())
    }

    pub fn get_method_max_locals_by_name(
        &mut self,
        class_name: &str,
        method_name: &str,
        type_str: &str,
    ) -> Result<u16> {
        let (_, code) = self.get_method_code_by_name(class_name, method_name, type_str)?;
        Ok(code.get_max_locals())
    }

    // returns class name where the method `method_name` is found and its info
    pub fn get_method_code_by_name(
        &mut self,
        class_name: &str,
        method_name: &str,
        type_str: &str,
    ) -> Result<(String, attributes::Code)> {
        let class = self.get(class_name)?;
        for i in 0..class.get_methods().len() {
            let method = class.get_methods().get(i);
            if method_name == method.get_name() && type_str == method.get_descriptor() {
                return Ok((
                    class_name.to_string(),
                    method
                        .get_code()
                        .ok_or_eyre(eyre!(errors::RuntimeError::GeneralException(&format!(
                            "{} {} {} code is empty",
                            class_name, method_name, type_str
                        ))
                        .to_string()))?
                        .clone(),
                ));
            }
        }
        if class.has_parent() {
            drop(class);
            let parent_name = self.get(class_name)?.get_parent_name().to_owned();
            self.get_method_code_by_name(&parent_name, method_name, type_str)
        } else {
            Err(eyre!(
                "function not found: {:?} {:?} in {:?}",
                method_name,
                type_str,
                class_name
            ))
        }
    }

    pub fn get_method_by_name(
        &mut self,
        class_name: &str,
        method_name: &str,
        type_str: &str,
    ) -> Result<(String, MethodInfo)> {
        let class = self.get(class_name)?;
        for i in 0..class.get_methods().len() {
            let method = class.get_methods().get(i);
            if method_name == method.get_name() && type_str == method.get_descriptor() {
                return Ok((class_name.to_string(), method.clone()));
            }
        }
        if class.has_parent() {
            drop(class);
            let parent_name = self.get(class_name)?.get_parent_name().to_owned();
            self.get_method_by_name(&parent_name, method_name, type_str)
        } else {
            Err(eyre!(
                "function not found: {:?} {:?} in {:?}",
                method_name,
                type_str,
                class_name
            ))
        }
    }

    // get method by name with index (for vtable)
    // if not in current class (maybe in super class)
    // the index is incremented by the number of methods in the current class
    pub fn get_method_by_name_with_index(
        &mut self,
        class_name: &str,
        method_name: &str,
        type_str: &str,
        _increment: usize,
    ) -> Result<(String, MethodInfo, i32)> {
        let class = self.get(class_name)?;
        let num_methods = class.get_methods().len();
        for i in 0..num_methods {
            let method = class.get_methods().get(i);
            if method_name == method.get_name() && type_str == method.get_descriptor() {
                return Ok((class_name.to_string(), method.clone(), i as i32));
            }
        }
        if class.has_parent() {
            drop(class);
            let parent_name = self.get(class_name)?.get_parent_name().to_owned();
            self.get_method_by_name_with_index(
                &parent_name,
                method_name,
                type_str,
                _increment + num_methods,
            )
        } else {
            Err(eyre!(
                "function not found: {:?} {:?} in {:?}",
                method_name,
                type_str,
                class_name
            ))
        }
    }

    // get field ref info as strings
    pub fn resolve_field_ref(
        &mut self,
        class_name: &str,
        index: ju2,
    ) -> Result<(String, String, String)> {
        let class = self.get(class_name)?;
        match class.resolve_field_ref(index) {
            a @ Ok(_) => a,
            _ => {
                if class.has_parent() {
                    drop(class);
                    let parent_name = self.get(class_name)?.get_parent_name().to_owned();
                    self.resolve_field_ref(&parent_name, index)
                } else {
                    Err(eyre!("method not found: {:?} in {:?}", index, class_name))
                }
            }
        }
    }

    pub fn resolve_method_ref(
        &mut self,
        class_name: &str,
        index: ju2,
    ) -> Result<(String, String, String)> {
        let class = self.get(class_name)?;
        match class.resolve_method_ref(index) {
            a @ Ok(_) => a,
            e => {
                if class.has_parent() {
                    drop(class);
                    let parent_name = self.get(class_name)?.get_parent_name().to_owned();
                    self.resolve_method_ref(&parent_name, index)
                } else {
                    Err(eyre!(
                        "method not found: {:?} in {:?} -> {:?}",
                        index,
                        class_name,
                        e
                    ))
                }
            }
        }
    }

    pub fn resolve_invokedynamic(
        &mut self,
        class_name: &str,
        index: ju2,
    ) -> Result<(String, String, String)> {
        let class = self.get(class_name)?;
        match class.resolve_invokedynamic(index) {
            a @ Ok(_) => a,
            e => {
                if class.has_parent() {
                    drop(class);
                    let parent_name = self.get(class_name)?.get_parent_name().to_owned();
                    self.resolve_invokedynamic(&parent_name, index)
                } else {
                    Err(eyre!(
                        "invokedynamic not found: {:?} in {:?} -> {:?}",
                        index,
                        class_name,
                        e
                    ))
                }
            }
        }
    }

    pub fn get(&mut self, name: &str) -> Result<RwLockReadGuard<class_file::ClassFile>> {
        if !self.classes.contains_key(name) {
            self.load_class(name)?;
        }
        Ok(self
            .classes
            .get(name)
            .ok_or_eyre(format!("class {:?} not found using get", name))?
            .read()
            .unwrap())
    }
    pub fn get_mut(&mut self, name: &str) -> Result<RwLockWriteGuard<class_file::ClassFile>> {
        if !self.classes.contains_key(name) {
            self.load_class(name)?;
        }
        Ok(self
            .classes
            .get_mut(name)
            .ok_or_eyre(format!("class {:?} not found using get_mut", name))?
            .write()
            .unwrap())
    }

    pub fn get_field_offset(&mut self, class_name: &str, field_name: &str) -> Result<i64> {
        let class = self.get(class_name)?;
        warn!("Super classes not checked properly in get_field_offset");
        let fields = class.get_fields().get_fields();
        for f in fields {
            debug!("{:?}", f.get_name())
        }
        match class.get_field_offset(field_name) {
            r @ Ok(_) => r,
            _ => {
                if class.has_parent() {
                    let parent_name = class.get_parent_name().to_string();
                    drop(class);
                    self.get_field_offset(&parent_name, field_name)
                } else {
                    panic!()
                }
            }
        }
    }

    pub fn get_field_by_offset(&mut self, class_name: &str, offset: i64) -> Result<String> {
        let class = self.get(class_name)?;
        warn!("Super classes not checked properly in get_field_by_offset");
        match class.get_field_by_offset(offset) {
            r @ Ok(_) => r,
            _ => {
                if class.has_parent() {
                    let parent_name = class.get_parent_name().to_string();
                    drop(class);
                    self.get_field_by_offset(&parent_name, offset)
                } else {
                    panic!("get field by offset")
                }
            }
        }
    }

    // does right implements left ?
    pub fn has_interface(&mut self, left: &str, right: &str) -> Result<bool> {
        if left == right {
            return Ok(true);
        }

        let right_class = self.get(right)?;
        let right_interfaces = right_class.get_interfaces().clone();
        drop(right_class);
        for interface in right_interfaces {
            if self.has_interface(left, &interface)? {
                return Ok(true);
            }
        }

        let right_class = self.get(right)?;
        if right_class.has_parent() {
            drop(right_class);
            let parent_name = self.get(right)?.get_parent_name().to_owned();
            self.has_interface(left, &parent_name)
        } else {
            Ok(false)
        }
    }

    pub fn is_method_native(
        &mut self,
        class_name: &str,
        method_name: &str,
        method_type: &str,
    ) -> Result<bool> {
        let class = self.get(class_name)?;
        let methods = class.get_methods();
        match methods.get_by_name(method_name, method_type) {
            Some(method) if method.is_native() => return Ok(true),
            _ => {}
        };

        if class.has_parent() {
            drop(class);
            let parent_name = self.get(class_name)?.get_parent_name().to_owned();
            self.is_method_native(&parent_name, method_name, method_type)
        } else {
            Ok(false)
        }
    }

    pub fn get_static_field(&mut self, class_name: &str, field_name: &str) -> Result<types::Type> {
        let class = self.get(class_name)?;
        match class.get_static_field(field_name) {
            Ok(v) => Ok(*v),
            _ => {
                if class.has_parent() {
                    drop(class);
                    let parent_name = self.get(class_name)?.get_parent_name().to_owned();
                    self.get_static_field(&parent_name, field_name)
                } else {
                    Err(eyre!(
                        "static field not found: {:?} in {:?}",
                        field_name,
                        class_name
                    ))
                }
            }
        }
    }
}
