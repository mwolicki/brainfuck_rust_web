
use byteorder::{WriteBytesExt, LittleEndian};

use brainfuck::*;
use std::fmt;
use leb128;

const EXTERNAL_CALL_PRINT:u8 = 0;
const EXTERNAL_CALL_READ:u8 = 1;

const FUNC:u8 = 0x60;
const I32:u8 = 0x7f;
const GET_LOCAL:u8 = 0x20;
const SET_LOCAL:u8 = 0x21;
const TEE_LOCAL:u8 = 0x22;
const I32_ADD:u8 = 0x6a;
const I32_SUB:u8 = 0x6b;
const I32_CONST:u8 = 0x41;
const I32_STORE8:u8 = 0x3a;
const I32_LOAD8_U:u8 = 0x2d;
const I32_EQZ:u8 = 0x45;
const BLOCK:u8 = 0x02;
const LOOP:u8 = 0x03;
const BR:u8 = 0x0c;
const BRIF:u8 = 0x0d;
const VOID:u8 = 0x40;
const END:u8 = 0x0b;
const CALL:u8 = 0x10;

const WASM_MAGIC:u32 = 0x6d73_6100;
const WASM_VERSION:u32 = 0x1;

type Position = u8;

#[derive(Clone)]
enum Wast {
    Call(u8), //should be LEB128
    I32Const(u8), //should be LEB128
    I32Store8,
    I32Load8u,
    I32Eqz,
    SetLocal(Position),
    GetLocal(Position),
    TeeLocal(Position),
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
            Wast::I32Eqz => vec.write_u8(I32_EQZ).unwrap(),
            Wast::Block => {
                vec.write_u8(BLOCK).unwrap();
                vec.write_u8(VOID).unwrap();
            },
            Wast::Loop => {
                vec.write_u8(LOOP).unwrap();
                vec.write_u8(VOID).unwrap();
            },
            Wast::End => vec.write_u8(END).unwrap(),
            Wast::I32Const(n) => {
                vec.write_u8(I32_CONST).unwrap();
                vec.write_u8(n).unwrap();
            },
            Wast::SetLocal(n) => {
                vec.write_u8(SET_LOCAL).unwrap();
                vec.write_u8(n).unwrap();
            },
            Wast::GetLocal(n) => {
                vec.write_u8(GET_LOCAL).unwrap();
                vec.write_u8(n).unwrap();
            },
            Wast::TeeLocal(n) => {
                vec.write_u8(TEE_LOCAL).unwrap();
                vec.write_u8(n).unwrap();
            },
            Wast::Call(n) => {
                vec.write_u8(CALL).unwrap();
                vec.write_u8(n).unwrap();
            },
            Wast::Br(n) => {
                vec.write_u8(BR).unwrap();
                vec.write_u8(n).unwrap();
            },
            Wast::BrIf(n) => {
                vec.write_u8(BRIF).unwrap();
                vec.write_u8(n).unwrap();
            },
        }
    }
}


fn simple_optimasation(code:&Vec<Wast>) -> Vec<Wast> {
    let mut r = vec![];

    let mut i = 0;
    let len = code.len ();
    while i < len {
        let op =  if i + 1 < len {
            match (&code[i], &code[i+1]) {
                (&Wast::SetLocal(ref a), &Wast::GetLocal(ref b)) if a == b =>
                    {
                        i+=1;
                        Wast::TeeLocal(a.clone())
                    },
                (w, _) => w.clone()
            }
        }
        else{
            code[i].clone()
        };

        r.push(op);
        i+=1;
    }
    r
}

type Name = String;
type NumberOfI32 = u8;
struct TypeDef {
    params : NumberOfI32,
    result : bool
}

impl TypeDef {
    fn to_binary (&self, vec: &mut Vec<u8>) {
        vec.write_u8(FUNC).unwrap();
        vec.write_u8(self.params).unwrap();
        for _ in 0..self.params {
            vec.write_u8(I32).unwrap();        
        }

        if self.result {
            vec.write_u8(1).unwrap();        
            vec.write_u8(I32).unwrap();                    
        }
        else{
            vec.write_u8(0).unwrap();                    
        }
    }
}

type Imports = Vec<(Name, Name, TypeDef)>;
type Functions = Vec<(Name, TypeDef, NumberOfI32, Vec<Wast>)>;

struct Module {
    imports: Imports,
    functions: Functions
}

