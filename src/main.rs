#![feature(rustc_private)]
use std::process::Command;

extern crate libc;

mod codegen;

use crate::codegen::Codegen;

#[derive(Debug)]
enum TokenType {
    Next(usize),
    Prev(usize),
    Inc(usize),
    Dec(usize),
    Print,
    Scan,
    Jumpf(usize),
    Jumpb(usize),
}

fn main() {
    let mut memory = [0u8; 512];
    let args: String = std::env::args().skip(1).take(1).collect();
    let data = std::fs::read_to_string(args).expect("USAGE: ./bf <program.bf>");
    let mut tokens = vec![];
    let mut stack = vec![];

    let mut ptr = 0;
    let mut instruction_index = 0;

    while ptr < data.len() {
        let ch = data.as_bytes()[ptr];
        match ch {
            b'+' => {
                let mut count = 1;
                while ch == data.as_bytes()[ptr + count] {
                    count += 1;
                }
                ptr += count - 1;
                tokens.push(TokenType::Inc(count));
            }
            b'-' => {
                let mut count = 1;
                while ch == data.as_bytes()[ptr + count] {
                    count += 1;
                }
                ptr += count - 1;
                tokens.push(TokenType::Dec(count));
            }
            b'>' => {
                let mut count = 1;
                while ch == data.as_bytes()[ptr + count] {
                    count += 1;
                }
                ptr += count - 1;
                tokens.push(TokenType::Next(count));
            }
            b'<' => {
                let mut count = 1;
                while ch == data.as_bytes()[ptr + count] {
                    count += 1;
                }
                ptr += count - 1;
                tokens.push(TokenType::Prev(count));
            }
            b'.' => {
                tokens.push(TokenType::Print);
            }
            b',' => {
                tokens.push(TokenType::Scan);
            }
            b'[' => {
                stack.push(instruction_index);
                tokens.push(TokenType::Jumpf(instruction_index));
                instruction_index += 1;
            }
            b']' => {
                let jump_index = stack.pop().unwrap();
                tokens.push(TokenType::Jumpb(jump_index));
            }
            _ => {}
        }
        ptr += 1;
    }

    // Codegen
    let mut codegen = Codegen::new();

    codegen.setup();

    for token in tokens {
        match token {
            TokenType::Next(offset) => codegen.next(offset),
            TokenType::Prev(offset) => codegen.prev(offset),
            TokenType::Inc(offset) => codegen.inc(offset),
            TokenType::Dec(offset) => codegen.dec(offset),
            TokenType::Jumpf(offset) => codegen.jumpf(offset),
            TokenType::Jumpb(offset) => codegen.jumpb(offset),
            TokenType::Print => codegen.print(),
            TokenType::Scan => todo!(),
        }
    }

    codegen.cleanup();

    std::fs::write("./test.s", codegen.code).unwrap();

    let res = Command::new("nasm")
        .args(&["./test.s", "-f", "bin", "-o", "test"])
        .status()
        .expect("Failed to run nasm.");

    assert!(res.success(), "Failed to generate assembly");

    let bytecode = std::fs::read("./test").expect("Failed to read file");

    let jit_mem = Codegen::alloc_rwx(16 * 1024 * 1024);

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
