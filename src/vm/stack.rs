use crate::vm::classes::{ClassIdx, CodePtr, ConstantPoolIdx, MethodInClassIdx};
use crate::vm::memory::HeapPtr;
use crate::VM;
use derive_more::TryInto;
use std::cell::{Cell, Ref, RefCell, RefMut};

#[derive(Default, Clone)]
pub struct Stack {
    frames: RefCell<Vec<StackFrame>>,
}

impl Stack {
    pub fn new() -> Self {
        return Default::default();
    }

    pub fn top_frame(&self) -> Ref<StackFrame> {
        let frames = self.frames.borrow();
        Ref::map(frames, |x| x.last().unwrap())
    }

    pub fn top_frame_mut(&self) -> RefMut<StackFrame> {
        let frames = self.frames.borrow_mut();
        RefMut::map(frames, |x| x.last_mut().unwrap())
    }

    pub fn push_frame(&self, stack_size: u16, locals_count: u16) -> RefMut<StackFrame> {
        let mut frames_mut = self.frames.borrow_mut();
        frames_mut.push(StackFrame::new(stack_size, locals_count));
        RefMut::map(frames_mut, |x| x.last_mut().unwrap())
    }

    pub fn pop_frame(&self) {
        self.frames.borrow_mut().pop();
    }

    pub fn inspect(&self) -> Vec<StackFrame> {
        return self.frames.borrow().clone();
    }

    pub fn is_empty(&self) -> bool {
        self.frames.borrow().is_empty()
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct FrameModifiers: u16 {
        const MEM_LOAD = 0x0001;
        const MEM_SAVE = 0x0002;
        const AUTO_FREE = 0x0004;
    }
}

#[derive(Default, Clone)]
pub struct StackFrame {
    pub cp_offset: ConstantPoolIdx,
    pub pc: Cell<CodePtr>,

    stack: RefCell<Vec<Value>>,
    locals: RefCell<Vec<Value>>,

    pub class_method_idxs: (ClassIdx, MethodInClassIdx),
    pub modifiers: FrameModifiers,

    instantiated: RefCell<Vec<HeapPtr>>,
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub enum Type {
    Byte,
    Short,
    Int,
    Long,
    Char,
    Float,
    Double,
    Boolean,
    Reference,
    #[default]
    Void,
}

impl Type {
    pub(crate) fn default_value(&self) -> Value {
        match self {
            Type::Byte => Value::Byte(0),
            Type::Short => Value::Short(0),
            Type::Int => Value::Int(0),
            Type::Long => Value::Long(0),
            Type::Char => Value::Char(0),
            Type::Float => Value::Float(0.0),
            Type::Double => Value::Double(0.0),
            Type::Boolean => Value::Boolean(0),
            Type::Reference => Value::Reference(0),
            Type::Void => panic!("cannot instantiate void"),
        }
    }
}

#[derive(TryInto, Clone, Debug, PartialEq)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Char(u16),
    Float(f32),
    Double(f64),
    Boolean(i32),
    ReturnType,
    // ?
    Reference(HeapPtr), //0 == null
    #[try_into(ignore)]
    ClassIndex(ClassIdx, u16),
    ArrayOf(Type, ClassIdx),
    String(String), // simplification
    Void,
}

impl StackFrame {
    pub fn new(stack_size: u16, locals_count: u16) -> Self {
        Self {
            stack: RefCell::new(Vec::with_capacity(stack_size as usize)),
            locals: RefCell::new(vec![Value::Int(0); locals_count as usize]),
            pc: Cell::new(0),
            cp_offset: 0,
            class_method_idxs: (0, 0),
            ..Default::default()
        }
    }

    pub fn pick(&self, offset: usize) -> Value {
        let stack_ref = self.stack.borrow();
        return stack_ref[stack_ref.len() - offset - 1].clone();
    }

    pub fn pick_u8(&self, code: &VM) -> u8 {
        code.code_read_u8(self.pc.get())
    }

    pub fn read_u8(&self, code: &VM) -> u8 {
        let pc = self.pc.get();
        self.pc.set(pc + 1);
        return code.code_read_u8(pc);
    }

    pub fn read_i8(&self, code: &VM) -> i8 {
        let pc = self.pc.get();
        self.pc.set(pc + 1);
        return code.code_read_u8(pc) as i8;
    }

    pub fn read_u16(&self, code: &VM) -> u16 {
        let high = self.read_u8(code) as u16;
        let low = self.read_u8(code) as u16;
        return (high << 8) | low;
    }

    pub fn read_code_offset(&self, code: &VM) -> i16 {
        let high = self.read_u8(code) as u16;
        let low = self.read_u8(code) as u16;
        return ((high << 8) | low) as i16 - 3;
    }

    pub fn jmp_relative(&self, offset: i16) {
        let pc = self.pc.get();
        self.pc.set(((pc as isize) + (offset as isize)) as usize);
    }

    pub fn set_local(&self, idx: u8, value: Value) {
        self.locals.borrow_mut()[idx as usize] = value
    }

    pub fn get_local(&self, idx: u8) -> Value {
        return self.locals.borrow()[idx as usize].clone();
    }

    pub fn push(&self, value: Value) {
        self.stack.borrow_mut().push(value)
    }

    pub fn pop(&self) -> Value {
        return self.stack.borrow_mut().pop().unwrap();
    }

    pub fn dup(&self) {
        let mut stack_mut = self.stack.borrow_mut();
        let value = (*stack_mut.last().unwrap()).clone();
        stack_mut.push(value)
    }

    pub fn dup_x1(&self) {
        let mut stack_mut = self.stack.borrow_mut();
        let value1 = stack_mut.pop().unwrap().clone();
        let value2 = stack_mut.pop().unwrap().clone();
        stack_mut.push(value1.clone());
        stack_mut.push(value2.clone());
        stack_mut.push(value1.clone());
    }

    pub fn inspect_stack(&self) -> Vec<Value> {
        self.stack.borrow().clone()
    }
    pub fn inspect_locals(&self) -> Vec<Value> {
        self.locals.borrow().clone()
    }

    pub fn on_instantiate(&self, value: HeapPtr) {
        self.instantiated.borrow_mut().push(value.clone())
    }

    pub fn get_instantiated(&self) -> Vec<HeapPtr> {
        self.instantiated.borrow().clone()
    }
}
