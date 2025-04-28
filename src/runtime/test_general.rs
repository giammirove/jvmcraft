#[cfg(test)]
mod tests {
    use crate::runtime::{jvm::JVM, types};

    #[test]
    fn test_exec_instanceof() {
        // positive test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof", "()I", vec![])
                .unwrap();

            for _ in 0..3 {
                assert!(jvm.step().unwrap().is_none());
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let string_obj = jvm.heap.alloc_string(&mut jvm.class_loader, "wow").unwrap();
            jvm.get_current_frame_mut().unwrap().push_stack(string_obj);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 1);
        }

        // negative test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof", "()I", vec![])
                .unwrap();

            for _ in 0..3 {
                assert!(jvm.step().unwrap().is_none());
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let array_obj = jvm.heap.alloc_array_primitive("I", vec![], 0).unwrap();
            jvm.get_current_frame_mut().unwrap().push_stack(array_obj);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 0);
        }
    }

    #[test]
    fn test_exec_instanceof_inheritance() {
        // steps to do to reach the testing point (instanceof opcode)
        let steps = 42;

        // positive test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let local0 = jvm
                .get_current_frame()
                .unwrap()
                .get_local(0)
                .unwrap()
                .clone();
            jvm.get_current_frame_mut().unwrap().push_stack(local0);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 1);
        }

        // negative test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let local1 = jvm
                .get_current_frame()
                .unwrap()
                .get_local(1)
                .unwrap()
                .clone();
            jvm.get_current_frame_mut().unwrap().push_stack(local1);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 0);
        }

        // positive test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let local3 = jvm
                .get_current_frame()
                .unwrap()
                .get_local(3)
                .unwrap()
                .clone();
            jvm.get_current_frame_mut().unwrap().push_stack(local3);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 1);
        }

        // negative test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let local2 = jvm
                .get_current_frame()
                .unwrap()
                .get_local(2)
                .unwrap()
                .clone();
            jvm.get_current_frame_mut().unwrap().push_stack(local2);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 0);
        }

        // positive test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            // skip instanceof
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip instanceof index
            assert!(jvm.get_current_frame_mut().unwrap().read_ju2().is_ok());
            // skip ifeq
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip ifeq arguments
            assert!(jvm.get_current_frame_mut().unwrap().read_ju2().is_ok());
            // skip iconst1
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip ireturn
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip aload_2
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip instanceof
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let local1 = jvm
                .get_current_frame()
                .unwrap()
                .get_local(1)
                .unwrap()
                .clone();
            jvm.get_current_frame_mut().unwrap().push_stack(local1);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 1);
        }
        // negative test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            // skip instanceof
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip instanceof index
            assert!(jvm.get_current_frame_mut().unwrap().read_ju2().is_ok());
            // skip ifeq
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip ifeq arguments
            assert!(jvm.get_current_frame_mut().unwrap().read_ju2().is_ok());
            // skip iconst1
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip ireturn
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip aload_2
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());
            // skip instanceof
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            let local2 = jvm
                .get_current_frame()
                .unwrap()
                .get_local(2)
                .unwrap()
                .clone();
            jvm.get_current_frame_mut().unwrap().push_stack(local2);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 0);
        }

        // null test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            jvm.get_current_frame_mut()
                .unwrap()
                .push_stack(types::Type::Null);

            assert!(jvm.exec_instanceof().unwrap().is_none());

            let result = jvm.pop_ioperand().unwrap();
            assert!(result == 0);
        }

        // error test
        {
            let mut jvm = JVM::mock("tests/classes/", vec![], false).unwrap();
            jvm.push_frame_from_class("TestGeneral", "_instanceof_inheritance", "()I", vec![])
                .unwrap();

            for _ in 0..steps {
                jvm.step().unwrap();
            }
            assert!(jvm.get_current_frame_mut().unwrap().read_ju1().is_ok());

            jvm.get_current_frame_mut()
                .unwrap()
                .push_stack(types::Type::Integer(23));

            assert!(jvm.exec_instanceof().is_err())
        }
    }
}
