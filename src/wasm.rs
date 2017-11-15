
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
const WASM_MAGIC:u8 = 0x6d736100;
const WASM_VERSION:u8 = 0x1;

type Position = u8;

enum Wast {
    Call(u8), //should be LEB128
    I32Const(u8), //should be LEB128
    I32Store8,
    I32Load8u,
    SetLocal(Position),
    GetLocal(Position),
    I32Add,
    I32Sub,
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
        }
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
        _ => ()
    }
}


pub fn to_wasm (ops: &[Op]) {
        let mut bin = vec![];

    for op in ops {
        let mut wast = vec![];
        to_wasmt(op, &mut wast);
        for w in &wast {
            println!("  {}", w);
        }
        
        
        for w in  wast{
            
            w.to_binary(&mut bin);
            
        }
    }
    let mut file = File::create("foo.txt").unwrap();
    file.write_all(&bin).unwrap();;


        println!("{:?}", bin);
    
}
