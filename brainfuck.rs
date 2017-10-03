use std::collections::HashMap;
use std::num::Wrapping;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Debug)]
enum Op {
    IncPointer,
    DecPointer,
    IncVal,
    DecVal,
    Print,
    Read,
    While { ops: Vec<Op> },
}

#[derive(Debug, Clone)]
struct State {
    curr_ptr: i32,
    data: HashMap<i32, u8>,
}

extern "C" {
    pub fn print_val(_: u8);
    pub fn read_val() -> u8;
}

fn eval_while(s: State, ops: &[Op]) -> State {
    let mut state = s;
    while *(state.data.entry(state.curr_ptr).or_insert(0)) != 0 {
        state = eval_vec(state, ops);
    }
    state
}

fn eval_vec(s: State, ops: &[Op]) -> State {
    let mut state = s;
    for op in ops.iter() {
        state = eval(state, op);
    }
    state
}

fn eval(mut s: State, op: &Op) -> State {
    {
        let state = s.clone();
        let val = s.data.entry(s.curr_ptr).or_insert(0);
        let wrapped_val = Wrapping(*val);
        let wrapped_one = Wrapping(1);

        match *op {
            Op::IncPointer => s.curr_ptr += 1,
            Op::DecPointer => s.curr_ptr -= 1,
            Op::IncVal => *val = (wrapped_val + wrapped_one).0,
            Op::DecVal => *val = (wrapped_val - wrapped_one).0,
            Op::Print => unsafe { print_val(*val) },
            Op::Read => *val = unsafe { read_val() },
            Op::While { ref ops } => {
                return eval_while(state, ops);
            }
        }
    }
    s
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

#[no_mangle]
pub fn run_code(code: &str) {

    let init_state = State {
        curr_ptr: 0,
        data: HashMap::new(),
    };
    let (ast, _) = get_ast(&code);
    eval_vec(init_state, &ast);
}

fn my_string_safe(i: *mut c_char) -> String {
    unsafe { CStr::from_ptr(i).to_string_lossy().into_owned() }
}


#[no_mangle]
pub fn js_run_code(code: *mut c_char) {
    let s = my_string_safe(code);
    println!("code: '{}'", s);
    run_code(&s.as_str());
}

fn main() {}