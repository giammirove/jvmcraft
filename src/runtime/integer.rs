use super::jvm::*;
use crate::runtime::errors;
use crate::runtime::opcode;
use crate::runtime::types;
use crate::utils::*;
use color_eyre::eyre::{eyre, Result};
use log::{debug, info};

impl JVM {
    // Integer - Exec

    pub(crate) fn pop_ioperand(&mut self) -> Result<i32> {
        let value1 = self.pop_stack()?;
        let v1 = match value1 {
            types::Type::Integer(v) => v,
            types::Type::Short(v) => v as i32,
            types::Type::Byte(v) => v as i32,
            types::Type::Character(v) => v as i32,
            types::Type::Boolean(v) => v as i32,
            v => return Err(eyre!("{:?} is not a integer", v)),
        };
        Ok(v1)
    }
    pub(crate) fn pop_ioperands(&mut self) -> Result<(i32, i32)> {
        let v2 = self.pop_ioperand()?;
        let v1 = self.pop_ioperand()?;
        Ok((v1, v2))
    }

    // Load int from local variable
    pub(crate) fn exec_iload(&mut self, index: usize) -> Result<Option<types::Type>> {
        let value = self.get_current_frame()?.get_local(index)?;
        if value.is_integer() {
            self.push_stack(*value)?;
            Ok(None)
        } else {
            Err(eyre!("{:?} is not an integer", value))
        }
    }
    // Store int into local variable
    pub(crate) fn exec_istore(&mut self, index: usize) -> Result<Option<types::Type>> {
        let value = self.pop_stack()?;
        if value.is_integer() {
            let frame = self.get_current_frame_mut()?;
            frame.set_local(index, value);
            Ok(None)
        } else {
            Err(eyre!("{:?} is not a integer", value))
        }
    }
    // Push int constant
    pub(crate) fn exec_iconst(&mut self, num: i32) -> Result<Option<types::Type>> {
        self.push_stack(types::Type::Integer(num))?;
        Ok(None)
    }

