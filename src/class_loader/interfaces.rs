use color_eyre::eyre::Result;
use log::debug;

use super::constant_pool::ConstantPool;
use crate::utils::*;

#[derive(Debug)]
pub struct Interfaces {
  interfaces: Vec<String>,
}

impl Interfaces {
  pub fn parse(bytes: &[u8], cp: &ConstantPool) -> Result<(Interfaces, usize)> {
    debug!("[-] Parsing Interfaces");

    let mut interfaces: Vec<String> = vec![];

    let mut index: usize = 0;

    let interfaces_count: ju2 = u16::from_be_bytes(bytes[0..2].try_into().unwrap());

    debug!("    Found {:?} interfaces", interfaces_count);
    index += 2;

    for _ in 0..interfaces_count {
      let slice = &bytes[index..];

      let interf: ju2 = u16::from_be_bytes(slice[0..2].try_into().unwrap());

      let intername = cp.resolve_name(cp.resolve_class(interf)?)?;
      debug!("    Interface : {:?}", intername);

      index += 2;

      interfaces.push(intername);
    }

    Ok((Interfaces { interfaces }, index))
  }

  pub fn empty() -> Interfaces {
    Interfaces { interfaces: vec![] }
  }

  pub fn add_interface(&mut self, interface: String) {
    self.interfaces.push(interface)
  }

  pub fn get_interfaces(&self) -> &Vec<String> {
    &self.interfaces
  }
}
