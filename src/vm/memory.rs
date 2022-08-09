use crate::vm::classes::{ClassIdx, FieldIdx};
use crate::vm::stack::{Type, Value};
use std::cell::{RefCell, RefMut};
use std::convert::TryInto;

//is very simplified heap
#[derive(Default)]
pub struct HeapMemory {
    values: RefCell<Vec<Value>>,
}

pub type HeapPtr = usize;

impl HeapMemory {
    pub fn new() -> Self {
        let heap: HeapMemory = Default::default();
        heap.values.borrow_mut().push(Value::Void);
        heap
    }

    pub fn append(&self, data: &String) {
        self.values.borrow_mut().push(Value::String(data.clone()))
    }

    pub fn new_object(&self, class_index: ClassIdx, fields_count: u16) -> HeapPtr {
        //todo: look for empty spaces first
        //todo: allocate size for all fields
        let mut values = self.values.borrow_mut();
        values.push(Value::ClassIndex(class_index, fields_count));
        return values.len() - 1;
    }

    pub fn free(&self, ptr: HeapPtr) {
        let mut values = self.values.borrow_mut();
        Self::_free(ptr, &mut values);
        if let Some(first_not_empty_from_end) = values.iter().rposition(|x| *x != Value::Void) {
            values.truncate(first_not_empty_from_end)
        }
    }

    fn _free(ptr: HeapPtr, values: &mut RefMut<Vec<Value>>) {
        if ptr == 0 || ptr >= values.len() {
            return;
        }
        let value = values.get(ptr).unwrap().clone();
        values[ptr] = Value::Void;
        match value {
            Value::Reference(reference) => {
                Self::_free(reference, values);
            }
            Value::ClassIndex(_, fields) => {
                for p in 0..=(fields as usize) {
                    Self::_free(ptr + p, values);
                }
            }
            Value::ArrayOf(_, _) => {
                let len: i32 = values[ptr + 1].clone().try_into().unwrap();
                for p in 0..=(1 + len as usize) {
                    Self::_free(ptr + p, values);
                }
            }
            _ => {}
        }
    }

    pub fn new_object_array(&self, class_index: ClassIdx, count: i32) -> HeapPtr {
        let mut values = self.values.borrow_mut();
        let ptr = values.len();
        values.push(Value::ArrayOf(Type::Reference, class_index));
        values.push(Value::Int(count));
        for _ in 0..count {
            values.push(Value::Reference(0));
        }
        return ptr;
    }

    pub fn get_array_element(&self, ptr: HeapPtr, idx: usize) -> Value {
        self.values.borrow()[ptr + 2 + idx].clone()
    }

    pub fn set_array_element(&self, ptr: HeapPtr, idx: usize, value: Value) {
        self.values.borrow_mut()[ptr + 2 + idx] = value
    }

    pub fn inspect(&self) -> Vec<Value> {
        return self.values.borrow().clone();
    }

    pub fn new_object_field(&self, value: Value) {
        self.values.borrow_mut().push(value);
    }

    pub fn put_value(&self, value: Value) -> HeapPtr {
        let mut values_mut = self.values.borrow_mut();
        values_mut.push(value);
        values_mut.len() - 1
    }

    pub fn get_value(&self, reference: HeapPtr) -> Value {
        return self.values.borrow()[reference].clone();
    }

    pub fn get_field(&self, reference: HeapPtr, field_idx: FieldIdx) -> Value {
        return self.values.borrow()[reference + 1 + field_idx].clone();
    }

    pub fn set_field(&self, reference: HeapPtr, field_idx: FieldIdx, value: Value) {
        self.values.borrow_mut()[reference + 1 + field_idx] = value;
    }
}
