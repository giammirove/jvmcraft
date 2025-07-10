use std::collections::HashMap;

use color_eyre::eyre::{eyre, OptionExt, Result};
use log::debug;

#[derive(Debug)]
pub struct NativeMemory {
  allocations: HashMap<u64, u64>,
}

impl NativeMemory {
  pub fn new() -> Self {
    NativeMemory {
      allocations: HashMap::new(),
    }
  }

  // must be freed
  pub fn alloc(&mut self, size: u64) -> Result<u64> {
    let ptr = unsafe { libc::malloc(size as usize) as *mut u8 };

    if ptr.is_null() {
      return Err(eyre!("OutOfMemoryError"));
    }

    let addr = ptr as u64;

    debug!("{} {}", addr, size);

    self.allocations.insert(addr, size);

    Ok(addr)
  }

  pub fn register(&mut self, addr: u64, size: u64) {
    self.allocations.insert(addr, size);
  }

  pub(crate) fn is_valid(&self, addr: u64) -> bool {
    for (k, v) in &self.allocations {
      if addr >= *k && addr < *k + *v {
        return true;
      }
    }

    panic!("invalid address {}", addr);
  }

  pub(crate) fn _get(&self, addr: u64) -> Result<&u64> {
    self
      .allocations
      .get(&addr)
      .ok_or_eyre(eyre!("off-heap not found"))
  }

  pub(crate) fn read_string(&self, addr: u64) -> Result<String> {
    let mut bytes = vec![];

    let mut offset = 0;

    let addr = addr as *const u8;

    loop {
      let b = unsafe { *(addr.wrapping_add(offset)) };

      if b == 0 {
        break;
      }

      bytes.push(b);

      offset += 1;
    }

    Ok(String::from_utf8(bytes)?)
  }
}
