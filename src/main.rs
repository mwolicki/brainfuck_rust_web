mod brainfuck;
mod wasm;
mod leb128;

use brainfuck::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char};
use std::num::Wrapping;
extern crate byteorder;


use std::mem;


//copred from https://gist.github.com/thomas-jeepe/ff938fe2eff616f7bbe4bd3dca91a550
#[repr(C)]
#[derive(Debug)]
pub struct JsBytes {
    ptr: u32,
    len: u32,
    cap: u32,
}

impl JsBytes {
    pub fn new(mut bytes: Vec<u8>) -> *mut JsBytes {
        let ptr = bytes.as_mut_ptr() as u32;
        let len = bytes.len() as u32;
        let cap = bytes.capacity() as u32;
        mem::forget(bytes);
        let boxed = Box::new(JsBytes { ptr, len, cap });
        Box::into_raw(boxed)
    }
}


#[no_mangle]
pub unsafe fn drop_bytes(ptr: *mut JsBytes) {
    let boxed: Box<JsBytes> = Box::from_raw(ptr);
    Vec::from_raw_parts(boxed.ptr as *mut u8, boxed.len as usize, boxed.cap as usize);
}

extern "C" {
    pub fn read_val(_: *mut c_char) -> u8;
}

fn read(current_output: &[u8]) -> u8 {
    let current_output = String::from_utf8_lossy(current_output).into_owned();
    unsafe { read_val(to_c_str(&current_output)) }
}



const HEAP_SIZE: usize = 4092;

struct State {
    curr_ptr: usize,
    data: [u8; HEAP_SIZE],
    output: Vec<u8>,
}


fn eval_while(state: &mut State, ops: &[Op]) {
    while state.data[state.curr_ptr] != 0 {
        eval_vec(state, ops);
    }
}

fn eval_vec(state: &mut State, ops: &[Op]) {
    for op in ops {
        eval(state, op);
    }
}

fn eval(state: &mut State, op: &Op) {
    match *op {
        Op::IncPointer(n) => {
            state.curr_ptr = (Wrapping(state.curr_ptr) + Wrapping(n)).0 % HEAP_SIZE
        }
        Op::DecPointer(n) => {
            state.curr_ptr = (Wrapping(state.curr_ptr) - Wrapping(n)).0 % HEAP_SIZE
        }
        Op::While { ref ops } => eval_while(state, ops),
        Op::IncVal(n) => {
            state.data[state.curr_ptr] = (Wrapping(state.data[state.curr_ptr]) + Wrapping(n)).0
        }
        Op::DecVal(n) => {
            state.data[state.curr_ptr] = (Wrapping(state.data[state.curr_ptr]) - Wrapping(n)).0
        }
        Op::SetRegisterToZero => state.data[state.curr_ptr] = 0,

        Op::Print => state.output.push(state.data[state.curr_ptr]),
        Op::Read => state.data[state.curr_ptr] = read(&state.output),
    }
}

fn run_brainfuck(code: &str) -> String {
    let mut state = State {
        curr_ptr: 0,
        data: [0; HEAP_SIZE],
        output: Vec::new(),
    };

    let chars: Vec<char> = code.chars().collect();
    let (ast, _) = get_ast(&chars);
    let ast = compact(&ast);
    eval_vec(&mut state, &ast);
    String::from_utf8_lossy(state.output.as_slice()).into_owned()
}

fn from_c_str(i: *mut c_char) -> String {
    unsafe { CStr::from_ptr(i).to_string_lossy().into_owned() }
}

fn to_c_str(s: &String) -> *mut c_char {
    CString::new(s.as_str())
        .expect("Couldn't convert to string.")
        .into_raw()
}

#[no_mangle]
pub fn js_run_code(code: *mut c_char) -> *mut c_char {
    let s = from_c_str(code);
    let output = run_brainfuck(s.as_str());
    to_c_str(&output)
}


#[no_mangle]
pub fn compile_to_wasm(code: *mut c_char) -> *mut JsBytes {
    let code = from_c_str(code);
    println!("{}", code);
    let chars: Vec<char> = code.chars().collect();
    let (ast, _) = get_ast(&chars);
    let ast = compact(&ast);
    let x = wasm::to_wasm(&ast);
    JsBytes::new(x)
}

fn main() {
}