use std::ops::Neg;

use super::jvm::*;
use crate::runtime::errors;
use crate::runtime::opcode;
use crate::runtime::types;
use color_eyre::eyre::{eyre, Result};

impl JVM {
    // Float - Exec

    pub(crate) fn exec_fload(&mut self, index: usize) -> Result<Option<types::Type>> {
        let toload = self.get_current_frame()?.get_local(index)?;
        match toload {
            types::Type::Float(_) => {
                self.push_stack(*toload)?;
                Ok(None)
            }
            _ => Err(eyre!("{:?} is not a float", toload)),
        }
    }
    pub(crate) fn exec_fstore(&mut self, index: usize) -> Result<Option<types::Type>> {
        let value = self.pop_stack()?.as_float()?;
        let frame = self.get_current_frame_mut()?;
        frame.set_local(index, types::Type::Float(value));
        Ok(None)
    }
    pub(crate) fn exec_fconst(&mut self, num: f32) -> Result<Option<types::Type>> {
        self.push_stack(types::Type::Float(num))?;
        Ok(None)
    }
    pub(crate) fn exec_f2d(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_stack()?.as_float()?;
        let double = types::Type::Double(value as f64);
        self.push_stack(double)?;
        Ok(None)
    }
    pub(crate) fn exec_f2l(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_stack()?.as_float()?;
        let double = types::Type::Long(value as i64);
        self.push_stack(double)?;
        Ok(None)
    }
    pub(crate) fn exec_f2i(&mut self) -> Result<Option<types::Type>> {
        let value = self.pop_stack()?.as_float()?;
        let double = types::Type::Integer(value as i32);
        self.push_stack(double)?;
        Ok(None)
    }
    pub(crate) fn exec_fcmp(&mut self, opcode: opcode::OpCode) -> Result<Option<types::Type>> {
        let v2 = self.pop_stack()?.as_float()?;
        let v1 = self.pop_stack()?.as_float()?;
        if f32::is_nan(v1) || f32::is_nan(v2) {
            match opcode {
                opcode::OpCode::FCMPG => {
                    self.push_stack(types::Type::Integer(1))?;
                }
                opcode::OpCode::FCMPL => {
                    self.push_stack(types::Type::Integer(-1))?;
                }
                _ => return Err(eyre!("opcode fcmp not recognized")),
            }
        } else if v1 > v2 {
            self.push_stack(types::Type::Integer(1))?;
        } else if v1 == v2 {
            self.push_stack(types::Type::Integer(0))?;
        } else {
            self.push_stack(types::Type::Integer(-1))?;
        }
        Ok(None)
    }
    pub(crate) fn exec_fadd(&mut self) -> Result<Option<types::Type>> {
        let v1 = self.pop_stack()?.as_float()?;
        let v0 = self.pop_stack()?.as_float()?;
        self.push_stack(types::Type::Float(v0 + v1))?;
        Ok(None)
    }
    pub(crate) fn exec_fsub(&mut self) -> Result<Option<types::Type>> {
        let v1 = self.pop_stack()?.as_float()?;
        let v0 = self.pop_stack()?.as_float()?;
        self.push_stack(types::Type::Float(v0 - v1))?;
        Ok(None)
    }
    pub(crate) fn exec_fdiv(&mut self) -> Result<Option<types::Type>> {
        let v1 = self.pop_stack()?.as_float()?;
        let v0 = self.pop_stack()?.as_float()?;

        if v1 == 0.0 {
            return Err(eyre!(errors::RuntimeError::ArithmeticException));
        }
        self.push_stack(types::Type::Float(v0 / v1))?;
        Ok(None)
    }
    pub(crate) fn exec_fmul(&mut self) -> Result<Option<types::Type>> {
        let v1 = self.pop_stack()?.as_float()?;
        let v0 = self.pop_stack()?.as_float()?;
        self.push_stack(types::Type::Float(v0 * v1))?;
        Ok(None)
    }
    pub(crate) fn exec_fneg(&mut self) -> Result<Option<types::Type>> {
        let v0 = self.pop_stack()?.as_float()?;
        self.push_stack(types::Type::Float(v0.neg()))?;
        Ok(None)
    }
    pub(crate) fn exec_freturn(&mut self) -> Result<Option<types::Type>> {
        let return_value = self.pop_stack()?;
        match return_value {
            types::Type::Float(_) => {}
            _ => return Err(eyre!("{:?} is not an Integer", return_value)),
        };
        self.pop_frame();
        let _ = self.push_stack(return_value);
        Ok(Some(return_value))
    }
}
