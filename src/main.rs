mod brainfuck;
mod wasm;

use brainfuck::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::num::Wrapping;
extern crate byteorder;



extern "C" {
    pub fn read_val(_: *mut c_char) -> u8;
}

fn read(current_output: &Vec<u8>) -> u8 {
    let current_output = String::from_utf8_lossy(&current_output.as_slice()).into_owned();
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

fn main() {
    let code = "++[-]++++++++++-";
    let chars: Vec<char> = code.chars().collect();
    let (ast, _) = get_ast(&chars);
    let ast = compact(&ast);
    wasm::to_wasm(&ast);
}