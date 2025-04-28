use std::collections::HashMap;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ConstantInfo {
    MethodHandle {
        method_name: String,
        descriptor: String,
    },
    VarHandle {
        field_name: String,
        descriptor: String,
    },
    CallSite {
        bootstrap_args: Vec<String>,
    },
}

#[derive(Debug)]
pub(crate) struct Constants {
    next_id: i32,
    constants_table: HashMap<i32, ConstantInfo>,
}

impl Constants {
    pub(crate) fn new() -> Self {
        Constants {
            next_id: 0,
            constants_table: HashMap::new(),
        }
    }

    pub(crate) fn get_next_id(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(crate) fn insert(&mut self, id: i32, value: ConstantInfo) {
        self.constants_table.insert(id, value);
    }
}
