use crate::vm::class_loader::ClassLoader;
use crate::vm::classes::{
    Class, ClassIdx, CodePtr, ConstantPoolIdx, ConstantPoolValue, Method, MethodInClassIdx,
};
use crate::vm::memory::{HeapMemory, HeapPtr};
use crate::vm::program::Program;
use crate::vm::stack::{Stack, Value};
use std::cell::{Ref, RefCell, RefMut};
use std::time::SystemTime;

pub struct VM {
    pub stack: Stack,
    pub heap: HeapMemory,

    class_loader: ClassLoader,
    pub(crate) program: RefCell<Program>,
    pub(crate) start_time: SystemTime,
}

impl VM {
    pub fn new(classpath: &str) -> Self {
        let vm = Self {
            class_loader: ClassLoader::new(classpath),
            start_time: SystemTime::now(),
            stack: Stack::new(),
            heap: HeapMemory::new(),
            program: RefCell::new(Program::default()),
        };
        vm.program.borrow_mut().init();
        vm
    }

    pub fn get_or_load_class_idx(&self, name: &String) -> ClassIdx {
        let mut program_mut = self.program.borrow_mut();
        if let Some(idx) = program_mut.class_names_to_idxs.get(name) {
            *idx
        } else {
            self.class_loader
                .load_class_into(name, self, &mut program_mut)
        }
    }

    pub fn get_or_load_class(&self, name: &str) -> Ref<Class> {
        let idx = self.get_or_load_class_idx(&name.to_string());
        self.get_class(idx)
    }

    pub fn get_class(&self, idx: ClassIdx) -> Ref<Class> {
        let program_ref = self.program.borrow();
        return Ref::map(program_ref, |x| &x.classes[idx]);
    }

    pub fn get_method_mut(&self, idx: ClassIdx, method_idx: MethodInClassIdx) -> RefMut<Method> {
        let program_ref = self.program.borrow_mut();
        return RefMut::map(program_ref, |x| &mut x.classes[idx].methods[method_idx]);
    }

    pub fn get_method(&self, idx: ClassIdx, method_idx: MethodInClassIdx) -> Ref<Method> {
        let program_ref = self.program.borrow();
        return Ref::map(program_ref, |x| &x.classes[idx].methods[method_idx]);
    }

    pub fn new_object(&self, class_idx: ClassIdx) -> HeapPtr {
        let program = self.program.borrow();
        let class = &program.classes[class_idx];
        let obj_ptr = self.heap.new_object(class_idx, class.fields.len() as u16);
        for field in &class.fields {
            self.heap.new_object_field(field.value_type.default_value())
        }
        return obj_ptr;
    }

    pub fn new_object_array(&self, class_idx: ClassIdx, length: i32) -> HeapPtr {
        let arr_ptr = self.heap.new_object_array(class_idx, length);
        for i in 0..length {
            let o_ptr = self.new_object(class_idx);
            self.heap
                .set_array_element(arr_ptr, i as usize, Value::Reference(o_ptr));
        }
        arr_ptr
    }

    pub fn new_string(&self, string_value: &String) -> HeapPtr {
        let obj_ptr = self.heap.new_object(1, 1);
        self.heap.append(string_value);
        return obj_ptr;
    }

    pub fn code_read_u8(&self, code_ptr: CodePtr) -> u8 {
        self.program.borrow().code[code_ptr]
    }

    pub fn get_constant_pool_value(&self, cpi: ConstantPoolIdx) -> ConstantPoolValue {
        self.program.borrow().constant_pool[cpi].clone()
    }

    pub fn set_constant_pool_value(&self, cpi: ConstantPoolIdx, value: ConstantPoolValue) {
        self.program.borrow_mut().constant_pool[cpi] = value
    }
}
