use crate::vm::classes::{
    Class, ClassIdx, ConstantPoolValue, FieldIdx, MethodInClassIdx, NativeMethod,
};
use crate::vm::rvm_class::RvmClass;
use std::collections::HashMap;

#[derive(Default)]
pub struct Program {
    pub classes: Vec<Class>,
    pub constant_pool: Vec<ConstantPoolValue>,
    pub code: Vec<u8>,

    pub class_names_to_idxs: HashMap<String, ClassIdx>,
    pub field_names_to_idxs: HashMap<(ClassIdx, String), FieldIdx>,
    pub method_names_to_idxs: HashMap<(ClassIdx, String, String), MethodInClassIdx>,

    pub native_methods: Vec<Box<dyn NativeMethod>>,
}

impl Program {
    pub(crate) fn init(&mut self) {
        self.code.push(0x00);
        self.code.push(177); //1 - return for empty methods
        self.init_java_lang();
        self.native_methods.push(Box::new(RvmClass {}));
        self.constant_pool.push(ConstantPoolValue::Skip); //skip 0 element, as starts from 1
    }
}
