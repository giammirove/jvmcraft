use super::jvm::*;
use crate::class_loader::constant_pool;
use crate::notimpl;
use crate::runtime::errors;
use crate::runtime::types;
use crate::utils::*;
use color_eyre::eyre::{eyre, Result};
use log::{debug, info};

impl JVM {
    // Long - Exec

    fn push_stack_long(&mut self, value: types::Type) -> Result<()> {
        self.push_stack(value)
    }

    pub(crate) fn pop_loperand(&mut self) -> Result<i64> {
        let value1 = self.pop_stack()?.as_long()?;
        Ok(value1)
    }

    fn pop_loperands(&mut self) -> Result<(i64, i64)> {
        let v2 = self.pop_loperand()?;
        let v1 = self.pop_loperand()?;
        Ok((v1, v2))
    }

    pub(crate) fn exec_lload(&mut self, index: usize) -> Result<Option<types::Type>> {
        let toload = self.get_current_frame()?.get_local(index)?;
        match toload {
            types::Type::Long(_) => {
                self.push_stack_long(*toload)?;
                Ok(None)
            }
            _ => Err(eyre!("{:?} is not an integer", toload)),
        }
    }
    pub(crate) fn exec_lstore(&mut self, index: usize) -> Result<Option<types::Type>> {
        let value = self.pop_loperand()?;
        let frame = self.get_current_frame_mut()?;
        frame.set_local(index, types::Type::Long(value));
        frame.set_local(index + 1, types::Type::Long(value));
        Ok(None)
    }

    pub(crate) fn exec_lushr(&mut self) -> Result<Option<types::Type>> {
        let shift = self.pop_ioperand()?;
        let value = self.pop_loperand()?;

        let amount = (shift & 0x3F) as u32;
        let result = ((value as u64) >> amount) as i64;

        self.push_stack(types::Type::Long(result))?;
        Ok(None)
    }

    pub(crate) fn exec_ladd(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        self.push_stack_long(types::Type::Long(v1.wrapping_add(v2)))?;
        Ok(None)
    }
    pub(crate) fn exec_lsub(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        self.push_stack_long(types::Type::Long(v1.wrapping_sub(v2)))?;
        Ok(None)
    }
    pub(crate) fn exec_ldiv(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        if v2 == 0 {
            return Err(eyre!(errors::RuntimeError::ArithmeticException));
        }
        self.push_stack_long(types::Type::Long(v1.wrapping_div(v2)))?;
        Ok(None)
    }
    pub(crate) fn exec_lmul(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        self.push_stack_long(types::Type::Long(v1.wrapping_mul(v2)))?;
        Ok(None)
    }

    pub(crate) fn exec_lxor(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        self.push_stack_long(types::Type::Long(v1 ^ v2))?;
        Ok(None)
    }
    pub(crate) fn exec_lor(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        self.push_stack_long(types::Type::Long(v1 | v2))?;
        Ok(None)
    }
    pub(crate) fn exec_land(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        self.push_stack_long(types::Type::Long(v1 & v2))?;
        Ok(None)
    }
    pub(crate) fn exec_lrem(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        if v2 == 0 {
            return Err(eyre!(errors::RuntimeError::ArithmeticException));
        }
        let result = v1.wrapping_sub((v1.wrapping_div(v2)).wrapping_mul(v2));
        self.push_stack(types::Type::Long(result))?;
        Ok(None)
    }

    pub(crate) fn exec_ldc2w(&mut self, index: ju2) -> Result<Option<types::Type>> {
        let value: types::Type = {
            let class = self.get_current_class()?;
            let item = class.resolve_index(index)?;
            match &item.get_info() {
                constant_pool::CpInfoInfoEnum::Long(int) => types::Type::Long(int.value() as i64),
                constant_pool::CpInfoInfoEnum::Double(fl) => {
                    types::Type::Double(f64::from_bits(fl.value()))
                }
                _ => notimpl!("ldc2w not implemented for {:?}", item),
            }
        };
        self.push_stack_long(value)?;
        Ok(None)
    }

    pub(crate) fn exec_lconst(&mut self, index: ju2) -> Result<Option<types::Type>> {
        self.push_stack_long(types::Type::Long(index as i64))?;
        Ok(None)
    }

    pub(crate) fn exec_lreturn(&mut self) -> Result<Option<types::Type>> {
        let return_value = types::Type::Long(self.pop_loperand()?);
        self.pop_frame();
        // pushed into invoker stack
        let _ = self.push_stack_long(return_value);
        info!("        LRETURN {:?}", return_value);
        Ok(Some(return_value))
    }

    pub(crate) fn exec_l2i(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_loperand()?;
        self.push_stack_long(types::Type::Integer(value as i32))?;
        Ok(None)
    }
    pub(crate) fn exec_l2d(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_loperand()?;
        self.push_stack_long(types::Type::Double(value as f64))?;
        Ok(None)
    }
    pub(crate) fn exec_l2f(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_loperand()?;
        self.push_stack_long(types::Type::Float(value as f32))?;
        Ok(None)
    }

    pub(crate) fn exec_lshl(&mut self) -> Result<Option<types::Type>> {
        let v2 = self.pop_ioperand()?;
        let v1 = self.pop_loperand()?;
        let shift = (v2 & 0x3F) as u32;
        let result = v1 << shift;
        self.push_stack_long(types::Type::Long(result))?;
        Ok(None)
    }
    pub(crate) fn exec_lshr(&mut self) -> Result<Option<types::Type>> {
        let v2 = self.pop_ioperand()?;
        let v1 = self.pop_loperand()?;
        let shift = (v2 & 0x3F) as u32;
        let result = v1 >> shift;
        self.push_stack_long(types::Type::Long(result))?;
        Ok(None)
    }
    pub(crate) fn exec_lcmp(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_loperands()?;
        let value = match v1.cmp(&v2) {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Less => -1,
        };
        self.push_stack_long(types::Type::Integer(value))?;
        Ok(None)
    }

    pub(crate) fn exec_lastore(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_loperand()?;
        let index = self.pop_ioperand()?;
        let array_ref = self.pop_stack()?;
        let array_ref = match array_ref {
            types::Type::ArrayRef(array_ref) => array_ref,
            _ => {
                return Err(eyre!(format!(
                    "ArrayRef not in the stack : {:?} {:?}",
                    array_ref, index
                )))
            }
        };
        let array = self.heap.get_array_instance_mut(array_ref)?;
        if array.get_classname() != "[J" {
            return Err(eyre!(format!("Input should be of type long {:?}", array)));
        }
        array.set(index as usize, types::Type::Long(value))?;
        Ok(None)
    }

    pub(crate) fn exec_laload(&mut self) -> Result<Option<types::Type>> {
        let index = self.pop_loperand()?;
        let array_ref = self.pop_stack()?;
        let array_ref = match array_ref {
            types::Type::ArrayRef(array_ref) => array_ref,
            _ => {
                return Err(eyre!(format!(
                    "ArrayRef not in the stack : {:?} {:?}",
                    array_ref, index
                )))
            }
        };
        let array = self.heap.get_array_instance(array_ref)?;
        debug!("{:?}", array);
        if array.get_classname() != "[J" {
            return Err(eyre!(format!("Input should be of type long {:?}", array)));
        }
        let ret = array.get(index as usize)?.as_long()?;
        self.push_stack(types::Type::Long(ret))?;
        Ok(None)
    }
}
