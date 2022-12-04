use crate::common::*;

use super::def::*;

impl ShiftInRange for Nf {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Nf::TypCtor { info, name, args } => Nf::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Nf::Ctor { info, name, args } => Nf::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Nf::Type { info } => Nf::Type { info: info.clone() },
            Nf::Comatch { info, name, body } => Nf::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.shift_in_range(range, by),
            },
            Nf::Neu { exp } => Nf::Neu { exp: exp.shift_in_range(range, by) },
        }
    }
}

impl ShiftInRange for Neu {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Neu::Var { info, name, idx } => Neu::Var {
                info: info.clone(),
                name: name.clone(),
                idx: idx.shift_in_range(range, by),
            },
            Neu::Dtor { info, exp, name, args } => Neu::Dtor {
                info: info.clone(),
                exp: exp.shift_in_range(range.clone(), by),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Neu::Match { info, name, on_exp, body } => Neu::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.shift_in_range(range.clone(), by),
                body: body.shift_in_range(range, by),
            },
        }
    }
}

impl ShiftInRange for Match {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.shift_in_range(range, by) }
    }
}

impl ShiftInRange for Comatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Comatch { info, cases } = self;
        Comatch { info: info.clone(), cases: cases.shift_in_range(range, by) }
    }
}

impl ShiftInRange for Case {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { info, name, args, body } = self;

        Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl ShiftInRange for Cocase {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Cocase { info, name, args, body } = self;

        Cocase {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}