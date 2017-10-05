use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::num::Wrapping;

extern "C" {
    pub fn read_val(_: *mut c_char) -> u8;
}

fn read(current_output: &Vec<u8>) -> u8 {
    let current_output = String::from_utf8_lossy(&current_output.as_slice()).into_owned();
    unsafe { read_val(to_c_str(&current_output)) }
}


#[derive(PartialEq, Clone)]
enum Op {
    IncPointer(usize),
    DecPointer(usize),
    IncVal(u8),
    DecVal(u8),
    Print,
    Read,
    While { ops: Vec<Op> },
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

        Op::Print => state.output.push(state.data[state.curr_ptr]),
        Op::Read => state.data[state.curr_ptr] = read(&state.output),
    }
}

fn compact(ast: &[Op]) -> Vec<Op> {
    let mut compacted_ast = Vec::new();
    let mut current_op: Option<Op> = None;
    let mut count = 0;

    for op in ast {
        if let Some(curr_op) = current_op.clone() {
            if *op == curr_op {
                count += 1;
            } else {
                match curr_op {
                    Op::IncPointer(n) => compacted_ast.push(Op::IncPointer(n + count)),
                    Op::DecPointer(n) => compacted_ast.push(Op::DecPointer(n + count)),
                    Op::IncVal(n) => compacted_ast.push(Op::IncVal(n + count as u8)),
                    Op::DecVal(n) => compacted_ast.push(Op::DecVal(n + count as u8)),
                    _ => (),
                }
                current_op = None;
                count = 0;
            }
        }
        match *op {
            Op::While { ref ops } => compacted_ast.push(Op::While { ops: compact(ops) }),
            Op::Print => compacted_ast.push(Op::Print),
            Op::Read => compacted_ast.push(Op::Read),
            _ => current_op = Some(op.clone()),
        }
    }

    if let Some(curr_op) = current_op.clone() {
        match curr_op {
            Op::IncPointer(n) => compacted_ast.push(Op::IncPointer(n + count)),
            Op::DecPointer(n) => compacted_ast.push(Op::DecPointer(n + count)),
            Op::IncVal(n) => compacted_ast.push(Op::IncVal(n + count as u8)),
            Op::DecVal(n) => compacted_ast.push(Op::DecVal(n + count as u8)),
            _ => (),
        }
    }

    compacted_ast
}

fn get_ast(code: &[char]) -> (Vec<Op>, usize) {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < code.len() {
        let ch = code[i];
        let op = match ch {
            '>' => Some(Op::IncPointer(1)),
            '<' => Some(Op::DecPointer(1)),
            '+' => Some(Op::IncVal(1)),
            '-' => Some(Op::DecVal(1)),
            '.' => Some(Op::Print),
            ',' => Some(Op::Read),
            '[' => {
                let (ops, size) = get_ast(&code[(i + 1..code.len())]);
                i += size + 1;
                match code[i] {
                    ']' => Some(Op::While { ops: ops }),
                    x => panic!("while loop needs to end with ']' but was with '{:?}'", x),
                }
            }
            ']' => return (ops, i),  
            _ => None,
        };
        if let Some(op) = op {
            ops.push(op);
        }
        i += 1;
    }
    (ops, i)
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

fn main() {}