use crate::vm::classes::NativeMethod;
use crate::vm::java_lang::{INTEGER_CLASS_IDX, STRING_CLASS_IDX};
use crate::vm::memory::HeapMemory;
use crate::vm::stack::Value;
use crate::VM;
use log::info;
use std::time::SystemTime;

pub struct RvmClass;

const RVM_CLASS_NAME: &str = "io/github/rvm/RVM";
const PRINT: &str = "print";
const PRINTLN: &str = "println";
const LOG_STATE: &str = "logState";
const TICK: &str = "tick";
const HEAP_SIZE: &str = "heapSize";

impl NativeMethod for RvmClass {
    fn invoke(
        &self,
        vm: &VM,
        class_name: &String,
        name: &String,
        arguments: Vec<Value>,
    ) -> Option<Value> {
        match (class_name.as_str(), name.as_str()) {
            (RVM_CLASS_NAME, PRINT) => self.print(&arguments, &vm.heap),
            (RVM_CLASS_NAME, PRINTLN) => self.println(),
            (RVM_CLASS_NAME, LOG_STATE) => self.log_state(vm),
            (RVM_CLASS_NAME, HEAP_SIZE) => Some(Value::Int(vm.heap.inspect().len() as i32)),
            (RVM_CLASS_NAME, TICK) => Some(Value::Int(
                SystemTime::now()
                    .duration_since(vm.start_time)
                    .unwrap()
                    .as_millis() as i32,
            )),
            _ => None,
        }
    }
}

impl RvmClass {
    fn println(&self) -> Option<Value> {
        println!();
        return Some(Value::Void);
    }

    fn log_state(&self, vm: &VM) -> Option<Value> {
        info!("--------------- STACK -----------------");
        let frames = vm.stack.inspect();
        for frame in frames.iter().rev() {
            let (ci, mi) = frame.class_method_idxs;
            info!(
                " [ {} {} ]",
                vm.get_class(ci).name,
                vm.get_class(ci).methods[mi].name
            );
            for item in frame.inspect_stack().iter().rev() {
                info!("  > {}", item.short())
            }
            let locals = frame.inspect_locals();
            for i in 0..locals.len() {
                info!("  local({}) = {}", i, locals[i].short())
            }
        }
        let heap_values = vm.heap.inspect();
        info!("------------ HEAP [{:4}] --------------", heap_values.len());
        let mut str = "".to_string();
        for i in 0..heap_values.len() {
            str += " [";
            str += &heap_values[i].short();
            str += " ]";
            if i % 10 == 9 {
                info!("{}", str);
                str = "".to_string()
            }
        }
        if str != "" {
            info!("{}", str);
        }

        info!("-------------- CLASSES ----------------");
        let classes = vm.program.borrow().classes.clone();
        for i in 0..classes.len() {
            info!("{:4} {}", i, classes[i].name)
        }
        info!("--------------- <eof> -----------------");
        return Some(Value::Void);
    }

    fn print(&self, arguments: &Vec<Value>, heap: &HeapMemory) -> Option<Value> {
        for arg in arguments {
            match arg {
                Value::Byte(v) => print!("{}", v),
                Value::Short(v) => print!("{}", v),
                Value::Int(v) => print!("{}", v),
                Value::Long(v) => print!("{}", v),
                Value::Char(v) => print!("{}", v),
                Value::Float(v) => print!("{}", v),
                Value::Double(v) => print!("{}", v),
                Value::Boolean(v) => print!("{}", v),
                Value::String(v) => print!("{}", v),
                Value::Reference(0) => print!("null"),
                Value::Reference(heap_ptr) => {
                    match (heap.get_value(*heap_ptr), heap.get_field(*heap_ptr, 0)) {
                        (Value::ClassIndex(STRING_CLASS_IDX, _), Value::String(str)) => {
                            print!("{}", str)
                        }
                        (Value::ClassIndex(INTEGER_CLASS_IDX, _), Value::Int(int)) => {
                            print!("{}", int)
                        }
                        _ => panic!(
                            "Don't know how to serialize to string: {:?} {:?}",
                            heap.get_value(*heap_ptr),
                            heap.get_field(*heap_ptr, 0)
                        ),
                    }
                }
                _ => {}
            }
        }
        return Some(Value::Void);
    }
}

impl Value {
    fn short(&self) -> String {
        return match self {
            Value::Byte(b) => format!("b{:5}", b),
            Value::Short(s) => format!("s{:5}", s),
            Value::Int(i) => format!("i{:5}", i),
            Value::Long(l) => format!("l{:5}", l),
            Value::Char(c) => format!("c{:5}", c),
            Value::Float(f) => format!("f{:5}", f),
            Value::Double(d) => format!("d{:5}", d),
            Value::Boolean(bool) => format!("{:6}", if *bool > 0 { "true" } else { "false" }),
            Value::ReturnType => format!("retut"),
            Value::Reference(0) => format!("P null",),
            Value::Reference(ptr) => format!("P{:5}", ptr),
            Value::ClassIndex(ci, _) => format!("C{:5}", ci),
            Value::ArrayOf(ty, _) => format!("A{:?}", ty),
            Value::String(s) => format!(
                "{:6}",
                s.replace("\n", "\\n").chars().take(6).collect::<String>()
            ),
            Value::Void => " ---- ".to_string(),
        };
    }
}
