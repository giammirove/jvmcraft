use std::collections::HashMap;

use color_eyre::eyre::{eyre, OptionExt, Result};

use crate::utils::ju4;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Module {
    obj_ref: ju4, // reference of the module in the heap
    is_open: bool,
    version: Option<String>,
    name: String,
    packages: Vec<String>,

    read: Vec<ju4>, // list of references to readable modules

    exports_all: Vec<String>, // list of packages exported to all

    exports: HashMap<ju4, Vec<String>>, // list of packages exported to a module
}

impl Module {
    pub(crate) fn add_read(&mut self, mod_ref: ju4) {
        if !self.has_read(&mod_ref) {
            self.read.push(mod_ref)
        }
    }

    pub(crate) fn has_read(&mut self, mod_ref: &ju4) -> bool {
        self.read.contains(mod_ref)
    }

    pub(crate) fn has_export_all(&mut self, exp: &String) -> bool {
        self.exports_all.contains(exp)
    }

    pub(crate) fn add_export_all(&mut self, exp: String) {
        if !self.has_export_all(&exp) {
            self.exports_all.push(exp)
        }
    }

    pub(crate) fn add_export_to_module(&mut self, mod_ref: ju4, exp: String) {
        self.exports.entry(mod_ref).or_default();
        let packages = self
            .exports
            .get_mut(&mod_ref)
            .ok_or_eyre(eyre!("module not found"))
            .unwrap();
        if !packages.contains(&exp) {
            packages.push(exp);
        }
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_obj_ref(&self) -> ju4 {
        self.obj_ref
    }
}

#[derive(Debug)]
pub(crate) struct ModuleManager {
    modules: HashMap<ju4, Module>, // key = module ref , value = module

    rev_mapping: HashMap<String, ju4>, // key = class name , value module ref
}

impl ModuleManager {
    pub(crate) fn new() -> Self {
        ModuleManager {
            modules: HashMap::new(),
            rev_mapping: HashMap::new(),
        }
    }

    pub(crate) fn add(
        &mut self,
        obj_ref: ju4,
        is_open: bool,
        version: Option<String>,
        location: String,
        packages: Vec<String>,
    ) {
        for p in &packages {
            self.rev_mapping.insert(p.to_string(), obj_ref);
        }
        self.modules.insert(
            obj_ref,
            Module {
                obj_ref,
                is_open,
                version,
                name: location,
                packages,
                read: vec![],
                exports_all: vec![],
                exports: HashMap::new(),
            },
        );
    }

    pub(crate) fn get(&self, mod_ref: ju4) -> Result<&Module> {
        self.modules
            .get(&mod_ref)
            .ok_or_eyre(eyre!("module not found {}", mod_ref))
    }

    pub(crate) fn get_module_by_class(&self, class_name: &str) -> Result<&Module> {
        let mod_ref = self
            .rev_mapping
            .get(class_name)
            .ok_or_eyre(eyre!("module not found for class {}", class_name))?;
        self.get(*mod_ref)
    }

    pub(crate) fn get_mut(&mut self, mod_ref: ju4) -> Result<&mut Module> {
        self.modules
            .get_mut(&mod_ref)
            .ok_or_eyre(eyre!("module not found {}", mod_ref))
    }
}
