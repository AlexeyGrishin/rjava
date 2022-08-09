use crate::vm::classes::{
    AccessFlags, Class, ClassIdx, ConstantPoolValue, Field, Method, MethodInClassIdx, Signature,
};
use crate::vm::program::Program;
use crate::vm::stack::{Type, Value};
use crate::vm::vm::VM;
use class_file::attr::{Code, RuntimeVisibleAnnotations};
use class_file::{CPEntry, ClassFile};
use log::trace;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;

#[derive(Clone)]
pub struct ClassLoader {
    base_path: String,
}

impl Default for ClassLoader {
    fn default() -> Self {
        ClassLoader::new(".")
    }
}

//todo: mark as internal
impl ClassLoader {
    pub fn new(path: &str) -> Self {
        return Self {
            base_path: path.to_string().clone(),
        };
    }

    pub fn load_class_into(&self, name: &String, vm: &VM, program: &mut Program) -> ClassIdx {
        if let Some(idx) = program.class_names_to_idxs.get(name) {
            return *idx;
        }
        let mut data = Vec::new();
        let path = Path::new(&self.base_path).join(name.to_owned() + ".class");
        File::open(&path)
            .expect(&format!("File not found: {:?}", path))
            .read_to_end(&mut data)
            .unwrap();
        //todo: NoClassDefFound
        let class_file = ClassFile::parse(&data).unwrap().1;

        //1. put constant pool to vm
        let class_idx: ClassIdx = program.classes.len();
        let pool = &class_file.constant_pool;
        let cpidx = program.constant_pool.len();
        for entry in &class_file.constant_pool.entries {
            let cpv: ConstantPoolValue = match entry {
                CPEntry::Class(ci) => ConstantPoolValue::UnresolvedClassRef {
                    class_name: pool
                        .index(ci.name_index)
                        .unwrap()
                        .data
                        .to_utf8()
                        .to_string(),
                },

                CPEntry::FieldRef(fi) => ConstantPoolValue::UnresolvedFieldRef {
                    class_name: (pool
                        .index(pool.index(fi.class_index).unwrap().name_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                    field_name: (pool
                        .index(pool.index(fi.name_and_type_index).unwrap().name_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                },
                CPEntry::MethodRef(mi) => ConstantPoolValue::UnresolvedMethodRef {
                    class_name: (pool
                        .index(pool.index(mi.class_index).unwrap().name_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                    method_name: (pool
                        .index(pool.index(mi.name_and_type_index).unwrap().name_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                    signature: (pool
                        .index(pool.index(mi.name_and_type_index).unwrap().descriptor_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                },
                CPEntry::InterfaceMethodRef(mi) => ConstantPoolValue::UnresolvedMethodRef {
                    class_name: (*pool
                        .index(pool.index(mi.class_index).unwrap().name_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                    method_name: (*pool
                        .index(pool.index(mi.name_and_type_index).unwrap().name_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                    signature: (*pool
                        .index(pool.index(mi.name_and_type_index).unwrap().descriptor_index)
                        .unwrap()
                        .data)
                        .to_utf8()
                        .to_string(),
                },
                CPEntry::String(si) => {
                    let string_value = pool
                        .index(si.string_index)
                        .unwrap()
                        .data
                        .to_utf8()
                        .to_string();
                    let ptr = vm.new_string(&string_value);
                    ConstantPoolValue::String(Value::Reference(ptr))
                }
                CPEntry::Integer(ii) => ConstantPoolValue::Const(Value::Int(ii.bytes as i32)),
                CPEntry::Float(fi) => {
                    ConstantPoolValue::Const(Value::Float(f32::from_bits(fi.bytes)))
                }
                CPEntry::Long(_) => ConstantPoolValue::Unsupported,
                CPEntry::Double(_) => ConstantPoolValue::Unsupported,
                _ => ConstantPoolValue::Skip,
            };
            program.constant_pool.push(cpv);
        }

        let super_class_name = pool
            .index(pool.index(class_file.super_class).unwrap().name_index)
            .unwrap()
            .data
            .to_utf8()
            .to_string();
        let super_class_idx = self.load_class_into(&super_class_name, vm, program);

        let mut class = Class {
            name: name.clone(),
            super_class_idx: super_class_idx,
            vmt: Default::default(),
            constant_pool_idx: cpidx - 1, //because start with [1]
            fields: vec![],
            methods: vec![],
        };

        let super_class = &program.classes[class.super_class_idx];

        //2. read fields
        for field in &super_class.fields {
            let findex = class.fields.len();
            program
                .field_names_to_idxs
                .insert((class_idx, field.name.clone()), findex);
            class.fields.push((*field).clone())
        }

        for field in &class_file.fields {
            let name = pool
                .index(field.name_index)
                .unwrap()
                .data
                .to_utf8()
                .to_string();
            let our_type = parse_type(
                pool.index(field.descriptor_index)
                    .unwrap()
                    .data
                    .to_utf8()
                    .deref(),
            );
            let findex = class.fields.len();
            program
                .field_names_to_idxs
                .insert((class_idx, name.clone()), findex);

            class.fields.push(Field {
                name: name.clone(),
                flags: AccessFlags::empty(),
                value_type: our_type,
            })
        }
        //3. read methods, put code to vm

        //todo: interfaces

        class.vmt = super_class.vmt.clone();

        for method_info in class_file.methods {
            let signature = pool
                .index(method_info.descriptor_index)
                .unwrap()
                .data
                .to_utf8()
                .to_string();
            let mut method = Method {
                name: pool
                    .index(method_info.name_index)
                    .unwrap()
                    .data
                    .to_utf8()
                    .to_string(),
                signature: parse_signature(&signature),
                flags: AccessFlags::from_bits(method_info.access_flags).unwrap(),
                code_ptr: 0,
                max_stack: 0,
                max_locals: 0,
                annotation_names: vec![],
                mem_entry_ptr: 0,
            };
            if let Some(code) = method_info.attributes.get::<Code>(pool) {
                method.code_ptr = program.code.len();
                program.code.extend_from_slice(code.code);
                method.max_locals = code.max_locals;
                method.max_stack = code.max_stack;
                trace!("{} {}: {:?}", class.name, method.name, code.code)
            }
            if let Some(annotations) = method_info
                .attributes
                .get::<RuntimeVisibleAnnotations>(pool)
            {
                for ann in &annotations.data {
                    let annotation_name = pool
                        .index(ann.type_index)
                        .unwrap()
                        .data
                        .to_utf8()
                        .to_string();
                    match annotation_name.as_str() {
                        "Lio/github/rvm/RVM$TailRecursion;" => {
                            method.flags.insert(AccessFlags::TAIL_RECURSION)
                        }
                        "Lio/github/rvm/RVM$AutoFree;" => {
                            method.flags.insert(AccessFlags::AUTO_FREE)
                        }
                        "Lio/github/rvm/RVM$Mem;" => {
                            method.flags.insert(AccessFlags::MEM);
                            method.mem_entry_ptr = 0
                        }
                        _ => {}
                    }
                    method.annotation_names.push(annotation_name);
                }
            }
            let midx = class.methods.len();
            let method_name = method.name.clone();
            let method_signature = method.signature.clone();
            class.methods.push(method);

            //4. compose VMT
            if let Some((ci, mi)) = find_method(
                program,
                class.super_class_idx,
                &method_name,
                &method_signature,
            ) {
                class.vmt.mapping.insert((ci, mi), (class_idx, midx));
            }

            program
                .method_names_to_idxs
                .insert((class_idx, method_name, signature.clone()), midx);
        }

        //5. create Class, put to vm
        program.classes.push(class);
        program.class_names_to_idxs.insert(name.clone(), class_idx);
        return class_idx;
    }
}

fn find_method(
    program: &mut Program,
    class_idx: ClassIdx,
    name: &str,
    signature: &Signature,
) -> Option<(ClassIdx, MethodInClassIdx)> {
    if class_idx == 0 {
        return None;
    }

    let class = &program.classes[class_idx];
    let mut i = 0;
    for m in &class.methods {
        if m.signature == *signature && m.name == name {
            return Some((class_idx, i));
        }
        i += 1;
    }
    return find_method(program, class.super_class_idx, name, signature);
}

fn parse_type(ftype: &str) -> Type {
    match ftype {
        "B" => Type::Byte,
        "C" => Type::Char,
        "D" => Type::Double,
        "F" => Type::Float,
        "I" => Type::Int,
        "J" => Type::Long,
        "S" => Type::Short,
        "Z" => Type::Boolean,
        "V" => Type::Void,
        ar if ar.starts_with("[") => Type::Reference, //todo: temp
        rf if rf.starts_with("L") => Type::Reference,
        _ => panic!("'{}' not supported yet", ftype),
    }
}

fn parse_signature(signature: &String) -> Signature {
    trace!("{}", signature);
    let mut sign = Signature {
        return_type: Type::Void,
        arguments: vec![],
    };
    let mut i = 1; //skip '('
    let mut is_return = false;
    while i < signature.len() {
        let mut c: &str = &signature[i..=i];
        if c == ")" {
            is_return = true;
            i += 1;
            continue;
        }
        if c == "[" {
            //todo: temp skip
            i += 1;
            continue;
        }
        if c == "L" || c == "[" {
            //
            let i2 = signature[i..].find(";").unwrap() + i;
            c = &signature[i..=i2];
            i = i2 + 1;
        } else {
            i += 1;
        }

        if is_return {
            sign.return_type = parse_type(c);
        } else {
            sign.arguments.push(parse_type(c));
        }
    }
    sign
}
