#[cfg(test)]
mod tests {
    use crate::runtime::{jvm::JVM, types};

    #[test]
    fn test_exec_pop_ioperand_integer() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "pop_ioperand", "()V", vec![])
            .unwrap();

        let test = types::Type::Integer(23);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        let local = jvm.pop_ioperand().unwrap();
        assert!(test == types::Type::Integer(local));
    }

    #[test]
    fn test_exec_pop_ioperand_short() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "pop_ioperand", "()V", vec![])
            .unwrap();

        let test = types::Type::Short(23);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        let local = jvm.pop_ioperand().unwrap();
        assert!(test == types::Type::Short(local as i16));
    }

    #[test]
    fn test_exec_pop_ioperand_byte() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "pop_ioperand", "()V", vec![])
            .unwrap();

        let test = types::Type::Byte(23);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        let local = jvm.pop_ioperand().unwrap();
        assert!(test == types::Type::Byte(local as i8));
    }

    #[test]
    fn test_exec_pop_ioperand_character() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "pop_ioperand", "()V", vec![])
            .unwrap();

        let test = types::Type::Character(23);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        let local = jvm.pop_ioperand().unwrap();
        assert!(test == types::Type::Character(local as i8));
    }

    #[test]
    fn test_exec_pop_ioperand_boolean() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "pop_ioperand", "()V", vec![])
            .unwrap();

        let test = types::Type::Boolean(true);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        let local = jvm.pop_ioperand().unwrap();
        assert!(test == types::Type::Boolean(local != 0));
    }

    #[test]
    fn test_exec_pop_ioperand_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "pop_ioperand", "()V", vec![])
            .unwrap();

        let test = types::Type::Null;
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        assert!(jvm.pop_ioperand().is_err());
    }

    #[test]
    fn test_exec_pop_ioperands() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "pop_ioperand", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 24;
        let test = types::Type::Integer(v1);
        let test2 = types::Type::Integer(v2);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        jvm.get_current_frame_mut().unwrap().push_stack(test2);
        let (rv1, rv2) = jvm.pop_ioperands().unwrap();
        assert!(rv1 == v1);
        assert!(rv2 == v2);
    }

    #[test]
    fn test_exec_iload_integer() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "iload", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Integer(23 + i);
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_iload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
    }

    #[test]
    fn test_exec_iload_short() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "iload", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Short(23 + i);
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_iload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
    }

    #[test]
    fn test_exec_iload_byte() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "iload", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Byte(23 + i);
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_iload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
    }

    #[test]
    fn test_exec_iload_character() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "iload", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Character(23 + i);
            jvm.get_current_frame_mut()
                .unwrap()
                .set_local(i as usize, test);
            assert!(jvm.exec_iload(i as usize).unwrap().is_none());
            let check = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == check);
        }
    }

    #[test]
    fn test_exec_iload_long() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "iload", "()V", vec![])
            .unwrap();

        let test = types::Type::Long(23);
        jvm.get_current_frame_mut().unwrap().set_local(0, test);
        assert!(jvm.exec_iload(0).is_err())
    }

    #[test]
    fn test_exec_istore() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "istore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Integer(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
        for i in 0..5 {
            let test = types::Type::Short(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
        for i in 0..5 {
            let test = types::Type::Byte(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).unwrap().is_none());
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
            assert!(jvm.exec_istore(i as usize).is_err());
        }
    }

    #[test]
    fn test_exec_istore_integer() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "istore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Integer(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
    }

    #[test]
    fn test_exec_istore_short() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "istore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Short(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
    }

    #[test]
    fn test_exec_istore_byte() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "istore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Byte(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
    }

    #[test]
    fn test_exec_istore_character() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "istore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Character(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).unwrap().is_none());
            let local = jvm
                .get_current_frame()
                .unwrap()
                .get_local(i as usize)
                .unwrap();
            assert!(test == *local);
        }
    }

    #[test]
    fn test_exec_istore_long() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "istore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Long(23 + i);
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).is_err());
        }
    }

    #[test]
    fn test_exec_istore_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "istore", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            let test = types::Type::Null;
            jvm.get_current_frame_mut().unwrap().push_stack(test);
            assert!(jvm.exec_istore(i as usize).is_err());
        }
    }

    #[test]
    fn test_exec_iconst() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "iconst", "()V", vec![])
            .unwrap();

        for i in 0..5 {
            assert!(jvm.exec_iconst(i).unwrap().is_none());
            let test = types::Type::Integer(i);
            let local = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
            assert!(test == local);
        }
    }

    #[test]
    fn test_exec_iushr() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "iushr", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 24;
        let test = types::Type::Integer(v1);
        let test2 = types::Type::Integer(v2);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        jvm.get_current_frame_mut().unwrap().push_stack(test2);

        assert!(jvm.exec_iushr().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 >> (v2 & 0x1F)))
    }

    #[test]
    fn test_exec_ishl() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "ishl", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 24;
        let test = types::Type::Integer(v1);
        let test2 = types::Type::Integer(v2);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        jvm.get_current_frame_mut().unwrap().push_stack(test2);

        assert!(jvm.exec_ishl().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 << (v2 & 0x1F)))
    }

    #[test]
    fn test_exec_ishr() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "ishr", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 24;
        let test = types::Type::Integer(v1);
        let test2 = types::Type::Integer(v2);
        jvm.get_current_frame_mut().unwrap().push_stack(test);
        jvm.get_current_frame_mut().unwrap().push_stack(test2);

        assert!(jvm.exec_ishr().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 >> (v2 & 0x1F)))
    }

    #[test]
    fn test_exec_i2f() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "i2", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let test = types::Type::Integer(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_i2f().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Float(v1 as f32))
    }

    #[test]
    fn test_exec_i2d() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "i2", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let test = types::Type::Integer(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_i2d().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Double(v1 as f64))
    }

    #[test]
    fn test_exec_i2l() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "i2", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let test = types::Type::Integer(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_i2l().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Long(v1 as i64))
    }

    #[test]
    fn test_exec_i2s() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "i2", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let test = types::Type::Integer(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_i2s().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Short(v1 as i16))
    }

    #[test]
    fn test_exec_i2c() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "i2", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let test = types::Type::Integer(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_i2c().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Character(v1 as i8))
    }

    #[test]
    fn test_exec_i2b() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "i2", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let test = types::Type::Integer(v1);
        jvm.get_current_frame_mut().unwrap().push_stack(test);

        assert!(jvm.exec_i2b().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Byte(v1 as i8))
    }

    #[test]
    fn test_exec_if() {
        let offset = 4;
        let values = vec![-23, 0, 23];
        let opcodes = vec![
            crate::runtime::opcode::OpCode::IFEQ,
            crate::runtime::opcode::OpCode::IFNE,
            crate::runtime::opcode::OpCode::IFLT,
            crate::runtime::opcode::OpCode::IFLE,
            crate::runtime::opcode::OpCode::IFGT,
            crate::runtime::opcode::OpCode::IFGE,
        ];
        for opcode in opcodes {
            for value in &values {
                let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
                jvm.push_frame_from_class("TestInteger", "_if", "()V", vec![])
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

                let test = types::Type::Integer(*value);
                jvm.get_current_frame_mut().unwrap().push_stack(test);

                assert!(jvm.exec_if(opcode).unwrap().is_none());

                if (opcode == crate::runtime::opcode::OpCode::IFEQ && *value == 0)
                    || (opcode == crate::runtime::opcode::OpCode::IFNE && *value != 0)
                    || (opcode == crate::runtime::opcode::OpCode::IFLT && *value < 0)
                    || (opcode == crate::runtime::opcode::OpCode::IFLE && *value <= 0)
                    || (opcode == crate::runtime::opcode::OpCode::IFGT && *value > 0)
                    || (opcode == crate::runtime::opcode::OpCode::IFGE && *value >= 0)
                {
                    assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + offset);
                } else {
                    assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + 3);
                }
            }
        }
    }

    #[test]
    fn test_exec_if_icmp() {
        let offset = 4;
        let values1 = vec![-10, 23, 7, 3, 8, 42];
        let values2 = vec![-10, 3, 7, 5, 3, 23];
        assert!(values1.len() == values2.len());
        let opcodes = vec![
            crate::runtime::opcode::OpCode::IFICMPEQ,
            crate::runtime::opcode::OpCode::IFICMPNE,
            crate::runtime::opcode::OpCode::IFICMPLT,
            crate::runtime::opcode::OpCode::IFICMPLE,
            crate::runtime::opcode::OpCode::IFICMPGT,
            crate::runtime::opcode::OpCode::IFICMPGE,
        ];
        for opcode in opcodes {
            for i in 0..values1.len() {
                let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
                jvm.push_frame_from_class("TestInteger", "_if", "()V", vec![])
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

                let test1 = types::Type::Integer(values1[i]);
                jvm.get_current_frame_mut().unwrap().push_stack(test1);
                let test2 = types::Type::Integer(values2[i]);
                jvm.get_current_frame_mut().unwrap().push_stack(test2);

                assert!(jvm.exec_if_icmp(opcode).unwrap().is_none());

                if (opcode == crate::runtime::opcode::OpCode::IFICMPEQ && values1[i] == values2[i])
                    || (opcode == crate::runtime::opcode::OpCode::IFICMPNE
                        && values1[i] != values2[i])
                    || (opcode == crate::runtime::opcode::OpCode::IFICMPLT
                        && values1[i] < values2[i])
                    || (opcode == crate::runtime::opcode::OpCode::IFICMPLE
                        && values1[i] <= values2[i])
                    || (opcode == crate::runtime::opcode::OpCode::IFICMPGT
                        && values1[i] > values2[i])
                    || (opcode == crate::runtime::opcode::OpCode::IFICMPGE
                        && values1[i] >= values2[i])
                {
                    assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + offset);
                } else {
                    assert!(jvm.get_current_frame().unwrap().get_pc() == prev_pc + 3);
                }
            }
        }
    }

    #[test]
    fn test_exec_iadd() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_iadd().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 + v2));
    }

    #[test]
    fn test_exec_iadd_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_iadd().is_err());
    }

    #[test]
    fn test_exec_isub() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_isub().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 - v2));
    }

    #[test]
    fn test_exec_isub_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_isub().is_err());
    }

    #[test]
    fn test_exec_idiv() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_idiv().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 / v2));
    }

    #[test]
    fn test_exec_idiv_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_idiv().is_err());
    }

    #[test]
    fn test_exec_idiv_zero_division() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(0));

        assert!(jvm.exec_idiv().is_err());
    }

    #[test]
    fn test_exec_irem() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_irem().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 % v2));
    }

    #[test]
    fn test_exec_irem_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_irem().is_err());
    }

    #[test]
    fn test_exec_irem_zero_division() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(0));

        assert!(jvm.exec_irem().is_err());
    }

    #[test]
    fn test_exec_imul() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_imul().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 * v2));
    }

    #[test]
    fn test_exec_imul_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_imul().is_err());
    }

    #[test]
    fn test_exec_ineg() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));

        assert!(jvm.exec_ineg().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(-v1));
    }

    #[test]
    fn test_exec_ineg_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_ineg().is_err());
    }

    #[test]
    fn test_exec_iinc() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        for _ in 0..4 {
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
        }

        let v1 = 23;
        jvm.get_current_frame_mut()
            .unwrap()
            .set_local(0, types::Type::Integer(v1));

        assert!(jvm.exec_iinc().unwrap().is_none());

        let local0 = jvm.get_current_frame().unwrap().get_local(0).unwrap();
        assert!(local0 == &types::Type::Integer(v1 + 2));
    }

    #[test]
    fn test_exec_iinc_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        for _ in 0..4 {
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
        }

        jvm.get_current_frame_mut()
            .unwrap()
            .set_local(0, types::Type::Null);

        assert!(jvm.exec_iinc().is_err());
    }

    #[test]
    fn test_exec_ixor() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_ixor().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 ^ v2));
    }

    #[test]
    fn test_exec_ixor_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_ixor().is_err());
    }

    #[test]
    fn test_exec_ior() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_ior().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 | v2));
    }

    #[test]
    fn test_exec_ior_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_ior().is_err());
    }

    #[test]
    fn test_exec_iand() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        let v2 = 25;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v2));

        assert!(jvm.exec_iand().unwrap().is_none());

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1 & v2));
    }

    #[test]
    fn test_exec_iand_null() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);

        assert!(jvm.exec_iand().is_err());
    }

    #[test]
    fn test_exec_iaload() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let elements = vec![types::Type::Integer(23), types::Type::Integer(46)];

        let array = jvm
            .heap
            .alloc_array_primitive("I", elements.clone(), 0)
            .unwrap();

        let index = 0;
        jvm.get_current_frame_mut().unwrap().push_stack(array);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(index));

        assert!(jvm.exec_iaload().unwrap().is_none());

        let check_value = jvm.pop_stack().unwrap();
        assert!(elements[index as usize] == check_value);
    }

    #[test]
    fn test_exec_iaload_not_array() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let index = 0;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(index));

        assert!(jvm.exec_iaload().is_err());
    }

    #[test]
    fn test_exec_iaload_wrong_type() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let elements = vec![types::Type::Float(23.0), types::Type::Float(46.0)];

        let array = jvm
            .heap
            .alloc_array_primitive("F", elements.clone(), 0)
            .unwrap();

        let index = 0;
        jvm.get_current_frame_mut().unwrap().push_stack(array);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(index));

        assert!(jvm.exec_iaload().is_err());
    }

    #[test]
    fn test_exec_iastore() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let array_ref = jvm.heap.alloc_array_primitive("I", vec![], 6).unwrap();

        let index = types::Type::Integer(0);
        let value = types::Type::Integer(23);
        jvm.get_current_frame_mut().unwrap().push_stack(array_ref);
        jvm.get_current_frame_mut().unwrap().push_stack(index);
        jvm.get_current_frame_mut().unwrap().push_stack(value);

        assert!(jvm.exec_iastore().unwrap().is_none());

        let array = jvm
            .heap
            .get_array_instance(array_ref.as_ref().unwrap())
            .unwrap();

        assert!(*array.get(0).unwrap() == value);
    }

    #[test]
    fn test_exec_iastore_not_array() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let index = types::Type::Integer(0);
        let value = types::Type::Integer(23);
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Null);
        jvm.get_current_frame_mut().unwrap().push_stack(index);
        jvm.get_current_frame_mut().unwrap().push_stack(value);

        assert!(jvm.exec_iastore().is_err());
    }

    #[test]
    fn test_exec_iastore_wrong_type() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let array_ref = jvm.heap.alloc_array_primitive("F", vec![], 6).unwrap();

        let index = types::Type::Integer(0);
        let value = types::Type::Float(23.0);
        jvm.get_current_frame_mut().unwrap().push_stack(array_ref);
        jvm.get_current_frame_mut().unwrap().push_stack(index);
        jvm.get_current_frame_mut().unwrap().push_stack(value);

        assert!(jvm.exec_iastore().is_err());
    }

    #[test]
    fn test_exec_ireturn() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "ireturn", "()V", vec![])
            .unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Integer(v1));

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestInteger");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "arithmetic");

        assert!(Some(types::Type::Integer(v1)) == jvm.exec_ireturn().unwrap());

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestInteger");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "ireturn");

        let res = jvm.get_current_frame_mut().unwrap().pop_stack().unwrap();
        assert!(res == types::Type::Integer(v1));
    }

    #[test]
    fn test_exec_ireturn_wrong_type() {
        let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
        jvm.push_frame_from_class("TestInteger", "ireturn", "()V", vec![])
            .unwrap();
        jvm.push_frame_from_class("TestInteger", "arithmetic", "()V", vec![])
            .unwrap();

        let v1 = 23.0;
        jvm.get_current_frame_mut()
            .unwrap()
            .push_stack(types::Type::Float(v1));

        assert!(jvm.get_current_frame().unwrap().get_classname() == "TestInteger");
        assert!(jvm.get_current_frame().unwrap().get_func_name() == "arithmetic");

        assert!(jvm.exec_ireturn().is_err());
    }
}
