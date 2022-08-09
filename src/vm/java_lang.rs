use crate::vm::classes::{AccessFlags, Class, ClassIdx, Field, Method, NativeMethod, Signature};
use crate::vm::memory::HeapPtr;
use crate::vm::program::Program;
use crate::vm::stack::{Type, Value};
use crate::VM;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::BitOr;

const OBJECT_CLASS_IDX: ClassIdx = 0;
pub const STRING_CLASS_IDX: ClassIdx = 1;
const STRING_BUILDER_CLASS_IDX: ClassIdx = 2;
pub const INTEGER_CLASS_IDX: ClassIdx = 3;

impl Program {
    //todo: more helpers to create "rust" java classes
    pub(crate) fn init_java_lang(&mut self) {
        self.classes.push(Class {
            name: "java/lang/Object".to_string(),
            methods: vec![
                Method {
                    name: INIT_METHOD.to_string(),
                    signature: Signature {
                        return_type: Type::Void,
                        arguments: vec![],
                    },
                    code_ptr: 1,
                    ..Default::default()
                },
                Method {
                    name: EQUALS.to_string(),
                    signature: Signature {
                        return_type: Type::Boolean,
                        arguments: vec![Type::Reference],
                    },
                    flags: AccessFlags::NATIVE,
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        self.class_names_to_idxs
            .insert("java/lang/Object".to_string(), OBJECT_CLASS_IDX);
        self.method_names_to_idxs.insert(
            (OBJECT_CLASS_IDX, INIT_METHOD.to_string(), "()V".to_string()),
            0,
        );
        self.method_names_to_idxs.insert(
            (
                OBJECT_CLASS_IDX,
                EQUALS.to_string(),
                "(Ljava/lang/Object;)Z".to_string(),
            ),
            1,
        );

        self.classes.push(Class {
            name: "java/lang/String".to_string(),
            ..Default::default()
        });
        self.class_names_to_idxs
            .insert("java/lang/String".to_string(), STRING_CLASS_IDX);

        self.classes.push(Class {
            name: STRING_BUILDER_CLASS.to_string(),
            super_class_idx: 0,
            vmt: Default::default(), // empty for now
            constant_pool_idx: 0,
            fields: vec![Field {
                name: "buffer".to_string(),
                flags: AccessFlags::empty(),
                value_type: Type::Reference,
            }],
            methods: vec![
                Method {
                    name: INIT_METHOD.to_string(),
                    signature: Signature {
                        return_type: Type::Void,
                        arguments: vec![],
                    },
                    flags: AccessFlags::NATIVE,
                    ..Default::default()
                },
                Method {
                    name: APPEND_METHOD.to_string(),
                    signature: Signature {
                        return_type: Type::Reference,
                        arguments: vec![Type::Reference],
                    },
                    flags: AccessFlags::NATIVE,
                    ..Default::default()
                },
                Method {
                    name: TO_STRING.to_string(),
                    signature: Signature {
                        return_type: Type::Reference,
                        arguments: vec![],
                    },
                    flags: AccessFlags::NATIVE,
                    ..Default::default()
                },
            ],
        });

        self.class_names_to_idxs
            .insert(STRING_BUILDER_CLASS.to_string(), STRING_BUILDER_CLASS_IDX);
        self.method_names_to_idxs.insert(
            (
                STRING_BUILDER_CLASS_IDX,
                INIT_METHOD.to_string(),
                "()V".to_string(),
            ),
            0,
        );
        self.method_names_to_idxs.insert(
            (
                STRING_BUILDER_CLASS_IDX,
                APPEND_METHOD.to_string(),
                "(Ljava/lang/String;)Ljava/lang/StringBuilder;".to_string(),
            ),
            1,
        );
        self.method_names_to_idxs.insert(
            (
                STRING_BUILDER_CLASS_IDX,
                TO_STRING.to_string(),
                "()Ljava/lang/String;".to_string(),
            ),
            2,
        );
        self.method_names_to_idxs.insert(
            (
                STRING_BUILDER_CLASS_IDX,
                APPEND_METHOD.to_string(),
                "(I)Ljava/lang/StringBuilder;".to_string(),
            ),
            1,
        ); //map to same

        self.classes.push(Class {
            name: INTEGER_CLASS.to_string(),
            super_class_idx: 0,
            vmt: Default::default(), // empty for now
            constant_pool_idx: 0,
            fields: vec![Field {
                name: "_".to_string(),
                flags: Default::default(),
                value_type: Type::Int,
            }],
            methods: vec![
                Method {
                    name: VALUE_OF.to_string(),
                    signature: Signature {
                        return_type: Type::Reference,
                        arguments: vec![Type::Int],
                    },
                    flags: AccessFlags::NATIVE.bitor(AccessFlags::STATIC),
                    ..Default::default()
                },
                Method {
                    name: INT_VALUE.to_string(),
                    signature: Signature {
                        return_type: Type::Int,
                        arguments: vec![],
                    },
                    flags: AccessFlags::NATIVE,
                    ..Default::default()
                },
            ],
        });
        self.class_names_to_idxs
            .insert(INTEGER_CLASS.to_string(), INTEGER_CLASS_IDX);
        self.method_names_to_idxs.insert(
            (
                INTEGER_CLASS_IDX,
                VALUE_OF.to_string(),
                "(I)Ljava/lang/Integer;".to_string(),
            ),
            0,
        );
        self.method_names_to_idxs.insert(
            (INTEGER_CLASS_IDX, INT_VALUE.to_string(), "()I".to_string()),
            1,
        );

        self.native_methods.push(Box::new(JavaLang::default()));
    }
}

#[derive(Default)]
struct JavaLang {
    ints: RefCell<Vec<HeapPtr>>,

    more_ints: RefCell<HashMap<i32, HeapPtr>>,
}

const STRING_BUILDER_CLASS: &str = "java/lang/StringBuilder";
const INTEGER_CLASS: &str = "java/lang/Integer";
const OBJECT_CLASS: &str = "java/lang/Object";

const INIT_METHOD: &str = "<init>";
const EQUALS: &str = "equals";
const APPEND_METHOD: &str = "append";
const TO_STRING: &str = "toString";
const VALUE_OF: &str = "valueOf";
const INT_VALUE: &str = "intValue";

impl NativeMethod for JavaLang {
    fn invoke(
        &self,
        vm: &VM,
        class_name: &String,
        name: &String,
        arguments: Vec<Value>,
    ) -> Option<Value> {
        match (class_name.as_str(), name.as_str()) {
            (OBJECT_CLASS, EQUALS) => {
                let value1 = &arguments[0];
                let value2 = &arguments[1];
                return Some(Value::Boolean(if value1 == value2 { 1 } else { 0 }));
            }
            (INTEGER_CLASS, INT_VALUE) => {
                let ptr: HeapPtr = arguments[0].clone().try_into().unwrap();
                Some(vm.heap.get_field(ptr, 0))
            }
            (INTEGER_CLASS, VALUE_OF) => {
                let value = &arguments[0];
                let int: i32 = value.clone().try_into().unwrap();
                if int >= 0 && int < 50 {
                    let int = int as usize;
                    let mut ints = self.ints.borrow_mut();
                    while int >= ints.len() {
                        let int_obj_ptr = vm.new_object(INTEGER_CLASS_IDX);
                        vm.heap
                            .set_field(int_obj_ptr, 0, Value::Int(ints.len() as i32));
                        ints.push(int_obj_ptr);
                    }

                    Some(Value::Reference(ints[int]))
                } else {
                    let mut more_ints = self.more_ints.borrow_mut();
                    if let Some(ptr) = more_ints.get(&int) {
                        Some(Value::Reference(*ptr))
                    } else {
                        let int_obj_ptr = vm.new_object(INTEGER_CLASS_IDX);
                        vm.heap.set_field(int_obj_ptr, 0, Value::Int(int));
                        more_ints.insert(int, int_obj_ptr);
                        Some(Value::Reference(int_obj_ptr))
                    }
                }
            }
            (STRING_BUILDER_CLASS, INIT_METHOD) => {
                if let Value::Reference(heap_ptr) = arguments[0] {
                    vm.heap.set_field(heap_ptr, 0, Value::String(String::new()))
                }
                Some(Value::Void)
            }
            (STRING_BUILDER_CLASS, TO_STRING) => {
                if let Value::Reference(heap_ptr) = arguments[0] {
                    let str_ptr = vm.heap.new_object(STRING_CLASS_IDX, 1);
                    let str_value = vm.heap.get_field(heap_ptr, 0);
                    vm.heap.new_object_field(str_value);
                    Some(Value::Reference(str_ptr))
                } else {
                    None
                }
            }
            (STRING_BUILDER_CLASS, APPEND_METHOD) => {
                if let Value::Reference(heap_ptr) = arguments[0] {
                    if let Value::String(mut str) = vm.heap.get_field(heap_ptr, 0) {
                        match &arguments[1] {
                            Value::Byte(v) => str.push_str(&*v.to_string()),
                            Value::Short(v) => str.push_str(&*v.to_string()),
                            Value::Int(v) => str.push_str(&*v.to_string()),
                            Value::Long(v) => str.push_str(&*v.to_string()),
                            Value::Char(v) => str.push_str(&*v.to_string()),
                            Value::Float(v) => str.push_str(&*v.to_string()),
                            Value::Double(v) => str.push_str(&*v.to_string()),
                            Value::Boolean(v) => str.push_str(&*v.to_string()),
                            Value::Reference(hptr) => {
                                //todo: support not only strings
                                if let Value::ClassIndex(STRING_CLASS_IDX, _) =
                                    vm.heap.get_value(*hptr)
                                {
                                    if let Value::String(s) = vm.heap.get_field(*hptr, 0) {
                                        str.push_str(&s);
                                    }
                                }
                            }
                            Value::String(s) => str.push_str(&s),
                            _ => panic!("Not supported"),
                        }
                        vm.heap.set_field(heap_ptr, 0, Value::String(str))
                    }
                }
                Some(arguments[0].clone())
            }
            _ => None,
        }
    }
}
