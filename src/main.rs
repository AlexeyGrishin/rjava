extern crate class_file;
extern crate core;

pub mod vm;

use crate::vm::vm::VM;
use simplelog::*;
use std::fs::File;

use log::LevelFilter;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let _ = WriteLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .build(),
        File::create("rjava.log").unwrap(),
    );

    let mut vm = VM::new(".");
    vm.start(&args[1]);
}
