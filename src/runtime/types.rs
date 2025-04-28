use crate::class_loader::loader::ClassLoader;
use crate::runtime::*;
use crate::utils::*;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::{eyre, Result};
use heap::Heap;
use log::error;
use log::warn;
use log::{debug, info};
use std::collections::HashMap;
use std::fmt;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem::discriminant;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type {
    None,
    _Unitialized,
    Null,
    Byte(i8),
    Boolean(bool),
    Character(i8),
    Short(i16),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),

    ArrayRef(ju4),
    ObjectRef(ju4),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Type {
    pub fn get_category(&self) -> u8 {
        match self {
            Type::Byte(_) => 1,
            Type::Boolean(_) => 1,
            Type::Character(_) => 1,
            Type::Short(_) => 1,
            Type::Integer(_) => 1,
            Type::Float(_) => 1,
            Type::Long(_) => 2,
            Type::Double(_) => 2,
            Type::Null => 1,
            Type::ArrayRef(_) => 1,
            Type::ObjectRef(_) => 1,
            v => panic!("{:?}", v),
        }
    }

    pub fn as_float(&self) -> Result<f32> {
        let value: f32 = match self {
            Type::Float(l) => *l,
            v => return Err(eyre!(errors::RuntimeError::WrongType("Number", *v))),
        };
        Ok(value)
    }

    pub fn as_short(&self) -> Result<i16> {
        let value: i16 = match self {
            Type::Byte(b) => *b as i16,
            Type::Boolean(b) => *b as i16,
            Type::Character(c) => *c as i16,
            Type::Short(s) => *s,
            Type::Integer(i) => *i as i16,
            Type::Long(l) => *l as i16,
            v => return Err(eyre!(errors::RuntimeError::WrongType("Number", *v))),
        };
        Ok(value)
    }

    pub fn as_byte(&self) -> Result<i8> {
        let value = match self {
            Type::Byte(b) => *b,
            Type::Boolean(b) => *b as i8,
            Type::Character(c) => *c,
            Type::Short(s) => *s as i8,
            Type::Integer(i) => *i as i8,
            Type::Long(l) => *l as i8,
            v => return Err(eyre!(errors::RuntimeError::WrongType("Byte", *v))),
        };
        Ok(value)
    }

    pub fn as_integer(&self) -> Result<i32> {
        let value: i32 = match self {
            Type::Byte(b) => *b as i32,
            Type::Boolean(b) => *b as i32,
            Type::Character(c) => *c as i32,
            Type::Short(s) => *s as i32,
            Type::Integer(i) => *i,
            Type::Long(l) => *l as i32,
            v => return Err(eyre!(errors::RuntimeError::WrongType("Integer", *v))),
        };
        Ok(value)
    }

    pub fn as_long(&self) -> Result<i64> {
        let value: i64 = match self {
            Type::Byte(b) => *b as i64,
            Type::Boolean(b) => *b as i64,
            Type::Character(c) => *c as i64,
            Type::Short(s) => *s as i64,
            Type::Integer(i) => *i as i64,
            Type::Long(l) => *l,
            v => return Err(eyre!(errors::RuntimeError::WrongType("Number", *v))),
        };
        Ok(value)
    }

    pub fn as_ref(&self) -> Result<ju4> {
        let value: ju4 = match self {
            Type::ObjectRef(b) => *b as ju4,
            Type::ArrayRef(b) => *b as ju4,
            v => return Err(eyre!(errors::RuntimeError::WrongType("Number", *v))),
        };
        Ok(value)
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            types::Type::Integer(_)
                | types::Type::Short(_)
                | types::Type::Byte(_)
                | types::Type::Character(_)
                | types::Type::Boolean(_)
        )
    }

    pub fn field_type_to_type(field_type: &str) -> String {
        if field_type.starts_with("L") {
            let size = field_type.len();
            return field_type[1..size - 1].to_string();
        }

        field_type.to_string()
    }

    pub fn get_type(&self, heap: &Heap) -> String {
        match self {
            Type::Byte(_) => "B".to_string(),
            Type::Boolean(_) => "Z".to_string(),
            Type::Character(_) => "C".to_string(),
            Type::Short(_) => "S".to_string(),
            Type::Integer(_) => "I".to_string(),
            Type::Float(_) => "F".to_string(),
            Type::Long(_) => "J".to_string(),
            Type::Double(_) => "D".to_string(),
            Type::Null => "Null".to_string(),
            Type::ArrayRef(array_ref) => heap
                .get_array_instance(*array_ref)
                .unwrap()
                .get_classname()
                .to_string(),
            Type::ObjectRef(obj_ref) => format!(
                "L{};",
                heap.get_obj_instance(*obj_ref).unwrap().get_classname()
            ),
            v => panic!("{:?}", v),
        }
    }

    pub fn is_primitive(type_str: &str) -> bool {
        type_str == "F"
            || type_str == "D"
            || type_str == "Z"
            || type_str == "C"
            || type_str == "I"
            || type_str == "B"
            || type_str == "S"
            || type_str == "J"
    }
    fn is_class_integer(type_str: &str) -> bool {
        type_str == "B"
            || type_str == "Z"
            || type_str == "C"
            || type_str == "S"
            || type_str == "I"
            || type_str == "J"
            || type_str == "java/lang/Boolean"
            || type_str == "java/lang/Character"
            || type_str == "java/lang/Byte"
            || type_str == "java/lang/Short"
            || type_str == "java/lang/Integer"
            || type_str == "java/lang/Long"
    }

    // input are like [B, [Z, [Ljava/lang/String']
    pub fn convert_array_descriptor_to_class_type(descriptor: &str) -> Result<String> {
        warn!("Multi array not supported");
        let element_type = descriptor.trim_start_matches('[');

        match element_type {
            v if Type::is_primitive(v) => Ok(element_type.to_owned()),
            t if t.starts_with('L') && t.ends_with(';') => {
                // [Ljava/lang/String; → java/lang/String
                Ok(t[1..t.len() - 1].into())
            }
            _ => panic!(),
        }
    }

    // order is important becase
    // if B <= A then
    //     A := B is ok
    //     B := A is wrong
    //  left = right is ok
    pub fn check_type(loader: &mut ClassLoader, left: &str, right: &str) -> Result<bool> {
        if left == right {
            return Ok(true);
        }
        if right == "Null" && !Type::is_primitive(left) {
            return Ok(true);
        }
        if left == "Ljava/lang/Object;" || right == "Ljava/lang/Object;" {
            return Ok(true);
        }
        if left == "java/lang/Object" || right == "java/lang/Object" {
            return Ok(true);
        }

        if left.starts_with("[") && right.starts_with("[") {
            warn!("Not handled : {:?} = {:?}", left, right);
            return Ok(true);
        }

        if left.starts_with("[") && !right.starts_with("[") {
            warn!("Not handled : {:?} = {:?}", left, right);
            return Ok(false);
        }

        if !left.starts_with("[") && right.starts_with("[") {
            warn!("Not handled : {:?} = {:?}", left, right);
            return Ok(false);
        }

        if Type::is_class_integer(left) && Type::is_class_integer(right) {
            return Ok(true);
        }
        if Type::is_subtype(loader, left, right)? {
            return Ok(true);
        }
        if Type::implements_interface(loader, left, right)? {
            return Ok(true);
        }
        debug!("Not possible : {:?} = {:?}", left, right);
        // check for parents
        Ok(false)
    }

    // is right a subtype of left ? right <= left ?
    pub fn is_subtype(loader: &mut ClassLoader, left: &str, right: &str) -> Result<bool> {
        if left.starts_with("L") || right.starts_with("L") {
            warn!(
                "SUBTYPE FOR FIELD TYPE NOT IMPLEMENTED YET {:?} {:?}",
                left, right
            );
            return Ok(true);
        }

        if left == "java/lang/Object" || left == right {
            return Ok(true);
        }

        let mut parent_class_name = right.to_string();
        while parent_class_name != *"java/lang/Object" {
            parent_class_name = {
                let class = loader.get(&parent_class_name)?;
                class.get_parent_name().to_owned()
            };
            if parent_class_name == left {
                return Ok(true);
            }
        }

        Ok(false)
    }

    // does right implement left ?
    pub fn implements_interface(loader: &mut ClassLoader, left: &str, right: &str) -> Result<bool> {
        debug!("Check if {:?} implements {:?}", right, left);
        if left.starts_with("L") || right.starts_with("L") {
            warn!(
                "INTERFACE FIELD TYPE NOT IMPLEMENTED YET {:?} {:?}",
                left, right
            );
            return Ok(true);
        }

        loader.has_interface(left, right)
    }
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    class_name: String,
    obj_ref: ju4,
    hash_code: ju4,
    fields: HashMap<String, types::Type>,
    fields_offset: HashMap<String, i64>,
    rev_fields_offset: HashMap<i64, String>,
    interfaces: Vec<String>,

    parent_obj: Option<Arc<RwLock<ObjectInstance>>>,
    monitor_count: ju4,
}

