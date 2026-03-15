mod brainfuck;
mod wasm;
mod leb128;

use crate::brainfuck::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char};
use std::num::Wrapping;
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

#[cfg(not(test))]
extern "C" {
    pub fn read_val(_: *mut c_char) -> u8;
}

#[cfg(not(test))]
fn read(current_output: &[u8]) -> u8 {
    let current_output = String::from_utf8_lossy(current_output).into_owned();
    unsafe { read_val(to_c_str(&current_output)) }
}

#[cfg(test)]
fn read(_current_output: &[u8]) -> u8 {
    0
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use super::run_brainfuck;

    proptest! {
        #[test]
        fn empty_program_produces_no_output(_ignored in Just(())) {
            prop_assert_eq!(run_brainfuck(""), "");
        }

        #[test]
        fn non_bf_chars_produce_no_output(s in "[^><+\\-.,\\[\\]]*") {
            prop_assert_eq!(run_brainfuck(&s), "");
        }

        #[test]
        fn n_increments_then_print_outputs_that_byte(n in 0u8..128u8) {
            // Values 0-127 are valid single-byte UTF-8, so no replacement occurs
            let code = format!("{}.", "+".repeat(n as usize));
            let result = run_brainfuck(&code);
            prop_assert_eq!(result.len(), 1);
            prop_assert_eq!(result.as_bytes()[0], n);
        }

        #[test]
        fn balanced_pointer_ops_return_to_origin(n in 1usize..4092usize) {
            // n rights then n lefts lands back at cell 0; increment and print it
            let code = format!("{}{}+.", ">".repeat(n), "<".repeat(n));
            let result = run_brainfuck(&code);
            prop_assert_eq!(result.as_bytes(), &[1u8]);
        }

        #[test]
        fn repeated_print_outputs_same_byte_multiple_times(
            val in 1u8..128u8,
            times in 1usize..=10usize
        ) {
            let code = format!("{}{}", "+".repeat(val as usize), ".".repeat(times));
            let result = run_brainfuck(&code);
            prop_assert_eq!(result.len(), times);
            prop_assert!(result.as_bytes().iter().all(|&b| b == val));
        }

        #[test]
        fn inc_and_dec_cancel_out(n in 0u8..128u8) {
            // n increments followed by n decrements leaves cell at 0, print = '\0'
            let code = format!("{}{}.", "+".repeat(n as usize), "-".repeat(n as usize));
            let result = run_brainfuck(&code);
            prop_assert_eq!(result.as_bytes(), &[0u8]);
        }

        #[test]
        fn set_register_to_zero_loop_clears_cell(val in 1u8..128u8) {
            // Set cell to val, then [-] zeroes it, then print
            let code = format!("{}[-].", "+".repeat(val as usize));
            let result = run_brainfuck(&code);
            prop_assert_eq!(result.as_bytes(), &[0u8]);
        }

        #[test]
        fn output_grows_with_print_count(
            val in 1u8..128u8,
            times in 1usize..=10usize
        ) {
            let code = format!("{}{}", "+".repeat(val as usize), ".".repeat(times));
            prop_assert_eq!(run_brainfuck(&code).len(), times);
        }
    }
}