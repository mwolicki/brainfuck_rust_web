

#[derive(PartialEq, Clone, Debug)]
pub enum Op {
    IncPointer(usize),
    DecPointer(usize),
    IncVal(u8),
    DecVal(u8),
    Print,
    Read,
    While { ops: Vec<Op> },
    SetRegisterToZero,
}

pub fn compact(ast: &[Op]) -> Vec<Op> {
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
            Op::While { ref ops } => {
                let compacted_ops = compact(ops);
                if compacted_ops == [Op::IncVal(1)] || compacted_ops == [Op::DecVal(1)] {
                    compacted_ast.push(Op::SetRegisterToZero)
                } else {
                    compacted_ast.push(Op::While { ops: compacted_ops })
                }
            }
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

pub fn get_ast(code: &[char]) -> (Vec<Op>, usize) {
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
