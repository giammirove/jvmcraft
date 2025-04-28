#[cfg(test)]
mod tests {
    use crate::runtime::{jvm::JVM, types};

    #[test]
    fn test_exec_aconstnull() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestReference", "aconstnull", "()V", vec![])
            .unwrap();

        assert!(jvm.exec_aconstnull().unwrap().is_none());

        let null = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(null == types::Type::Null);
    }

    #[test]
    fn test_exec_aload() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestReference", "aload", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::ObjectRef(23 + i);
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_aload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
        for i in 0..5 {
            let test = types::Type::ArrayRef(23 + i);
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_aload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
        for i in 0..5 {
            let test = types::Type::Null;
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_aload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
        let test = types::Type::Long(23);
        jvm.get_current_frame_mut().unwrap().set_local(0, test);
        assert!(jvm.exec_aload(0).is_err())
    }

    #[test]
    fn test_exec_astore() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestReference", "astore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::ObjectRef(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_astore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
        for i in 0..5 {
            let test = types::Type::ArrayRef(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_astore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
        for i in 0..5 {
            let test = types::Type::Null;
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_astore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
        for i in 0..5 {
            let test = types::Type::Long(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_astore(i as usize).is_err());
        }
    }

    #[test]
    fn test_exec_if_acmp() {
        // jump done in case of positive check
        let posjump = 4;
        // jump done in case of negative check
        let negjump = 3;
        let values = vec![
            (types::Type::Null, types::Type::Null),
            //
            (types::Type::Null, types::Type::ObjectRef(23)),
            (types::Type::ObjectRef(23), types::Type::Null),
            (types::Type::Null, types::Type::ArrayRef(23)),
            (types::Type::ArrayRef(23), types::Type::Null),
            //
            (types::Type::ObjectRef(23), types::Type::ObjectRef(23)),
            (types::Type::ObjectRef(23), types::Type::ObjectRef(46)),
            (types::Type::ArrayRef(23), types::Type::ArrayRef(23)),
            (types::Type::ArrayRef(23), types::Type::ArrayRef(46)),
            //
            (types::Type::ObjectRef(23), types::Type::ArrayRef(23)),
            (types::Type::ArrayRef(23), types::Type::ObjectRef(23)),
        ];
        let opcodes = vec![
            crate::runtime::opcode::OpCode::IFACMPEQ,
            crate::runtime::opcode::OpCode::IFACMPNE,
        ];
        for (v1, v2) in values {
            for opcode in &opcodes {
                let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
                jvm.push_frame_from_class("TestReference", "if_acmp", "()V", vec![])
                    .unwrap();
                jvm.get_current_frame_mut().unwrap().push_stack(v1);
                jvm.get_current_frame_mut().unwrap().push_stack(v2);
                // read first instructions
                for _ in 0..7 {
                    assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
                }
                let prev_pc = jvm.get_current_frame().unwrap().get_pc() - 1;
                // due to internal logic of jump
                jvm.get_current_frame_mut()
                    .unwrap()
                    ._set_last_opcode_pc(prev_pc);
                assert!(jvm.exec_if_acmp(*opcode).unwrap().is_none());
                if *opcode == crate::runtime::opcode::OpCode::IFACMPEQ {
                    if v1 == v2 {
                        assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + posjump);
                    } else {
                        assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + negjump);
                    }
                }
                if *opcode == crate::runtime::opcode::OpCode::IFACMPNE {
                    if v1 != v2 {
                        assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + posjump);
                    } else {
                        assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + negjump);
                    }
                }
            }
        }
    }

    #[test]
    fn test_exec_areturn() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestReference", "areturn", "()V", vec![])
            .unwrap();
        jvm.push_frame_from_class("TestReference", "if_acmp", "()V", vec![])
            .unwrap();

        let v1 = 23;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::ObjectRef(v1));

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestReference");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "if_acmp");

        assert!(Some(types::Type::ObjectRef(v1)) == jvm.exec_areturn().unwrap());

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestReference");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "areturn");

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::ObjectRef(v1));
    }
}
