use std::ops::{Add, Div, Mul, Neg, Sub};

use color_eyre::eyre::{eyre, Result};
use log::info;

use super::jvm::*;
use crate::runtime::{errors, opcode, types};

impl JVM {
  // Double - Exec
  fn push_stack_double(&mut self, value: types::Type) -> Result<()> {
    self.push_stack(value)
  }

  pub(crate) fn pop_doperand(&mut self) -> Result<f64> {
    let value1 = self.pop_stack()?.as_double()?;

    Ok(value1)
  }

  pub(crate) fn pop_doperands(&mut self) -> Result<(f64, f64)> {
    let v2 = self.pop_doperand()?;

    let v1 = self.pop_doperand()?;

    Ok((v1, v2))
  }

  pub(crate) fn exec_dload(&mut self, index: usize) -> Result<Option<types::Type>> {
    let toload = self.get_current_frame()?.get_local(index)?;

    match toload {
      types::Type::Double(_) => {
        self.push_stack_double(*toload)?;

        Ok(None)
      }
      _ => Err(eyre!("{:?} is not an double", toload)),
    }
  }

  pub(crate) fn exec_dstore(&mut self, index: usize) -> Result<Option<types::Type>> {
    let value = self.pop_doperand()?;

    let frame = self.get_current_frame_mut()?;

    frame.set_local(index, types::Type::Double(value));

    Ok(None)
  }

  pub(crate) fn exec_dcmp(&mut self, opcode: opcode::OpCode) -> Result<Option<types::Type>> {
    let (d1, d2) = self.pop_doperands()?;

    if f64::is_nan(d1) || f64::is_nan(d2) {
      match opcode {
        opcode::OpCode::DCMPG => {
          self.push_stack(types::Type::Integer(1))?;
        }
        opcode::OpCode::DCMPL => {
          self.push_stack(types::Type::Integer(-1))?;
        }
        _ => return Err(eyre!("opcode dcmp not recognized")),
      }
    } else if d1 > d2 {
      self.push_stack(types::Type::Integer(1))?;
    } else if d1 == d2 {
      self.push_stack(types::Type::Integer(0))?;
    } else {
      self.push_stack(types::Type::Integer(-1))?;
    }

    Ok(None)
  }

  pub(crate) fn exec_dconst(&mut self, num: f64) -> Result<Option<types::Type>> {
    self.push_stack(types::Type::Double(num))?;

    Ok(None)
  }

  pub(crate) fn exec_dadd(&mut self) -> Result<Option<types::Type>> {
    let (v1, v2) = self.pop_doperands()?;

    self.push_stack(types::Type::Double(v1.add(v2)))?;

    Ok(None)
  }

  pub(crate) fn exec_dsub(&mut self) -> Result<Option<types::Type>> {
    let (v1, v2) = self.pop_doperands()?;

    self.push_stack(types::Type::Double(v1.sub(v2)))?;

    Ok(None)
  }

  pub(crate) fn exec_ddiv(&mut self) -> Result<Option<types::Type>> {
    let (v1, v2) = self.pop_doperands()?;

    if v2 == 0.0 {
      return Err(eyre!(errors::JavaException::Arithmetic));
    }

    self.push_stack(types::Type::Double(v1.div(v2)))?;

    Ok(None)
  }

  pub(crate) fn exec_dmul(&mut self) -> Result<Option<types::Type>> {
    let (v1, v2) = self.pop_doperands()?;

    self.push_stack(types::Type::Double(v1.mul(v2)))?;

    Ok(None)
  }

  pub(crate) fn exec_dneg(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_doperand()?;

    self.push_stack(types::Type::Double(value.neg()))?;

    Ok(None)
  }

  pub(crate) fn exec_d2l(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_doperand()?;

    self.push_stack(types::Type::Long(value as i64))?;

    Ok(None)
  }

  pub(crate) fn exec_d2i(&mut self) -> Result<Option<types::Type>> {
    let value = self.pop_doperand()?;

    self.push_stack(types::Type::Integer(value as i32))?;

    Ok(None)
  }

  pub(crate) fn exec_dreturn(&mut self) -> Result<Option<types::Type>> {
    let return_value = types::Type::Double(self.pop_doperand()?);

    self.pop_frame();

    // pushed into invoker stack
    let _ = self.push_stack(return_value);

    info!("        DRETURN {:?}", return_value);

    Ok(Some(return_value))
  }
}
