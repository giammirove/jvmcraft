use crate::class_loader::attributes::*;
use crate::class_loader::constant_pool::*;
use crate::utils::*;
use color_eyre::eyre::Result;
use log::debug;

#[derive(Debug)]
#[allow(dead_code)]
pub struct FieldInfo {
    access_flags: ju2,
    name: String,
    descriptor: String,
    attributes: Attributes,
}

impl FieldInfo {
    fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(FieldInfo, usize)> {
        let mut index: usize = 0;
        let access_flags: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;
        index += 2;
        let name_index: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;
        index += 2;
        let name = cp.resolve_name(name_index)?;
        let descriptor_index: ju2 = ju2_from_bytes(get_slice_arr(bytes, index, 2))?;
        let descriptor = cp.resolve_name(descriptor_index)?;
        index += 2;

        let (attributes, attributes_size) = Attributes::parse(&bytes[index..], cp)?;
        index += attributes_size;

        Ok((
            FieldInfo {
                access_flags,
                name,
                descriptor,
                attributes,
            },
            index,
        ))
    }

    pub(crate) fn new(access_flags: ju2, name: &str, descriptor: &str) -> FieldInfo {
        FieldInfo {
            access_flags,
            name: name.to_string(),
            descriptor: descriptor.to_string(),
            attributes: Attributes::new(0, vec![]),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_descriptor(&self) -> &str {
        &self.descriptor
    }
    pub fn _get_attributes(&self) -> &Attributes {
        &self.attributes
    }
    pub fn is_static(&self) -> bool {
        (self.access_flags & 0x0008) == 0x0008
    }
}

#[derive(Debug)]
pub struct Fields {
    fields_count: ju2,
    fields: Vec<FieldInfo>,
}
impl Fields {
    pub fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Fields, usize)> {
        debug!("[-] Parsing Fields");
        let mut fields: Vec<FieldInfo> = vec![];
        let mut index: usize = 0;
        let fields_count: ju2 = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
        //debug!("    Found {:?} fields", fields_count);
        index += 2;
        for _ in 0..fields_count {
            let slice = &bytes[index..];
            let (field, size) = FieldInfo::parse(slice, cp)?;
            //debug!("    Field: {:?}", field);
            index += size;
            fields.push(field);
        }
        Ok((
            {
                Fields {
                    fields_count,
                    fields,
                }
            },
            index,
        ))
    }
    pub fn empty() -> Fields {
        Fields {
            fields_count: 0,
            fields: vec![],
        }
    }

    pub fn put_field(&mut self, field: FieldInfo) {
        self.fields_count += 1;
        self.fields.push(field);
    }

    pub fn get_fields(&self) -> &Vec<FieldInfo> {
        &self.fields
    }
    pub fn _get_field(&self, index: usize) -> &FieldInfo {
        &self.fields[index]
    }
}
