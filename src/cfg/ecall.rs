use std::collections::HashSet;

use crate::parser::register::Register;

pub fn ecall_in_outs(call_num: i32) -> Option<(HashSet<Register>, HashSet<Register>)> {
    use crate::parser::register::Register::{X10, X11, X12, X13};
    let vecs = match call_num {
        1 => (vec![X10], vec![]),
        // 2 => (vec![], vec![]), Not supporting floating point yet
        // 3 => (vec![], vec![]),
        4 => (vec![X10], vec![]),
        5 => (vec![], vec![X10]),
        // 6 => (vec![], vec![]),
        // 7 => (vec![], vec![]),
        8 => (vec![X10, X11], vec![]),
        9 => (vec![X10], vec![X10]),
        10 => (vec![], vec![]),
        11 => (vec![X10], vec![]),
        12 => (vec![], vec![X10]),
        17 => (vec![X10, X11], vec![X10]),
        30 => (vec![], vec![X10, X11]),
        31 => (vec![X10, X11, X12, X13], vec![]),
        32 => (vec![X10], vec![]),
        33 => (vec![X10, X11, X12, X13], vec![]),
        34 => (vec![X10], vec![]),
        35 => (vec![X10], vec![]),
        36 => (vec![X10], vec![]),
        40 => (vec![X10, X11], vec![]),
        41 => (vec![X10], vec![X10]),
        42 => (vec![X10, X11], vec![X10]),
        43 => (vec![X10], vec![X10]),
        // 44 => (vec![X10], vec![]),
        50 => (vec![X10], vec![X10]),
        54 => (vec![X10, X11, X12], vec![X11]),
        55 => (vec![X10], vec![]),
        56 => (vec![X10, X11], vec![]),
        57 => (vec![X10], vec![]),
        // 58 => (vec![X10], vec![]),
        59 => (vec![X10, X11], vec![]),
        // 60 => (vec![X10], vec![]),
        62 => (vec![X10, X11, X12], vec![X10]),
        63 => (vec![X10, X11, X12], vec![X10]),
        64 => (vec![X10, X11, X12], vec![X10]),
        93 => (vec![X10], vec![]),
        1024 => (vec![X10, X11], vec![X10]),
        _ => return None,
    };

    Some((
        HashSet::from_iter(vecs.0.into_iter()),
        HashSet::from_iter(vecs.1.into_iter()),
    ))
}

pub fn is_ecall_exit(call_num: i32) -> bool {
    call_num == 93 || call_num == 10
}