impl fmt::Display for ObjectInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "ObjectInstace {:?} {:?} {:?}",
            self.class_name, self.obj_ref, self.hash_code
        )?;
        Ok(())
    }
}

impl ObjectInstance {
    pub fn new(class_name: String, obj_ref: ju4) -> Result<ObjectInstance> {
        let nonce = generate_nonce();
        // Hash the reference to the object (memory address)
        let str_to_hash = format!("{}-{}-{}", class_name, obj_ref, nonce);
        let mut hasher = DefaultHasher::new();
        str_to_hash.hash(&mut hasher);
        let hash_code = hasher.finish() as ju4;

        let fields = HashMap::new();
        let fields_offset = HashMap::new();
        let rev_fields_offset = HashMap::new();
        let interfaces = vec![];
        Ok(ObjectInstance {
            class_name: class_name.to_string(),
            obj_ref,
            hash_code,
            fields,
            fields_offset,
            rev_fields_offset,
            interfaces,
            parent_obj: None,
            monitor_count: 0,
        })
    }

    pub fn is_class(&self) -> bool {
        self.get_classname() == "java/lang/Class"
    }
    pub fn is_string(&self) -> bool {
        self.get_classname() == "java/lang/String"
    }

    pub fn is_primitive(&self) -> bool {
        types::Type::is_primitive(self.get_classname())
    }

