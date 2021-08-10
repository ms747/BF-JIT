#![feature(rustc_private)]
use std::process::Command;

extern crate libc;

mod codegen;

use crate::codegen::Codegen;

fn alloc_rwx(size: usize) -> &'static mut [u8] {
    extern "C" {
        fn mmap(
            addr: *mut u8,
            length: usize,
            prot: i32,
            flags: i32,
            fd: i32,
            offset: usize,
        ) -> *mut u8;
    }

    unsafe {
        let ret = mmap(0 as *mut u8, size, 7, 34, -1, 0);
        assert!(!ret.is_null());
        std::slice::from_raw_parts_mut(ret, size)
    }
}

fn main() {
    // BF memory
    let mut memory = [0u8; 512];

    let mut codegen = Codegen::new();

    codegen.setup();
    codegen.inc(14);
    codegen.jumpf(1);
    codegen.next(1);
    codegen.inc(5);
    codegen.prev(1);
    codegen.dec(1);
    codegen.jumpb(1);
    codegen.next(1);
    codegen.print();
    codegen.cleanup();

    // println!("{}", codegen.code);
    // codegen.setup();
    // codegen.inc(72);
    // codegen.print();
    // codegen.dec(3);
    // codegen.print();
    // codegen.inc(7);
    // codegen.print();
    // codegen.print();
    // codegen.inc(3);
    // codegen.print();
    // codegen.dec(35);
    // codegen.print();
    // codegen.dec(12);
    // codegen.print();
    // codegen.inc(55);
    // codegen.print();
    // codegen.dec(8);
    // codegen.print();
    // codegen.inc(3);
    // codegen.print();
    // codegen.dec(6);
    // codegen.print();
    // codegen.dec(8);
    // codegen.print();
    // codegen.dec(58);
    // codegen.print();
    // codegen.cleanup();

    std::fs::write("./test.s", codegen.code).unwrap();

    let res = Command::new("nasm")
        .args(&["./test.s", "-f", "bin", "-o", "test"])
        .status()
        .expect("Failed to run nasm.");

    assert!(res.success(), "Failed to generate assembly");

    let bytecode = std::fs::read("./test").expect("Failed to read file");

    let jit_mem = alloc_rwx(16 * 1024 * 1024);

    unsafe {
        libc::memcpy(
            jit_mem.as_mut_ptr() as *mut libc::c_void,
            bytecode.as_ptr() as *const libc::c_void,
            bytecode.len(),
        );
    }

    let jitfn: fn(memory: &mut [u8; 512]) -> () = unsafe { std::mem::transmute(jit_mem.as_ptr()) };

    jitfn(&mut memory);
}
