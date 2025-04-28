use crate::class_loader::attributes::*;
use crate::class_loader::constant_pool::*;
use crate::utils::*;
use color_eyre::eyre::Result;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MethodInfo {
    name: String,       // not in original struct
    descriptor: String, // not in original struct
    access_flags: ju2,
    name_index: ju2,
    descriptor_index: ju2,
    attributes: Attributes,
}

impl MethodInfo {
    fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(MethodInfo, usize)> {
        let mut index: usize = 0;
        let access_flags: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;
        index += 2;
        let name_index: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;
        index += 2;
        let descriptor_index: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;
        index += 2;

        let (attributes, attributes_size) = Attributes::parse(&bytes[index..], cp)?;
        index += attributes_size;

        let name = cp.resolve_name(name_index)?;
        let descriptor = cp.resolve_name(descriptor_index)?;

        Ok((
            MethodInfo {
                name,
                descriptor,
                access_flags,
                name_index,
                descriptor_index,
                attributes,
            },
            index,
        ))
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn _get_name_index(&self) -> ju2 {
        self.name_index
    }
    pub fn get_descriptor(&self) -> &str {
        &self.descriptor
    }
    pub fn _get_descriptor_index(&self) -> ju2 {
        self.descriptor_index
    }
    pub fn _get_attributes(&self) -> &Attributes {
        &self.attributes
    }
    pub fn get_code(&self) -> Option<&Code> {
        for i in 0..self.attributes.len() {
            if let AttributeInfoInfoEnum::Code(c) = self.attributes.get(i).get_info() {
                return Some(c);
            }
        }
        None
    }

    pub fn get_access_flags(&self) -> &ju2 {
        &self.access_flags
    }

    pub fn is_public(&self) -> bool {
        (self.access_flags & 0x0001) == 0x0001
    }
    pub fn is_native(&self) -> bool {
        (self.access_flags & 0x0100) == 0x0100
    }
    pub fn _is_static(&self) -> bool {
        (self.access_flags & 0x0008) == 0x0008
    }
}

#[derive(Debug)]
pub struct Methods {
    methods: Vec<MethodInfo>,
}
impl Methods {
    pub fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Methods, usize)> {
        //debug!("[-] Parsing Methods");
        let mut methods: Vec<MethodInfo> = vec![];
        let mut index: usize = 0;
        let methods_count: ju2 = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
        //debug!("    Found {:?} methods", methods_count);
        index += 2;
        for _ in 0..methods_count {
            let slice = &bytes[index..];
            let (field, size) = MethodInfo::parse(slice, cp)?;
            index += size;
            methods.push(field);
        }
        Ok((Methods { methods }, index))
    }

    pub fn empty() -> Methods {
        Methods { methods: vec![] }
    }

    pub fn len(&self) -> usize {
        self.methods.len()
    }

    pub fn get(&self, index: usize) -> &MethodInfo {
        &self.methods[index]
    }

    pub fn get_methods(&self) -> &Vec<MethodInfo> {
        &self.methods
    }

    pub fn _get_code_by_name_index(&self, index: ju2) -> Option<&MethodInfo> {
        for i in 0..self.methods.len() {
            if self.methods[i]._get_name_index() == index {
                return Some(&self.methods[i]);
            }
        }
        None
    }

    pub fn _get_by_name_index(&self, index: ju2) -> Option<&MethodInfo> {
        for i in 0..self.methods.len() {
            if self.methods[i]._get_name_index() == index {
                return Some(&self.methods[i]);
            }
        }
        None
    }

    pub fn get_by_name(&self, name: &str, method_type: &str) -> Option<&MethodInfo> {
        for i in 0..self.methods.len() {
            if self.methods[i].get_name() == name && self.methods[i].get_descriptor() == method_type
            {
                return Some(&self.methods[i]);
            }
        }
        None
    }
}