    pub fn get_hash_code(&self) -> ju4 {
        self.hash_code
    }

    pub fn set_parent(&mut self, parent: Arc<RwLock<ObjectInstance>>) {
        if parent.read().unwrap().get_obj_ref() == 0 {
            self.parent_obj = None;
            debug!("{}", self.get_classname());
            panic!()
        } else {
            self.parent_obj = Some(parent)
        }
    }
    pub fn get_parent(&self) -> Option<Arc<RwLock<ObjectInstance>>> {
        self.parent_obj.as_ref().map(|v| v.clone())
    }

    pub fn add_interface(&mut self, interfacename: &str) {
        self.interfaces.push(interfacename.to_string());
    }
    pub fn _has_interface(&self, interfacename: &str) -> bool {
        for interface in &self.interfaces {
            if interface == interfacename {
                return true;
            }
        }
        false
    }

    // TODO: a way to not clone ?
    pub fn get_field(&self, fieldname: &str) -> Result<types::Type> {
        if fieldname == "jfrTracing" {
            return Ok(types::Type::Boolean(false));
        }

        match self.fields.get(fieldname) {
            Some(v) => Ok(*v),
            _ => {
                // maybe it's a field of the parent class
                let mut parent = self.get_parent();
                loop {
                    match parent {
                        None => break,
                        Some(parent_obj) => {
                            let parent_read = parent_obj.read().unwrap();
                            if let Ok(res) = parent_read.get_field(fieldname) {
                                return Ok(res);
                            } else {
                                parent = parent_read.get_parent();
                            }
                        }
                    }
                }
                Err(eyre!("field not found: {:?}", fieldname))
            }
        }
    }

    pub fn put_field(&mut self, fieldname: &str, fieldvalue: types::Type) -> Result<()> {
        match self.fields.get(fieldname) {
            Some(_) => self.fields.insert(fieldname.to_string(), fieldvalue),
            _ => {
                // maybe it's a field of the parent class
                let mut parent = self.get_parent();
                loop {
                    match parent {
                        None => break,
                        Some(parent_obj) => {
                            let mut parent_write = parent_obj.write().unwrap();
                            if parent_write.get_field(fieldname).is_ok() {
                                parent_write
                                    .fields
                                    .insert(fieldname.to_string(), fieldvalue);
                                return Ok(());
                            } else {
                                parent = parent_write.get_parent();
                            }
                        }
                    }
                }
                return Err(eyre!(
                    "field not found: {:?} in {:?}",
                    fieldname,
                    self.class_name
                ));
            }
        };
        Ok(())
    }

    // create new field in current instance
    // TODO: check it exists
    pub fn new_field(&mut self, fieldname: &str, fieldvalue: types::Type) -> Result<()> {
        let n = self.fields.len() as i64;
        self.fields.insert(fieldname.to_string(), fieldvalue);
        self.fields_offset.insert(fieldname.to_string(), n);
        self.rev_fields_offset.insert(n, fieldname.to_string());
        Ok(())
    }

    pub fn get_classname(&self) -> &str {
        &self.class_name
    }

    pub fn get_obj_ref(&self) -> ju4 {
        self.obj_ref
    }

    pub fn monitorenter(&mut self) {
        if self.monitor_count == 0 {
            self.monitor_count += 1
        }
        // TODO: check owner and then self.monitor_count += 1 or wait
        debug!("TODO: monitor enter")
    }

