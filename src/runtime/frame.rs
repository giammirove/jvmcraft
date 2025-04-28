use std::fmt;

use crate::runtime::*;
use crate::utils::*;
use color_eyre::eyre::{eyre, OptionExt, Result};
use log::debug;

#[derive(Debug, Clone)]
pub struct Frame {
    pc: usize,
    last_opcode_pc: usize,
    class: String,
    func_name: String,
    func_type: String,
    code: Vec<ju1>,
    stack: Vec<types::Type>,
    local: Vec<types::Type>,
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}.{} {}", self.class, self.func_name, self.func_type)?;
        for l in &self.local {
            if *l == types::Type::None {
                break;
            }
            writeln!(f, "\t\t  {:?}", l)?;
        }
        Ok(())
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.class == *other.get_classname() && self.func_name == other.get_func_name()
    }
}

impl Frame {
    pub fn new(
        class: String,
        func_name: String,
        func_type: String,
        code: Vec<ju1>,
        args: Vec<types::Type>,
    ) -> Frame {
        Frame {
            pc: 0,
            last_opcode_pc: 0,
            class,
            func_name,
            func_type,
            code,
            stack: vec![],
            local: args,
        }
    }

    pub fn read_current_opcode(&mut self) -> Result<ju1> {
        self.last_opcode_pc = self.pc;
        self.read_ju1()
    }
    pub fn read_ju1(&mut self) -> Result<ju1> {
        if self.pc >= self.code.len() {
            return Err(eyre!("no more code to execute"));
        }
        let op = self.code[self.pc];
        self.pc += 1;
        Ok(op)
    }
    pub fn read_ju2(&mut self) -> Result<ju2> {
        if self.pc + 1 >= self.code.len() {
            return Err(eyre!("no more code to execute"));
        }
        let index = ju2_from_bytes(get_slice_arr(&self.code, self.pc, 2))?;
        self.pc += 2;
        Ok(index)
    }
    pub fn read_ju4(&mut self) -> Result<ju4> {
        if self.pc + 3 >= self.code.len() {
            return Err(eyre!("no more code to execute"));
        }
        let index = ju4_from_bytes(get_slice_arr(&self.code, self.pc, 4))?;
        self.pc += 4;
        Ok(index)
    }

    pub fn set_local(&mut self, index: usize, value: types::Type) {
        self.local[index] = value;
    }
    pub fn get_local(&self, index: usize) -> Result<&types::Type> {
        if index >= self.local.len() {
            return Err(eyre!(
                "locals is not big enough {:?} >= {:?}",
                index,
                self.local.len()
            ));
        }
        Ok(&self.local[index])
    }

    pub fn get_func_name(&self) -> &str {
        &self.func_name
    }
    pub fn get_func_type(&self) -> &str {
        &self.func_type
    }
    pub fn get_code(&self) -> &Vec<ju1> {
        &self.code
    }
    pub fn get_code_length(&self) -> usize {
        self.code.len()
    }
    pub fn get_pc(&self) -> usize {
        self.pc
    }
    // mainly used for testing
    pub fn _set_last_opcode_pc(&mut self, pc: usize) {
        self.last_opcode_pc = pc;
    }
    pub fn can_jump_by(&self, offset: i16) -> bool {
        (self.last_opcode_pc as i32 + offset as i32) < self.get_code_length() as i32
    }
    pub fn jump_by(&mut self, offset: i16) {
        self.pc = (self.last_opcode_pc as i32 + offset as i32) as usize;
    }
    pub fn get_classname(&self) -> &String {
        &self.class
    }

    pub fn push_stack(&mut self, value: types::Type) {
        //debug!("PUSH STACK {:?}/{:?}", value, self.stack.len() + 1);
        self.stack.push(value);
        debug!("PUSH STACK {:?}", self.stack);
    }

    pub fn pop_stack(&mut self) -> Result<types::Type> {
        let popped = self.stack.pop();
        //debug!("POP STACK {:?}/{:?}", popped, self.stack.len());
        debug!("POP STACK {:?}", self.stack);
        popped.ok_or_eyre("stack is empty")
    }
}
