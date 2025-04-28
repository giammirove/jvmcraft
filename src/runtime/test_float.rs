#[cfg(test)]
mod tests {
    use std::ops::Neg;

    use crate::runtime::{jvm::JVM, types};

    #[test]
    fn test_exec_fload() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "fload", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Float(23.23 + i as f32);
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_fload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
    }

    #[test]
    fn test_exec_fload_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "fload", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Null;
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_fload(i as usize).is_err());
        }
    }

    #[test]
    fn test_exec_fstore() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "fstore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Float(23.23 + i as f32);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_fstore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
    }

    #[test]
    fn test_exec_fstore_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "fstore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Null;
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_fstore(i as usize).is_err());
        }
    }

    #[test]
    fn test_exec_fconst() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "fconst", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            assert!(jvm.exec_fconst(i as f32 + 0.23).unwrap().is_none());
            let test = types::Type::Float(i as f32 + 0.23);
            let local = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == local);
        }
    }

    #[test]
    fn test_exec_f2d() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "f2", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        let test = types::Type::Float(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_f2d().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Double(v1 as f64))
    }

    #[test]
    fn test_exec_f2l() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "f2", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        let test = types::Type::Float(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_f2l().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Long(v1 as i64))
    }

    #[test]
    fn test_exec_f2i() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "f2", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        let test = types::Type::Float(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_f2i().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 as i32))
    }

    #[test]
    fn test_exec_fcmpg() {
        let values1 = vec![-10.10, 23.23, 7.7, 3.3, 8.8, 42.42, f32::NAN];
        let values2 = vec![-10.10, 3.4, 7.7, 5.5, 3.3, 23.23, f32::NAN];
        let exp = vec![0, 1, 0, -1, 1, 1, 1];
        assert!(values1.len() == values2.len());
        assert!(exp.len() == values2.len());
        for i in 0..values1.len() {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestFloat", "_if", "()V", vec![])
                .unwrap();

            // read first instructions
            for _ in 0..7 {
                assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            }
            // prev_pc is the next pc to use
            let prev_pc = jvm.get_current_frame().unwrap().get_pc() - 1;
            // due to internal logic of jump
            jvm.get_current_frame_mut()
                .unwrap()
                ._set_last_opcode_pc(prev_pc);

            let test1 = types::Type::Float(values1[i]);
            jvm.get_current_frame_mut().unwrap().push_stack(test1);
            let test2 = types::Type::Float(values2[i]);
            jvm.get_current_frame_mut().unwrap().push_stack(test2);

            assert!(jvm
                .exec_fcmp(crate::runtime::opcode::OpCode::FCMPG)
                .unwrap()
                .is_none());

            let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(res == types::Type::Integer(exp[i]));
        }
    }

    #[test]
    fn test_exec_fcmpl() {
        let values1 = vec![-10.10, 23.23, 7.7, 3.3, 8.8, 42.42, f32::NAN];
        let values2 = vec![-10.10, 3.4, 7.7, 5.5, 3.3, 23.23, f32::NAN];
        let exp = vec![0, 1, 0, -1, 1, 1, -1];
        assert!(values1.len() == values2.len());
        assert!(exp.len() == values2.len());
        for i in 0..values1.len() {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestFloat", "_if", "()V", vec![])
                .unwrap();

            // read first instructions
            for _ in 0..7 {
                assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            }
            // prev_pc is the next pc to use
            let prev_pc = jvm.get_current_frame().unwrap().get_pc() - 1;
            // due to internal logic of jump
            jvm.get_current_frame_mut()
                .unwrap()
                ._set_last_opcode_pc(prev_pc);

            let test1 = types::Type::Float(values1[i]);
            jvm.get_current_frame_mut().unwrap().push_stack(test1);
            let test2 = types::Type::Float(values2[i]);
            jvm.get_current_frame_mut().unwrap().push_stack(test2);

            assert!(jvm
                .exec_fcmp(crate::runtime::opcode::OpCode::FCMPL)
                .unwrap()
                .is_none());

            let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(res == types::Type::Integer(exp[i]));
        }
    }

    #[test]
    fn test_exec_fcmp_error() {
        let values1 = vec![f32::NAN, 1.0];
        let values2 = vec![0.0, f32::NAN];
        assert!(values1.len() == values2.len());
        for i in 0..values1.len() {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestFloat", "_if", "()V", vec![])
                .unwrap();

            // read first instructions
            for _ in 0..7 {
                assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            }
            // prev_pc is the next pc to use
            let prev_pc = jvm.get_current_frame().unwrap().get_pc() - 1;
            // due to internal logic of jump
            jvm.get_current_frame_mut()
                .unwrap()
                ._set_last_opcode_pc(prev_pc);

            let test1 = types::Type::Float(values1[i]);
            jvm.get_current_frame_mut().unwrap().push_stack(test1);
            let test2 = types::Type::Float(values2[i]);
            jvm.get_current_frame_mut().unwrap().push_stack(test2);

            assert!(jvm.exec_fcmp(crate::runtime::opcode::OpCode::NOP).is_err());
        }
    }

    #[test]
    fn test_exec_fadd() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        let v2 = 25.25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v2));

        assert!(jvm.exec_fadd().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Float(v1 + v2));
    }

    #[test]
    fn test_exec_fadd_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_fadd().is_err());
    }

    #[test]
    fn test_exec_fsub() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        let v2 = 25.25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v2));

        assert!(jvm.exec_fsub().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Float(v1 - v2));
    }

    #[test]
    fn test_exec_fsub_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_fsub().is_err());
    }

    #[test]
    fn test_exec_fdiv() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        let v2 = 25.25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v2));

        assert!(jvm.exec_fdiv().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Float(v1 / v2));
    }

    #[test]
    fn test_exec_fdiv_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_fdiv().is_err());
    }

    #[test]
    fn test_exec_fdiv_zero_division() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.0;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(0.0));

        assert!(jvm.exec_fdiv().is_err());
    }

    #[test]
    fn test_exec_fmul() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        let v2 = 25.25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v2));

        assert!(jvm.exec_fmul().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Float(v1 * v2));
    }

    #[test]
    fn test_exec_fmul_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_fmul().is_err());
    }

    #[test]
    fn test_exec_fneg() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));

        assert!(jvm.exec_fneg().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Float(v1.neg()));
    }

    #[test]
    fn test_exec_fneg_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_fneg().is_err());
    }

    #[test]
    fn test_exec_freturn() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "freturn", "()V", vec![])
            .unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.23;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestFloat");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "arithmetic");

        assert!(Some(types::Type::Float(v1)) == jvm.exec_freturn().unwrap());

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestFloat");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "freturn");

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Float(v1));
    }

    #[test]
    fn test_exec_freturn_wrong_type() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestFloat", "freturn", "()V", vec![])
            .unwrap();
        jvm.push_frame_from_class("TestFloat", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.0;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Long(v1 as i64));

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestFloat");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "arithmetic");

        assert!(jvm.exec_freturn().is_err());
    }
}
