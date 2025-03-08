use crate::parser::Register;

use super::RegisterSet;

#[allow(clippy::match_same_arms)]
#[must_use]
pub fn environment_in_outs(call_num: i32) -> Option<(RegisterSet, RegisterSet)> {
    use crate::parser::Register::{X10, X11, X12, X13};
    let (args, rets): (&[Register], &[Register]) = match call_num {
        1 => (&[X10], &[]),
        // 2 => (&[], &[]), Not supporting floating point yet
        // 3 => (&[], &[]),
        4 => (&[X10], &[]),
        5 => (&[], &[X10]),
        // 6 => (&[], &[]),
        // 7 => (&[], &[]),
        8 => (&[X10, X11], &[]),
        9 => (&[X10], &[X10]),
        10 => (&[], &[]),
        11 => (&[X10], &[]),
        12 => (&[], &[X10]),
        17 => (&[X10, X11], &[X10]),
        30 => (&[], &[X10, X11]),
        31 => (&[X10, X11, X12, X13], &[]),
        32 => (&[X10], &[]),
        33 => (&[X10, X11, X12, X13], &[]),
        34 => (&[X10], &[]),
        35 => (&[X10], &[]),
        36 => (&[X10], &[]),
        40 => (&[X10, X11], &[]),
        41 => (&[X10], &[X10]),
        42 => (&[X10, X11], &[X10]),
        43 => (&[X10], &[X10]),
        // 44 => (&[X10], &[]),
        50 => (&[X10], &[X10]),
        54 => (&[X10, X11, X12], &[X11]),
        55 => (&[X10], &[]),
        56 => (&[X10, X11], &[]),
        57 => (&[X10], &[]),
        // 58 => (&[X10], &[]),
        59 => (&[X10, X11], &[]),
        // 60 => (&[X10], &[]),
        62 => (&[X10, X11, X12], &[X10]),
        63 => (&[X10, X11, X12], &[X10]),
        64 => (&[X10, X11, X12], &[X10]),
        93 => (&[X10], &[]),
        1024 => (&[X10, X11], &[X10]),
        _ => return None,
    };

    Some((
        args.iter().copied().collect(),
        rets.iter().copied().collect(),
    ))
}
