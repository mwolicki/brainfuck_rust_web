use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

enum Op {
    IncPointer,
    DecPointer,
    IncVal,
    DecVal,
    Print,
    Read,
    While { ops: Vec<Op> },
}

struct State {
    curr_ptr: usize,
    data: [u8; 4092],
    output: String,
}

extern "C" {
    pub fn read_val() -> u8;
}

fn eval_while(state: &mut State, ops: &[Op]) {
    while state.data[state.curr_ptr] != 0 {
        eval_vec(state, ops);
    }
}

fn eval_vec(state: &mut State, ops: &[Op]) {
    for op in ops.iter() {
        eval(state, op);
    }
}

fn eval(state: &mut State, op: &Op) {
    match *op {
        Op::IncPointer => state.curr_ptr += 1,
        Op::DecPointer => state.curr_ptr -= 1,
        Op::While { ref ops } => {
            eval_while(state, ops);
        }
        Op::IncVal => state.data[state.curr_ptr] += 1,
        Op::DecVal => state.data[state.curr_ptr] -= 1,
        Op::Print => state.output.push(char::from(state.data[state.curr_ptr])),
        Op::Read => state.data[state.curr_ptr] = unsafe { read_val() },
    }
}

fn get_ast(code: &str) -> (Vec<Op>, usize) {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < code.len() {
        let ch = code[(i..i + 1)].as_ref();
        let op = match ch {
            ">" => Some(Op::IncPointer),
            "<" => Some(Op::DecPointer),
            "+" => Some(Op::IncVal),
            "-" => Some(Op::DecVal),
            "." => Some(Op::Print),
            "," => Some(Op::Read),
            "[" => {
                let (ops, size) = get_ast(code[(i + 1..code.len())].as_ref());
                i += size + 1;
                match code[(i..i + 1)].as_ref() {
                    "]" => Some(Op::While { ops: ops }),
                    x => panic!("while loop needs to end with ']' but was with '{:?}'", x),
                }
            }
            "]" => return (ops, i), 
            _ => None,
        };
        if let Some(op) = op {
            ops.push(op);
        }
        i += 1;
    }
    (ops, i)
}

pub fn run_code(code: &str) -> String {

    let mut state = State {
        curr_ptr: 0,
        data: [0; 4092],
        output: String::new(),
    };
    let (ast, _) = get_ast(code);
    eval_vec(&mut state, &ast);
    state.output
}

fn my_string_safe(i: *mut c_char) -> String {
    unsafe { CStr::from_ptr(i).to_string_lossy().into_owned() }
}

#[no_mangle]
pub fn js_run_code(code: *mut c_char) -> *mut c_char {
    let s = my_string_safe(code);
    let output = run_code(s.as_str());
    CString::new(output.as_str()).unwrap().into_raw()
}

fn main() {}