use crate::vm::classes::{
    AccessFlags, ClassIdx, ConstantPoolIdx, ConstantPoolValue, FieldIdx, Method, MethodInClassIdx,
};
use crate::vm::memory::HeapPtr;
use crate::vm::stack::{FrameModifiers, StackFrame, Type, Value};
use crate::vm::vm::VM;
use log::{debug, info, trace};
use std::cell::RefMut;
use std::convert::TryInto;

impl VM {
    pub fn start(&mut self, class_name: &str) {
        //lookup for main method
        {
            let class_idx = self.get_or_load_class_idx(&class_name.to_string());
            let class = &self.program.borrow().classes[class_idx];
            let method_idx = class.methods.iter().position(|m| m.name == "main").unwrap();
            let main_method = &class.methods[method_idx];
            let mut frame = self
                .stack
                .push_frame(main_method.max_stack, main_method.max_locals);
            frame.pc.set(main_method.code_ptr);
            frame.cp_offset = class.constant_pool_idx;
            frame.class_method_idxs = (class_idx, method_idx);
        }

        self.do_loop();
    }

    fn call(&self, class_idx: ClassIdx, method_idx: MethodInClassIdx) {
        let class_name: String;
        let method_flags: AccessFlags;
        let method_name: String;
        let method: Method;

        {
            let class = self.get_class(class_idx);
            class_name = class.name.clone();
            method = class.methods[method_idx].clone();
            method_name = method.name.clone();
            method_flags = method.flags;
        }

        let mut args = Vec::new();
        {
            let frame = self.stack.top_frame();
            for _i in 0..method.signature.arguments.len() {
                args.insert(0, frame.pop());
            }
            if !method_flags.contains(AccessFlags::STATIC) {
                args.insert(0, frame.pop()); //object itself
            }
        }

        let is_tail_rec_optimization_requested = method_flags.contains(AccessFlags::TAIL_RECURSION);

        if is_tail_rec_optimization_requested {
            let mut frame = self.stack.top_frame_mut();

            let not_native = !method_flags.contains(AccessFlags::NATIVE);
            let same_method = frame.class_method_idxs == (class_idx, method_idx);
            let last_in_current_method = is_return(frame.pick_u8(self));
            let is_tail_rec_optimization_available =
                not_native && same_method && last_in_current_method;

            if is_tail_rec_optimization_available {
                self.perform_call(class_idx, method_idx, &args, &mut frame);
                return;
            }
        }

        debug!("Call {}#{}({:?})", class_name, method_name, &args);
        if method_flags.contains(AccessFlags::NATIVE) {
            for nm in &self.program.borrow().native_methods {
                let value = nm.invoke(&self, &class_name, &method_name, args.clone());
                if let Some(value) = value {
                    if value != Value::Void {
                        self.stack.top_frame().push(value);
                    }
                    return;
                }
            }
            //not found
            panic!(
                "Cannot find implementation of native method {}#{}",
                class_name, method_name
            );
        } else {
            assert_ne!(
                method.code_ptr, 0,
                "Method {}#{} is abstract!",
                class_name, method_name
            );

            {
                let prev_frame_modifiers = self.stack.top_frame().modifiers;
                let mut frame = self.stack.push_frame(method.max_stack, method.max_locals);
                if method_flags.contains(AccessFlags::AUTO_FREE)
                    || prev_frame_modifiers.contains(FrameModifiers::AUTO_FREE)
                {
                    frame.modifiers.insert(FrameModifiers::AUTO_FREE)
                }
                self.perform_call(class_idx, method_idx, &args, &mut frame);
            }

            let is_mem_optimization_requested = method.flags.contains(AccessFlags::MEM);
            if is_mem_optimization_requested {
                let is_mem_optimization_available = method
                    .signature
                    .arguments
                    .iter()
                    .all(|x| *x == Type::Reference)
                    && method.signature.return_type == Type::Reference;

                if is_mem_optimization_available {
                    //add call to RVM.getAnswer.
                    let rvm_class_idx =
                        self.get_or_load_class_idx(&"io/github/rvm/RVM".to_string());
                    let mut args_count = method.signature.arguments.len();
                    if !method.flags.contains(AccessFlags::STATIC) {
                        args_count += 1;
                    }
                    let get_answer_method_idx = match args_count {
                        1 => self.get_method_idx(rvm_class_idx, "getAnswer".to_string(), "(Lio/github/rvm/MemEntry;Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap(),
                        2 => self.get_method_idx(rvm_class_idx, "getAnswer".to_string(), "(Lio/github/rvm/MemEntry;Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap(),
                        _ => panic!("Unsupported amount of arguments")
                    };
                    //check mem entry associated with method

                    {
                        self.stack
                            .top_frame_mut()
                            .modifiers
                            .insert(FrameModifiers::MEM_LOAD);
                    }
                    debug!(
                        "  > redirect to RVM.getAnswer {}#{}({:?})",
                        class_name, method_name, &args
                    );
                    let mut frame = self
                        .stack
                        .push_frame(method.max_stack, method.max_locals + 1);
                    let mut get_answer_args = args.clone();
                    get_answer_args.insert(0, Value::Reference(method.mem_entry_ptr));
                    self.perform_call(
                        rvm_class_idx,
                        get_answer_method_idx,
                        &get_answer_args,
                        &mut frame,
                    );
                }
            }
        }
    }

    fn perform_call(
        &self,
        class_idx: ClassIdx,
        method_idx: MethodInClassIdx,
        args: &Vec<Value>,
        frame: &mut RefMut<StackFrame>,
    ) {
        let class = self.get_class(class_idx);
        let method = self.get_method(class_idx, method_idx);
        frame.pc.set(method.code_ptr);
        frame.class_method_idxs = (class_idx, method_idx);
        frame.cp_offset = class.constant_pool_idx;
        for i in 0..(method.max_locals as usize) {
            if i >= args.len() {
                break;
            }
            frame.set_local(i as u8, args[i].clone());
        }
    }

    fn return_call(&self) {
        debug!("Return");
        self.stack.pop_frame();
    }

    fn return_call_with_value(&self, value: Value) {
        debug!("Return {:?}", value);
        {
            let frame = self.stack.top_frame();
            if frame.modifiers.contains(FrameModifiers::MEM_SAVE) {
                //need to save value.
                debug!("Save to mem");
                let (class_idx, method_idx) = frame.class_method_idxs;

                let mut args_count: usize;
                {
                    let method = self.get_method(class_idx, method_idx);
                    args_count = method.signature.arguments.len();
                    if !method.flags.contains(AccessFlags::STATIC) {
                        args_count += 1;
                    }
                }
                let args_ptr = self.heap.new_object_array(0, args_count as i32);
                for i in 0..args_count {
                    self.heap
                        .set_array_element(args_ptr, i, frame.get_local(i as u8));
                }

                let heap_ptr = self.heap.new_object(
                    self.get_or_load_class_idx(&"io/github/rvm/MemEntry".to_string()),
                    3,
                );
                self.heap.new_object_field(Value::Reference(args_ptr)); //arguments
                self.heap.new_object_field(value.clone()); //answer
                let mut method = self.get_method_mut(class_idx, method_idx);
                self.heap
                    .new_object_field(Value::Reference(method.mem_entry_ptr)); //next
                method.mem_entry_ptr = heap_ptr;
            }

            if frame.modifiers.contains(FrameModifiers::AUTO_FREE) {
                for ptr in frame.get_instantiated() {
                    debug!("Auto free {}", ptr);
                    self.heap.free(ptr)
                }
            }
        }

        self.stack.pop_frame();
        let mods = self.stack.top_frame().modifiers.clone();

        if mods.contains(FrameModifiers::MEM_LOAD) {
            //so we have value to return. check it is not null
            if value != Value::Reference(0) {
                //ok, return it
                debug!("Return from mem");
                self.stack.pop_frame();
            } else {
                //work as usual, but add save modifier
                self.stack
                    .top_frame_mut()
                    .modifiers
                    .insert(FrameModifiers::MEM_SAVE);
                self.stack
                    .top_frame_mut()
                    .modifiers
                    .remove(FrameModifiers::MEM_LOAD);
            }
        }

        self.stack.top_frame().push(value);
    }

    pub fn do_loop(&self) {
        loop {
            match self.do_command() {
                StackModification::Nop => {}
                StackModification::Call(class_idx, method_idx) => self.call(class_idx, method_idx),
                StackModification::Return(value) => {
                    if let Value::Void = value {
                        self.return_call()
                    } else {
                        self.return_call_with_value(value)
                    }
                    if self.stack.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    fn do_command(&self) -> StackModification {
        let frame = &self.stack.top_frame();
        let code = self;
        let cmd = frame.read_u8(code);
        trace!("Process cmd [{}] at [{}]", cmd, frame.pc.get() - 1);
        match cmd {
            //aconst_null
            1 => frame.push(Value::Reference(0)),
            //iconst_m1
            2 => frame.push(Value::Int(-1)),
            //iconst_0
            3 => frame.push(Value::Int(0)),
            //iconst_1
            4 => frame.push(Value::Int(1)),
            //iconst_2
            5 => frame.push(Value::Int(2)),
            //iconst_3
            6 => frame.push(Value::Int(3)),
            //iconst_4
            7 => frame.push(Value::Int(4)),
            //iconst_5
            8 => frame.push(Value::Int(5)),
            //bipush
            16 => {
                let byte_value = frame.read_u8(code);
                frame.push(Value::Int(byte_value as i32))
            }
            //ldc
            18 => {
                let idx = frame.read_u8(code) as usize;
                let cp_entry = &self.program.borrow().constant_pool[frame.cp_offset + idx];
                match cp_entry {
                    ConstantPoolValue::String(value) => frame.push(value.clone()),
                    ConstantPoolValue::Const(value) => frame.push(value.clone()),
                    _ => panic!("ldc {:?} not supported yet", cp_entry),
                }
            }
            //iload
            21 => {
                let idx = frame.read_u8(code);
                frame.push(frame.get_local(idx).clone())
            }
            //iload_0
            26 => frame.push(frame.get_local(0).clone()),
            //iload_1
            27 => frame.push(frame.get_local(1).clone()),
            //iload_2
            28 => frame.push(frame.get_local(2).clone()),
            //iload_3
            29 => frame.push(frame.get_local(3).clone()),

            //aload_0
            42 => frame.push(frame.get_local(0).clone()),
            //aload_1
            43 => frame.push(frame.get_local(1).clone()),
            //aload_2
            44 => frame.push(frame.get_local(2).clone()),
            //aload_3
            45 => frame.push(frame.get_local(3).clone()),

            //aaload
            50 => {
                let idx: i32 = frame.pop().try_into().unwrap();
                let obj_ref: HeapPtr = frame.pop().try_into().unwrap();
                let value = self.heap.get_array_element(obj_ref, idx as usize);
                frame.push(value);
            }
            //istore
            54 => {
                let idx = frame.read_u8(code);
                let value = frame.pop();
                frame.set_local(idx, value);
            }
            //astore
            58 => {
                let idx = frame.read_u8(code);
                let obj_ref = frame.pop();
                frame.set_local(idx, obj_ref);
            }
            //istore_0
            59 => {
                let value = frame.pop();
                frame.set_local(0, value)
            }
            //istore_1
            60 => {
                let value = frame.pop();
                frame.set_local(1, value)
            }
            //istore_2
            61 => {
                let value = frame.pop();
                frame.set_local(2, value)
            }
            //istore_3
            62 => {
                let value = frame.pop();
                frame.set_local(3, value)
            }

            //astore_0
            75 => {
                let value = frame.pop();
                frame.set_local(0, value)
            }
            //astore_1
            76 => {
                let value = frame.pop();
                frame.set_local(1, value)
            }
            //astore_2
            77 => {
                let value = frame.pop();
                frame.set_local(2, value)
            }
            //astore_3
            78 => {
                let value = frame.pop();
                frame.set_local(3, value)
            }

            //aastore
            83 => {
                let value = frame.pop();
                let index: i32 = frame.pop().try_into().unwrap();
                let array_ref: HeapPtr = frame.pop().try_into().unwrap();
                self.heap
                    .set_array_element(array_ref, index as usize, value);
            }

            //pop
            87 => {
                frame.pop();
            }

            // dup
            89 => {
                frame.dup();
            }
            // dup_x1
            90 => {
                frame.dup_x1();
            }

            //iadd
            96 => {
                let i2: i32 = frame.pop().try_into().unwrap();
                let i1: i32 = frame.pop().try_into().unwrap();
                frame.push(Value::Int(i1 + i2));
            }
            //isub
            100 => {
                let i2: i32 = frame.pop().try_into().unwrap();
                let i1: i32 = frame.pop().try_into().unwrap();
                frame.push(Value::Int(i1 - i2));
            }

            //iinc
            132 => {
                let var_idx = frame.read_u8(code);
                let delta = frame.read_i8(code) as i32;
                let current_value: i32 = (frame.get_local(var_idx).clone()).try_into().unwrap();
                frame.set_local(var_idx, Value::Int(current_value + delta))
            }

            //ifeq
            153 => {
                let (then_offset, i1, i2) = Self::if_z_prepare(frame, code);
                if i1 == i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //ifne
            154 => {
                let (then_offset, i1, i2) = Self::if_z_prepare(frame, code);
                if i1 != i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //iflt
            155 => {
                let (then_offset, i1, i2) = Self::if_z_prepare(frame, code);
                if i1 < i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //ifge
            156 => {
                let (then_offset, i1, i2) = Self::if_z_prepare(frame, code);
                if i1 >= i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //ifgt
            157 => {
                let (then_offset, i1, i2) = Self::if_z_prepare(frame, code);
                if i1 > i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //ifle
            158 => {
                let (then_offset, i1, i2) = Self::if_z_prepare(frame, code);
                if i1 <= i2 {
                    frame.jmp_relative(then_offset)
                }
            }

            //if_icmpeq
            159 => {
                let (then_offset, i1, i2) = Self::iif_prepare(frame, code);
                if i1 == i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //if_icmpne
            160 => {
                let (then_offset, i1, i2) = Self::iif_prepare(frame, code);
                if i1 != i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //if_icmplt
            161 => {
                let (then_offset, i1, i2) = Self::iif_prepare(frame, code);
                if i1 < i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //if_icmpge
            162 => {
                let (then_offset, i1, i2) = Self::iif_prepare(frame, code);
                if i1 >= i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //if_icmpgt
            163 => {
                let (then_offset, i1, i2) = Self::iif_prepare(frame, code);
                if i1 > i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //if_icmple
            164 => {
                let (then_offset, i1, i2) = Self::iif_prepare(frame, code);
                if i1 <= i2 {
                    frame.jmp_relative(then_offset)
                }
            }
            //goto
            167 => {
                let offset = frame.read_code_offset(code);
                frame.jmp_relative(offset);
            }

            //ireturn
            172 => {
                let ret_value = frame.pop();
                return StackModification::Return(ret_value);
            }
            //freturn
            174 => {
                let ret_value = frame.pop();
                return StackModification::Return(ret_value);
            }
            //areturn
            176 => {
                let ret_value = frame.pop();
                return StackModification::Return(ret_value);
            }
            // return
            177 => {
                return StackModification::Return(Value::Void);
            }
            //invokevirtual
            182 => {
                let cpi = frame.cp_offset + frame.read_u16(code) as usize;
                let (mut class_idx, mut method_idx) = self.resolve_method_reference(cpi);
                //get reference
                let class = &self.program.borrow().classes[class_idx];
                let method = &class.methods[method_idx];
                let args_count = method.signature.arguments.len();
                if let Value::Reference(heap_ptr) = frame.pick(args_count) {
                    let value = self.heap.get_value(heap_ptr);
                    if let Value::ClassIndex(real_class_idx, _) = value {
                        //check vmt
                        let real_class = &self.program.borrow().classes[real_class_idx];
                        (class_idx, method_idx) = *real_class
                            .vmt
                            .mapping
                            .get(&(class_idx, method_idx))
                            .unwrap_or(&(class_idx, method_idx));
                        return StackModification::Call(class_idx, method_idx);
                    }
                }
                panic!("Cannot find object in stack :(")
            }
            //invokespecial
            183 => {
                let cpi = frame.cp_offset + frame.read_u16(code) as usize;
                let (class_idx, method_idx) = self.resolve_method_reference(cpi);
                //constructors here too
                return StackModification::Call(class_idx, method_idx);
            }

            //invokestatic
            184 => {
                let cpi = frame.cp_offset + frame.read_u16(code) as usize;
                let (class_idx, method_idx) = self.resolve_method_reference(cpi);
                return StackModification::Call(class_idx, method_idx);
            }
            // getfield
            180 => {
                let cpi = frame.cp_offset + frame.read_u16(code) as usize;
                let (_, field_idx) = self.resolve_field_reference(cpi);
                let object_ref_ptr: HeapPtr = frame.pop().try_into().unwrap();
                let value = self.heap.get_field(object_ref_ptr, field_idx);
                frame.push(value);
            }
            // putfield
            181 => {
                let cpi = frame.cp_offset + frame.read_u16(code) as usize;
                let (_, field_idx) = self.resolve_field_reference(cpi);
                let value: Value = frame.pop();
                let object_ref_ptr: HeapPtr = frame.pop().try_into().unwrap();
                self.heap.set_field(object_ref_ptr, field_idx, value);
            }
            //new
            187 => {
                let cpi = frame.cp_offset + frame.read_u16(code) as usize;

                let class_idx = self.resolve_class_reference(cpi);
                let ptr = self.new_object(class_idx);
                frame.push(Value::Reference(ptr));
                if frame.modifiers.contains(FrameModifiers::AUTO_FREE) {
                    frame.on_instantiate(ptr);
                }
            }
            //anewarray
            189 => {
                let cpi = frame.cp_offset + frame.read_u16(code) as usize;
                let class_idx = self.resolve_class_reference(cpi);

                let count: i32 = frame.pop().try_into().unwrap();
                let ptr = self.new_object_array(class_idx, count);
                frame.push(Value::Reference(ptr));
                if frame.modifiers.contains(FrameModifiers::AUTO_FREE) {
                    frame.on_instantiate(ptr);
                }
            }
            //arraylength
            190 => {
                let ptr: HeapPtr = frame.pop().try_into().unwrap();
                frame.push(self.heap.get_field(ptr, 0)); //length is 0th field
            }
            //ifnull
            198 => {
                let ptr: HeapPtr = frame.pop().try_into().unwrap();
                let then_offset = frame.read_code_offset(code);
                if ptr == 0 {
                    frame.jmp_relative(then_offset)
                }
            }
            //ifnonnull
            199 => {
                let ptr: HeapPtr = frame.pop().try_into().unwrap();
                let then_offset = frame.read_code_offset(code);
                if ptr != 0 {
                    frame.jmp_relative(then_offset)
                }
            }

            _ => panic!("Unknown code: {}", cmd),
        }
        return StackModification::Nop;
    }

    fn iif_prepare(frame: &StackFrame, code: &VM) -> (i16, i32, i32) {
        let then_offset = frame.read_code_offset(code);
        let i2 = frame.pop().try_into().unwrap();
        let i1 = frame.pop().try_into().unwrap();
        (then_offset, i1, i2)
    }

    fn if_z_prepare(frame: &StackFrame, code: &VM) -> (i16, i32, i32) {
        let then_offset = frame.read_code_offset(code);
        let i1 = frame.pop().try_into().unwrap();
        (then_offset, i1, 0)
    }

    fn resolve_method_reference(&self, cpi: ConstantPoolIdx) -> (ClassIdx, MethodInClassIdx) {
        let value = self.get_constant_pool_value(cpi);
        match value {
            ConstantPoolValue::UnresolvedMethodRef {
                class_name,
                method_name,
                signature,
            } => {
                let class_idx = self.get_or_load_class_idx(&class_name);
                let method_idx = self
                    .get_method_idx(class_idx, method_name.clone(), signature.clone())
                    .expect(&format!(
                        "Cannot find method {} {} {}",
                        class_name, method_name, signature
                    ));
                self.set_constant_pool_value(
                    cpi,
                    ConstantPoolValue::MethodRef(class_idx, method_idx),
                );
                return (class_idx, method_idx);
            }
            ConstantPoolValue::MethodRef(class_idx, method_idx) => {
                return (class_idx, method_idx);
            }
            _ => panic!("Unexpected cp entry {:?}", value),
        }
    }

    fn resolve_field_reference(&self, cpi: ConstantPoolIdx) -> (ClassIdx, FieldIdx) {
        let value = self.get_constant_pool_value(cpi);
        match value {
            ConstantPoolValue::UnresolvedFieldRef {
                class_name,
                field_name,
            } => {
                let class_idx = self.get_or_load_class_idx(&class_name);
                let field_idx = self
                    .get_field_idx(class_idx, field_name.clone())
                    .expect(&format!("Cannot find field {} {}", class_name, field_name));
                self.set_constant_pool_value(
                    cpi,
                    ConstantPoolValue::FieldRef(class_idx, field_idx),
                );
                return (class_idx, field_idx);
            }
            ConstantPoolValue::FieldRef(class_idx, field_idx) => {
                return (class_idx, field_idx);
            }
            _ => panic!("Unexpected cp entry {:?}", value),
        }
    }

    fn resolve_class_reference(&self, cpi: ConstantPoolIdx) -> ClassIdx {
        let value = self.get_constant_pool_value(cpi);
        match value {
            ConstantPoolValue::UnresolvedClassRef { class_name } => {
                let class_idx = self.get_or_load_class_idx(&class_name);
                self.set_constant_pool_value(cpi, ConstantPoolValue::Class(class_idx));
                return class_idx;
            }
            ConstantPoolValue::Class(class_idx) => {
                return class_idx;
            }
            _ => panic!("Unexpected cp entry {:?}", value),
        }
    }

    fn get_method_idx(
        &self,
        class_idx: ClassIdx,
        method_name: String,
        method_signature: String,
    ) -> Option<MethodInClassIdx> {
        let program_ref = self.program.borrow();
        program_ref
            .method_names_to_idxs
            .get(&(class_idx, method_name, method_signature))
            .map(|x| *x)
    }

    fn get_field_idx(&self, class_idx: ClassIdx, field_name: String) -> Option<FieldIdx> {
        let program_ref = self.program.borrow();
        program_ref
            .field_names_to_idxs
            .get(&(class_idx, field_name))
            .map(|x| *x)
    }
}

fn is_return(op_code: u8) -> bool {
    (172..=177).contains(&op_code)
}

enum StackModification {
    Nop,
    Call(ClassIdx, MethodInClassIdx),
    Return(Value),
}