    pub fn monitorexit(&mut self) {
        debug!("TODO: monitor exit")
    }
}

#[derive(Debug, Clone)]
pub struct ArrayInstance {
    class_name: String,         // like [java/lang/Byte
    element_class_name: String, // like java/lang/Byte
    array_ref: ju4,
    elements: Vec<Type>,
}

impl ArrayInstance {
    pub fn new(class_name: &str, array_ref: ju4, elements: Vec<Type>) -> Result<ArrayInstance> {
        let class_type = match class_name {
            "Z" | "C" | "F" | "D" | "B" | "S" | "I" | "J" => &format!("[{}", class_name),
            _ if class_name.starts_with("[") => &format!("[{}", class_name),
            _ => &format!("[L{};", class_name),
        };
        Ok(ArrayInstance {
            class_name: class_type.to_string(),
            element_class_name: types::Type::convert_array_descriptor_to_class_type(class_type)?,
            array_ref,
            elements,
        })
    }

    pub fn print(&self) {
        let mut array = vec![];
        for e in &self.elements {
            array.push(match e {
                Type::Integer(int) => *int as u8,
                Type::Short(int) => *int as u8,
                Type::Byte(int) => *int as u8,
                Type::Character(int) => *int as u8,
                v => panic!("{:?}", v),
            });
        }
        let string = String::from_utf8(array);
        info!("Array: {:?}", string);
    }

    pub fn get_string(&self) -> Result<String> {
        let mut array = vec![];
        for e in &self.elements {
            array.push(match e {
                Type::Integer(int) => *int as u8,
                Type::Short(int) => *int as u8,
                Type::Byte(int) => *int as u8,
                _ => panic!("{:?} {:#?}", e, self),
            });
        }
        Ok(String::from_utf8(array)?)
    }

    pub fn get(&self, index: usize) -> Result<&types::Type> {
        self.elements.get(index).ok_or_eyre(eyre!(
            errors::RuntimeError::ArrayIndexOutOfBoundsException(index, self.elements.len())
        ))
    }
    pub fn get_with_index_scale(&self, index: usize) -> Result<&types::Type> {
        let scale = (get_index_scale(self.get_classname())).ilog2();
        let index = index >> scale;
        self.get(index)
    }
    pub fn get_elements(&self) -> &Vec<types::Type> {
        &self.elements
    }
    pub fn set(&mut self, index: usize, value: Type) -> Result<()> {
        if index >= self.elements.len() {
            return Err(eyre!(errors::RuntimeError::ArrayIndexOutOfBoundsException(
                index,
                self.elements.len()
            )));
        }
        let def_value = get_default_value(&self.class_name);
        if discriminant(&def_value) != discriminant(&value) && def_value != types::Type::Null {
            error!(
                "{:?} {:?}",
                (&get_default_value(&self.class_name)),
                (&value)
            );
            error!("SET WRONG {:?} {:?}", self.class_name, value);
            panic!()
        }
        // TODO: check type
        self.elements[index] = value;
        Ok(())
    }
    // set considering index scale -> rescaling
    pub fn set_with_index_scale(&mut self, index: usize, value: Type) -> Result<()> {
        let scale = (get_index_scale(self.get_classname())).ilog2();
        let index = index >> scale;
        self.set(index, value)
    }
    pub fn get_ref(&self) -> ju4 {
        self.array_ref
    }
    pub fn get_classname(&self) -> &str {
        &self.class_name
    }

    pub fn get_element_classname(&self) -> &str {
        &self.element_class_name
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

#[derive(Debug, Clone)]
pub enum Instance {
    ObjectInstance(ObjectInstance),
    ArrayInstance(ArrayInstance),
}

impl Instance {
    // class field type like [B, [Z, etc
    pub fn get_class_field_type(&self) -> &str {
        match self {
            Instance::ObjectInstance(obj) => obj.get_classname(),
            Instance::ArrayInstance(obj) => obj.get_classname(),
        }
    }
    pub fn get_classname(&self) -> &str {
        match self {
            Instance::ObjectInstance(obj) => obj.get_classname(),
            Instance::ArrayInstance(obj) => obj.get_classname(),
        }
    }
    pub fn get_ref(&self) -> ju4 {
        match self {
            Instance::ObjectInstance(obj) => obj.get_obj_ref(),
            Instance::ArrayInstance(obj) => obj.get_ref(),
        }
    }
    pub fn get_hash_code(&self) -> ju4 {
        match self {
            Instance::ObjectInstance(obj) => obj.get_hash_code(),
            _ => todo!(),
        }
    }

    pub fn _is_array(&self) -> bool {
        matches!(self, Instance::ArrayInstance(_))
    }
}