    pub(crate) fn exec_iushr(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        let shift = v2 & 0x1F; // use only low 5 bits
        let result = ((v1 as u32) >> shift) as i32;
        self.push_stack(types::Type::Integer(result))?;
        Ok(None)
    }
    pub(crate) fn exec_ishl(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        let shift = v2 & 0x1F; // use only low 5 bits
        let result = ((v1 as u32) << shift) as i32;
        self.push_stack(types::Type::Integer(result))?;
        Ok(None)
    }
    pub(crate) fn exec_ishr(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        let shift = v2 & 0x1F; // use only low 5 bits
        let result = ((v1 as u32) >> shift) as i32;
        self.push_stack(types::Type::Integer(result))?;
        Ok(None)
    }
    pub(crate) fn exec_i2f(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
        self.push_stack(types::Type::Float(value as f32))?;
        Ok(None)
    }
    pub(crate) fn exec_i2d(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
        self.push_stack(types::Type::Double(value as f64))?;
        Ok(None)
    }
    pub(crate) fn exec_i2l(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
        self.push_stack(types::Type::Long(value as i64))?;
        Ok(None)
    }
    pub(crate) fn exec_i2s(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
        self.push_stack(types::Type::Short(sign_extend16(value as u16) as i16))?;
        Ok(None)
    }
    pub(crate) fn exec_i2c(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
        self.push_stack(types::Type::Character(value as i8))?;
        Ok(None)
    }
    pub(crate) fn exec_i2b(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
        self.push_stack(types::Type::Byte(value as i8))?;
        Ok(None)
    }
    pub(crate) fn exec_if(&mut self, opcode: opcode::OpCode) -> Result<Option<types::Type>> {
        let offset = self.get_current_frame_mut()?.read_ju2()? as i16;
        let value = self.pop_ioperand()?;
        if (opcode == opcode::OpCode::IFEQ && value == 0)
            || (opcode == opcode::OpCode::IFNE && value != 0)
            || (opcode == opcode::OpCode::IFLT && value < 0)
            || (opcode == opcode::OpCode::IFLE && value <= 0)
            || (opcode == opcode::OpCode::IFGT && value > 0)
            || (opcode == opcode::OpCode::IFGE && value >= 0)
        {
            self.jump_by(offset)?;
        }
        Ok(None)
    }
    pub(crate) fn exec_if_icmp(&mut self, opcode: opcode::OpCode) -> Result<Option<types::Type>> {
        let offset = self.get_current_frame_mut()?.read_ju2()? as i16;
        let (v1, v2) = self.pop_ioperands()?;
        if (opcode == opcode::OpCode::IFICMPEQ && v1 == v2)
            || (opcode == opcode::OpCode::IFICMPNE && v1 != v2)
            || (opcode == opcode::OpCode::IFICMPLT && v1 < v2)
            || (opcode == opcode::OpCode::IFICMPLE && v1 <= v2)
            || (opcode == opcode::OpCode::IFICMPGT && v1 > v2)
            || (opcode == opcode::OpCode::IFICMPGE && v1 >= v2)
        {
            self.jump_by(offset)?
        }
        Ok(None)
    }
    pub(crate) fn exec_iadd(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        self.push_stack(types::Type::Integer(v1.wrapping_add(v2)))?;
        Ok(None)
    }
    pub(crate) fn exec_isub(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        self.push_stack(types::Type::Integer(v1.wrapping_sub(v2)))?;
        Ok(None)
    }
    pub(crate) fn exec_idiv(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        if v2 == 0 {
            return Err(eyre!(errors::RuntimeError::ArithmeticException));
        }
        self.push_stack(types::Type::Integer(v1.wrapping_div(v2)))?;
        Ok(None)
    }
    pub(crate) fn exec_imul(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        self.push_stack(types::Type::Integer(v1.wrapping_mul(v2)))?;
        Ok(None)
    }
    pub(crate) fn exec_ineg(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
        self.push_stack(types::Type::Integer(value.wrapping_neg()))?;
        Ok(None)
    }
    pub(crate) fn exec_irem(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        if v2 == 0 {
            return Err(eyre!(errors::RuntimeError::ArithmeticException));
        }
        let result = v1.wrapping_sub((v1.wrapping_div(v2)).wrapping_mul(v2));
        self.push_stack(types::Type::Integer(result))?;
        Ok(None)
    }
    pub(crate) fn exec_iinc(&mut self) -> Result<Option<types::Type>> {
        let index = self.get_current_frame_mut()?.read_ju1()? as usize;
        let const_par = sign_extend8(self.get_current_frame_mut()?.read_ju1()?);
        let local = *self.get_current_frame_mut()?.get_local(index)?;
        match local {
            types::Type::Integer(v1) => {
                debug!("        [~] IINC {:?} ({:?}) {:?}", index, v1, const_par);
                self.get_current_frame_mut()?
                    .set_local(index, types::Type::Integer(v1.wrapping_add(const_par)))
            }
            v => return Err(eyre!(errors::RuntimeError::WrongType("integer", v))),
        }
        Ok(None)
    }
    pub(crate) fn exec_ixor(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        self.push_stack(types::Type::Integer(v1 ^ v2))?;
        Ok(None)
    }
    pub(crate) fn exec_ior(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        self.push_stack(types::Type::Integer(v1 | v2))?;
        Ok(None)
    }
    pub(crate) fn exec_iand(&mut self) -> Result<Option<types::Type>> {
        let (v1, v2) = self.pop_ioperands()?;
        self.push_stack(types::Type::Integer(v1 & v2))?;
        Ok(None)
    }

    pub(crate) fn exec_iastore(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_ioperand()?;
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
        if array.get_classname() != "[I" {
            return Err(eyre!(format!("Input should be of type int {:?}", array)));
        }
        array.set(index as usize, types::Type::Integer(value))?;
        Ok(None)
    }

    pub(crate) fn exec_iaload(&mut self) -> Result<Option<types::Type>> {
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
        let array = self.heap.get_array_instance(array_ref)?;
        if array.get_classname() != "[I" {
            return Err(eyre!(format!("Input should be of type int {:?}", array)));
        }
        let ret = array.get(index as usize)?.as_integer()?;
        self.push_stack(types::Type::Integer(ret))?;
        Ok(None)
    }

    pub(crate) fn exec_ireturn(&mut self) -> Result<Option<types::Type>> {
        let return_value = self.pop_stack()?;
        if return_value.is_integer() {
            self.pop_frame();
            // pushed into invoker stack
            let _ = self.push_stack(return_value);
            info!("        IRETURN {:?}", return_value);
            return Ok(Some(return_value));
        }

        Err(eyre!(errors::RuntimeError::WrongType(
            "Integer",
            return_value
        )))
    }
}
