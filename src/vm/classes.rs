use crate::vm::memory::HeapPtr;
use crate::vm::stack::{Type, Value};
use crate::vm::vm::VM;
use bitflags::bitflags;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub flags: AccessFlags,
    pub value_type: Type,
}

#[derive(Clone, Default)]
pub struct Method {
    pub name: String,
    pub signature: Signature,
    pub flags: AccessFlags,
    pub code_ptr: CodePtr, //0 means abstract or native
    pub max_locals: u16,
    pub max_stack: u16,
    pub annotation_names: Vec<String>,

    //extra data
    pub mem_entry_ptr: HeapPtr,
}

pub trait NativeMethod {
    fn invoke(
        &self,
        vm: &VM,
        class_name: &String,
        name: &String,
        arguments: Vec<Value>,
    ) -> Option<Value>;
}

bitflags! {
    #[derive(Default)]
    pub struct AccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABTRACT = 0x0400;
        const STRICT = 0x0800;

        // custom
        const TAIL_RECURSION = 0x1000;
        const MEM = 0x2000;
        const AUTO_FREE = 0x4000;
    }
}

pub type ClassIdx = usize;
pub type MethodInClassIdx = usize;
pub type FieldIdx = usize;
pub type CodePtr = usize;
pub type ConstantPoolIdx = usize;

#[derive(Default, Clone)]
pub struct VirtualMethodsTable {
    pub mapping: HashMap<(ClassIdx, MethodInClassIdx), (ClassIdx, MethodInClassIdx)>,
}

#[derive(Clone, Default)]
pub struct Class {
    pub name: String,
    pub super_class_idx: ClassIdx, //0 == Object
    pub vmt: VirtualMethodsTable,
    pub constant_pool_idx: ConstantPoolIdx,
    pub fields: Vec<Field>, //both types and default values.
    pub methods: Vec<Method>,
}

#[derive(Eq, PartialEq, Clone, Default)]
pub struct Signature {
    pub return_type: Type,
    pub arguments: Vec<Type>,
}

#[derive(Debug, Clone)]
pub enum ConstantPoolValue {
    Class(ClassIdx),
    FieldRef(ClassIdx, FieldIdx),
    MethodRef(ClassIdx, MethodInClassIdx),
    String(Value),
    Const(Value),

    UnresolvedClassRef {
        class_name: String,
    },
    UnresolvedFieldRef {
        class_name: String,
        field_name: String,
    },
    UnresolvedMethodRef {
        class_name: String,
        method_name: String,
        signature: String,
    },

    //unsupported yet
    Unsupported,
    Skip,
}
