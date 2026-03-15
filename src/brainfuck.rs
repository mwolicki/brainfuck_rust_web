

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


#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use super::*;

    fn simple_op_strategy() -> impl Strategy<Value = Op> {
        prop_oneof![
            Just(Op::IncPointer(1)),
            Just(Op::DecPointer(1)),
            Just(Op::IncVal(1)),
            Just(Op::DecVal(1)),
            Just(Op::Print),
            Just(Op::Read),
        ]
    }

    fn compactable_op_strategy() -> impl Strategy<Value = Op> {
        prop_oneof![
            Just(Op::IncPointer(1)),
            Just(Op::DecPointer(1)),
            Just(Op::IncVal(1)),
            Just(Op::DecVal(1)),
        ]
    }

    fn count_print(ops: &[Op]) -> usize {
        ops.iter().filter(|op| **op == Op::Print).count()
    }

    fn count_read(ops: &[Op]) -> usize {
        ops.iter().filter(|op| **op == Op::Read).count()
    }

    fn sum_inc_ptr(ops: &[Op]) -> usize {
        ops.iter().map(|op| if let Op::IncPointer(n) = op { *n } else { 0 }).sum()
    }

    fn sum_dec_ptr(ops: &[Op]) -> usize {
        ops.iter().map(|op| if let Op::DecPointer(n) = op { *n } else { 0 }).sum()
    }

    fn sum_inc_val(ops: &[Op]) -> usize {
        ops.iter().map(|op| if let Op::IncVal(n) = op { *n as usize } else { 0 }).sum()
    }

    fn sum_dec_val(ops: &[Op]) -> usize {
        ops.iter().map(|op| if let Op::DecVal(n) = op { *n as usize } else { 0 }).sum()
    }

    proptest! {
        // --- Parser (get_ast) properties ---

        #[test]
        fn non_bf_chars_produce_empty_ast(s in "[^><+\\-.,\\[\\]]*") {
            let chars: Vec<char> = s.chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert!(ast.is_empty(), "Expected empty AST for {:?}, got {:?}", s, ast);
        }

        #[test]
        fn gt_chars_map_to_inc_pointer_sum(n in 1usize..=50usize) {
            let chars: Vec<char> = ">".repeat(n).chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert_eq!(sum_inc_ptr(&ast), n);
        }

        #[test]
        fn lt_chars_map_to_dec_pointer_sum(n in 1usize..=50usize) {
            let chars: Vec<char> = "<".repeat(n).chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert_eq!(sum_dec_ptr(&ast), n);
        }

        #[test]
        fn plus_chars_map_to_inc_val_sum(n in 1usize..=50usize) {
            let chars: Vec<char> = "+".repeat(n).chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert_eq!(sum_inc_val(&ast), n);
        }

        #[test]
        fn minus_chars_map_to_dec_val_sum(n in 1usize..=50usize) {
            let chars: Vec<char> = "-".repeat(n).chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert_eq!(sum_dec_val(&ast), n);
        }

        #[test]
        fn dot_chars_map_to_print_ops(n in 1usize..=50usize) {
            let chars: Vec<char> = ".".repeat(n).chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert_eq!(count_print(&ast), n);
        }

        #[test]
        fn comma_chars_map_to_read_ops(n in 1usize..=50usize) {
            let chars: Vec<char> = ",".repeat(n).chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert_eq!(count_read(&ast), n);
        }

        #[test]
        fn non_bf_chars_interspersed_are_ignored(
            n in 1usize..=20usize,
            noise in "[^><+\\-.,\\[\\]]*"
        ) {
            let code = format!("{}{}", ">".repeat(n), noise);
            let chars: Vec<char> = code.chars().collect();
            let (ast, _) = get_ast(&chars);
            prop_assert_eq!(sum_inc_ptr(&ast), n);
        }

        // --- compact() properties ---

        #[test]
        fn compact_is_idempotent(ops in prop::collection::vec(simple_op_strategy(), 0..30)) {
            let once = compact(&ops);
            let twice = compact(&once);
            prop_assert_eq!(once, twice, "compact is not idempotent for input {:?}", ops);
        }

        #[test]
        fn compact_preserves_print_count(ops in prop::collection::vec(simple_op_strategy(), 0..30)) {
            let original = count_print(&ops);
            prop_assert_eq!(count_print(&compact(&ops)), original);
        }

        #[test]
        fn compact_preserves_read_count(ops in prop::collection::vec(simple_op_strategy(), 0..30)) {
            let original = count_read(&ops);
            prop_assert_eq!(count_read(&compact(&ops)), original);
        }

        #[test]
        fn compact_merges_consecutive_inc_pointer(n in 2usize..=20usize) {
            let ops: Vec<Op> = std::iter::repeat(Op::IncPointer(1)).take(n).collect();
            prop_assert_eq!(compact(&ops), vec![Op::IncPointer(n)]);
        }

        #[test]
        fn compact_merges_consecutive_dec_pointer(n in 2usize..=20usize) {
            let ops: Vec<Op> = std::iter::repeat(Op::DecPointer(1)).take(n).collect();
            prop_assert_eq!(compact(&ops), vec![Op::DecPointer(n)]);
        }

        #[test]
        fn compact_merges_consecutive_inc_val(n in 2usize..=20usize) {
            let ops: Vec<Op> = std::iter::repeat(Op::IncVal(1)).take(n).collect();
            prop_assert_eq!(compact(&ops), vec![Op::IncVal(n as u8)]);
        }

        #[test]
        fn compact_merges_consecutive_dec_val(n in 2usize..=20usize) {
            let ops: Vec<Op> = std::iter::repeat(Op::DecVal(1)).take(n).collect();
            prop_assert_eq!(compact(&ops), vec![Op::DecVal(n as u8)]);
        }

        #[test]
        fn compact_no_adjacent_identical_compactable_ops(
            ops in prop::collection::vec(compactable_op_strategy(), 0..30)
        ) {
            let compacted = compact(&ops);
            for window in compacted.windows(2) {
                prop_assert!(
                    window[0] != window[1],
                    "Adjacent identical ops after compact: {:?}", compacted
                );
            }
        }

        #[test]
        fn compact_dec_loop_becomes_set_register_to_zero(
            prefix in prop::collection::vec(compactable_op_strategy(), 0..5),
            suffix in prop::collection::vec(compactable_op_strategy(), 0..5),
        ) {
            let mut ops = prefix;
            ops.push(Op::While { ops: vec![Op::DecVal(1)] });
            ops.extend(suffix);
            let compacted = compact(&ops);
            prop_assert!(
                compacted.iter().any(|op| *op == Op::SetRegisterToZero),
                "Expected SetRegisterToZero in {:?}", compacted
            );
        }

        #[test]
        fn compact_inc_loop_becomes_set_register_to_zero(
            prefix in prop::collection::vec(compactable_op_strategy(), 0..5),
            suffix in prop::collection::vec(compactable_op_strategy(), 0..5),
        ) {
            let mut ops = prefix;
            ops.push(Op::While { ops: vec![Op::IncVal(1)] });
            ops.extend(suffix);
            let compacted = compact(&ops);
            prop_assert!(
                compacted.iter().any(|op| *op == Op::SetRegisterToZero),
                "Expected SetRegisterToZero in {:?}", compacted
            );
        }
    }
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
