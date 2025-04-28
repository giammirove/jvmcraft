use super::jvm::*;
use crate::runtime::opcode;
use crate::runtime::types;
use color_eyre::eyre::{eyre, Result};
use log::{debug, info};

impl JVM {
    // Reference - Exec

    pub(crate) fn pop_reference(&mut self) -> Result<types::Type> {
        let value1 = self.pop_stack()?;
        match value1 {
            types::Type::ObjectRef(_) | types::Type::ArrayRef(_) | types::Type::Null => Ok(value1),
            v => Err(eyre!("{:?} is not a reference", v)),
        }
    }
    pub(crate) fn pop_references(&mut self) -> Result<(types::Type, types::Type)> {
        let v2 = self.pop_reference()?;
        let v1 = self.pop_reference()?;
        Ok((v1, v2))
    }

    pub(crate) fn exec_aconstnull(&mut self) -> Result<Option<types::Type>> {
        self.push_stack(types::Type::Null)?;
        Ok(None)
    }

    pub(crate) fn exec_aload(&mut self, index: usize) -> Result<Option<types::Type>> {
        let value = self.get_current_frame()?.get_local(index)?;
        match value {
            types::Type::ObjectRef(_) | types::Type::ArrayRef(_) | types::Type::Null => {
                debug!("        [~] ALOAD at {:?} is {:?}", index, value);
                self.push_stack(*value)?;
                Ok(None)
            }
            _ => Err(eyre!(
                "{:?} is not an ObjectRef/ArrayRef/Null at index {}",
                value,
                index
            )),
        }
    }

    pub(crate) fn exec_astore(&mut self, index: usize) -> Result<Option<types::Type>> {
        let value = self.pop_reference()?;
        let frame = self.get_current_frame_mut()?;
        frame.set_local(index, value);
        Ok(None)
    }

    pub(crate) fn exec_if_acmp(&mut self, opcode: opcode::OpCode) -> Result<Option<types::Type>> {
        let offset = self.get_current_frame_mut()?.read_ju2()? as i32;
        let (value1, value2) = self.pop_references()?;
        let eq = value1 == value2;
        if (opcode == opcode::OpCode::IFACMPEQ && eq) || (opcode == opcode::OpCode::IFACMPNE && !eq)
        {
            self.jump_by(offset as i16)?;
        }
        Ok(None)
    }

    pub(crate) fn exec_areturn(&mut self) -> Result<Option<types::Type>> {
        let return_value = self.pop_reference()?;
        self.pop_frame();
        // pushed into invoker stack
        let _ = self.push_stack(return_value);
        info!("        ARETURN {:?}", return_value);
        Ok(Some(return_value))
    }
}