impl Module {
    fn to_binary (&self, vec: &mut Vec<u8>) {
        fn types_section (module: &Module, vec: &mut Vec<u8>){
            let elements = (module.functions.len() + module.imports.len()) as u8;
            if elements > 0 {

                let mut td_vec = vec![];
                td_vec.write_u8(elements).unwrap();
                for &(_, _, ref td) in &module.imports {
                    td.to_binary(&mut td_vec);
                }

                for &(_, ref td, _, _) in &module.functions {
                    td.to_binary(&mut td_vec);
                }
                const TYPES_SECTION : u8 = 1;
                vec.write_u8(TYPES_SECTION).unwrap();
                write_leb128(td_vec.len() as u32, vec);
                vec.append(&mut td_vec);
            }
        }

        fn write_leb128(i:u32, vec: &mut Vec<u8>) {
            let mut n = leb128::encode_unsigned(i);
            vec.append(&mut n);
        }

        fn append_wasm_string(s:&String, vec: &mut Vec<u8>) {
            let mut bytes = s.clone().into_bytes();
            write_leb128(bytes.len() as u32,  vec);
            vec.append(&mut bytes);
        }

        fn import_section (imports: &Imports, vec: &mut Vec<u8>){
            let elements = imports.len() as u8;
            if elements > 0 {
                let mut ims_vec = vec![];
                ims_vec.write_u8(elements).unwrap();
            
                let mut i = 0;
                
                for &(ref module_str, ref field_str, _) in imports {
                    append_wasm_string(module_str, &mut ims_vec);
                    append_wasm_string(field_str, &mut ims_vec);
                    ims_vec.write_u8(0).unwrap();//kind
                    ims_vec.write_u8(i).unwrap();//signature
                    i+=1;
                }

                const IMPORTS_SECTION : u8 = 2;
                vec.write_u8(IMPORTS_SECTION).unwrap();
                write_leb128(ims_vec.len() as u32, vec);
                vec.append(&mut ims_vec);
            }
        }

        fn functions_section (functions: &Functions, imports_no :u8, vec: &mut Vec<u8>){
            let elements = functions.len() as u8;
            if elements > 0 {
                let mut fns_vec = vec![];
                fns_vec.write_u8(elements).unwrap();
            
                let mut i = imports_no;
                
                for _ in functions {
                    fns_vec.write_u8(i).unwrap();
                    i+=1;
                }

                const FUNCTIONS_SECION : u8 = 3;
                vec.write_u8(FUNCTIONS_SECION).unwrap();
                write_leb128(fns_vec.len() as u32, vec);
                vec.append(&mut fns_vec);
            }
        }

        fn exports_section (functions: &Functions, no_imports:u8, vec: &mut Vec<u8>){
            let elements = functions.len() as u8;
            if elements > 0 {
                let mut fns_vec = vec![];
                fns_vec.write_u8(elements).unwrap();
            
                let mut i = no_imports;
                
                for &(ref name, _, _, _) in functions {
                    append_wasm_string(name, &mut fns_vec);
                    fns_vec.write_u8(0).unwrap();//kind
                    fns_vec.write_u8(i).unwrap();//signature
                    i+=1;
                }

                const EXPORTS_SECTION : u8 = 7;
                vec.write_u8(EXPORTS_SECTION).unwrap();
                write_leb128(fns_vec.len() as u32, vec);
                vec.append(&mut fns_vec);
            }
        }

        fn memory_section (vec: &mut Vec<u8>){
        
            let mut fns_vec = vec![];
            fns_vec.write_u8(1).unwrap(); //1 section
            fns_vec.write_u8(0).unwrap(); //flags

            write_leb128(4092, &mut fns_vec);

            const MEMORY_SECTION : u8 = 5;
            vec.write_u8(MEMORY_SECTION).unwrap();
            write_leb128(fns_vec.len() as u32, vec);
            vec.append(&mut fns_vec);
        }

        fn code_section (functions: &Functions, vec: &mut Vec<u8>){
            let elements = functions.len() as u8;
            if elements > 0 {
                let mut fns_vec = vec![];
                fns_vec.write_u8(elements).unwrap();
            
                for &(_, _, ref local_vars, ref wasmt) in functions {
                    let mut code = vec![];

                    code.write_u8(*local_vars).unwrap();
                    for _ in 0..*local_vars {
                        code.write_u8(1).unwrap(); //type
                        code.write_u8(I32).unwrap();
                    }
                    for w in wasmt {
                        w.to_binary(&mut code);
                    }
                    code.write_u8(END).unwrap();
                    write_leb128(code.len() as u32, &mut fns_vec);
                    fns_vec.append(&mut code);
                    
                }
                const CODE_SECTION : u8 = 10;
                vec.write_u8(CODE_SECTION).unwrap();
                write_leb128(fns_vec.len() as u32, vec);
                vec.append(&mut fns_vec);
            }
        }

        vec.write_u32::<LittleEndian>(WASM_MAGIC).unwrap();
        vec.write_u32::<LittleEndian>(WASM_VERSION).unwrap();

        types_section(self, vec);
        import_section(&self.imports, vec);
        functions_section(&self.functions, self.imports.len() as u8, vec);
        memory_section(vec);
        exports_section(&self.functions, self.imports.len() as u8, vec);
        code_section(&self.functions, vec);
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
            Wast::TeeLocal (i) => write!(f, "tee_local {}", i),
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


pub fn to_wasm (ops: &[Op]) -> Vec<u8> {
    let mut wast = vec![];
    
    for op in ops {
        to_wasmt(op, &mut wast);
    }

    let wast = simple_optimasation(&wast);

    let module = Module{
         imports : vec![("io".to_owned(), "print".to_owned(), TypeDef{ result : false, params : 1 }),
                        ("io".to_owned(), "read".to_owned(), TypeDef{ result : true, params : 0 })],
        functions : vec![("exec".to_owned(), TypeDef{
            result : false,
            params : 0
        }, 1, wast)],
    };

    let mut module_bin = vec![];
    module.to_binary(&mut module_bin);
    module_bin
    
}