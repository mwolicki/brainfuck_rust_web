
use byteorder::{WriteBytesExt, LittleEndian};

use std::fs::File;
use std::io::prelude::*;
use brainfuck::*;
use std::fmt;

const EXTERNAL_CALL_PRINT:u8 = 0;
const EXTERNAL_CALL_READ:u8 = 1;

const GET_LOCAL:u8 = 0x20;
const SET_LOCAL:u8 = 0x21;
const I32_ADD:u8 = 0x6a;
const I32_SUB:u8 = 0x6b;
const I32_CONST:u8 = 0x41;
const I32_STORE8:u8 = 0x3a;
const I32_LOAD8_U:u8 = 0x2d;
const END:u8 = 0x0b;
const CALL:u8 = 0x10;

const WASM_MAGIC:u32 = 0x6d736100;
const WASM_VERSION:u32 = 0x1;
const TYPE_I32:u8 = 0x7f;

type Position = u8;

enum Wast {
    Call(u8), //should be LEB128
    I32Const(u8), //should be LEB128
    I32Store8,
    I32Load8u,
    I32Eqz,
    SetLocal(Position),
    GetLocal(Position),
    I32Add,
    I32Sub,
    Loop,
    Block,
    End,
    Br(u8),
    BrIf(u8),
}

impl Wast {
    fn to_binary (&self, vec: &mut Vec<u8>) {
        match *self {
            Wast::I32Store8 => {
                vec.write_u8(I32_STORE8).unwrap();
                vec.write_u16::<LittleEndian>(0).unwrap(); // alignment
            },
            Wast::I32Load8u => {
                vec.write_u8(I32_LOAD8_U).unwrap();
                vec.write_u16::<LittleEndian>(0).unwrap(); // alignment
                
            },
            Wast::I32Add => vec.write_u8(I32_ADD).unwrap(),
            Wast::I32Sub => vec.write_u8(I32_SUB).unwrap(),
            Wast::I32Const(n) => {
                vec.write_u8(I32_CONST).unwrap();
                vec.write_u8(n).unwrap(); // alignment
            },
            Wast::SetLocal(n) => {
                vec.write_u8(SET_LOCAL).unwrap();
                vec.write_u8(n).unwrap(); // alignment
            },
            Wast::GetLocal(n) => {
                vec.write_u8(GET_LOCAL).unwrap();
                vec.write_u8(n).unwrap(); // alignment
            },
            Wast::Call(n) => {
                vec.write_u8(CALL).unwrap();
                vec.write_u8(n).unwrap(); // alignment
            },
            _ => {}
        }
    }
}


type Name = String;
type NumberOfI32 = u8;
struct TypeDef {
    name: Name,
    params : NumberOfI32,
    result : NumberOfI32
}

struct Module {
    imports: Vec<TypeDef>,
    functions: Vec<(TypeDef, NumberOfI32, Vec<Wast>)>
}



impl Module {
    fn to_binary (&self, vec: &mut Vec<u8>) {
        vec.write_u32::<LittleEndian>(WASM_MAGIC).unwrap();
        vec.write_u32::<LittleEndian>(WASM_VERSION).unwrap();

        //Type
        vec.write_u8(1).unwrap();

        //for i 

    }
}

impl fmt::Display for Wast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Wast::I32Const (i) => write!(f, "i32.const {}", i),
            Wast::I32Store8 => write!(f, "i32.store8"),
            Wast::I32Load8u => write!(f, "i32.load8_u"),
            Wast::SetLocal (i) => write!(f, "set_local {}", i),
            Wast::GetLocal (i) => write!(f, "get_local {}", i),
            Wast::I32Add => write!(f, "i32.add"),
            Wast::I32Sub => write!(f, "i32.sub"),
            Wast::Block => write!(f, "block"),
            Wast::Loop => write!(f, "loop"),
            Wast::End => write!(f, "end"),
            Wast::Br (i) => write!(f, "br {}", i),
            Wast::BrIf (i) => write!(f, "br_if {}", i),
            Wast::I32Eqz => write!(f, "i32.eqz"),
            Wast::Call(i) => write!(f, "call {}", i),
        }
    }
}

fn to_wasmt (ops: &Op, res : &mut Vec<Wast>) {
    match *ops {
        Op::IncPointer(n) => {
            res.push(Wast::GetLocal(0));
            res.push(Wast::I32Const(n as u8));
            res.push(Wast::I32Add);
            res.push(Wast::SetLocal(0));
        },
        Op::DecPointer(n) => {
            res.push(Wast::GetLocal(0));
            res.push(Wast::I32Const(n as u8));
            res.push(Wast::I32Sub);
            res.push(Wast::SetLocal(0));
        },
        Op::IncVal(n) => {
            res.push(Wast::GetLocal(0));

            res.push(Wast::GetLocal(0));            
            res.push(Wast::I32Load8u);
            res.push(Wast::I32Const(n as u8));
            res.push(Wast::I32Add);
            res.push(Wast::I32Store8);
        },
        Op::DecVal(n) => {
            res.push(Wast::GetLocal(0));                        
            res.push(Wast::GetLocal(0));            
            res.push(Wast::I32Load8u);
            res.push(Wast::I32Const(n as u8));
            res.push(Wast::I32Sub);
            res.push(Wast::I32Store8);
        },
        Op::SetRegisterToZero => {
            res.push(Wast::GetLocal(0));
            res.push(Wast::I32Const(0));
            res.push(Wast::I32Store8);
        },
        Op::Print => {
            res.push(Wast::GetLocal(0));            
            res.push(Wast::I32Load8u);
            res.push(Wast::Call(EXTERNAL_CALL_PRINT));
        },
        Op::Read => {
            res.push(Wast::GetLocal(0));
            res.push(Wast::Call(EXTERNAL_CALL_READ));
            res.push(Wast::I32Store8);
        },
        Op::While {ref ops } => {
            res.push(Wast::Block);
            res.push(Wast::Loop);

            res.push(Wast::GetLocal(0));            
            res.push(Wast::I32Load8u);
            res.push(Wast::I32Eqz);
            res.push(Wast::BrIf(1));

            for o in ops {
                to_wasmt(o, res);
            }

            res.push(Wast::Br(0));

            res.push(Wast::End);
            res.push(Wast::End);
        }
    }
}


pub fn to_wasm (ops: &[Op]) {
    let mut wast = vec![];
    
    for op in ops {
        to_wasmt(op, &mut wast);
    }

    for w in &wast {
        println!("{}", w);
    }

    let module = Module{
        imports : vec![],
        functions : vec![(TypeDef{
            name : "exec".to_owned(),
            result : 0,
            params : 0
        }, 1, wast)],
    };
    
    // let mut file = File::create("foo.txt").unwrap();
    // file.write_all(&bin).unwrap();;


    //     println!("{:?}", bin);
    
}
