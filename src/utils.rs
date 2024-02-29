use std::ops::Not;

use logic_form::{Var, VarMap};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Lbool(pub u8);

impl Lbool {
    pub const FALSE: Lbool = Lbool(0);
    pub const TRUE: Lbool = Lbool(1);
    pub const NONE: Lbool = Lbool(2);

    #[inline]
    pub fn is_true(self) -> bool {
        self == Self::TRUE
    }

    #[inline]
    pub fn is_false(self) -> bool {
        self == Self::FALSE
    }

    #[inline]
    pub fn is_none(self) -> bool {
        self.0 & 2 != 0
    }
}

impl Default for Lbool {
    fn default() -> Self {
        Self::NONE
    }
}

impl Not for Lbool {
    type Output = Self;

    fn not(self) -> Self::Output {
        Lbool(self.0 ^ 1)
    }
}

#[derive(Default)]
pub struct VarMark {
    map: VarMap<bool>,
    marks: Vec<Var>,
}

impl VarMark {
    pub fn reserve(&mut self, var: Var) {
        self.map.reserve(var);
    }

    #[inline]
    pub fn has(&self, var: Var) -> bool {
        self.map[var]
    }

    #[inline]
    pub fn mark(&mut self, var: Var) {
        if !self.map[var] {
            self.map[var] = true;
            self.marks.push(var);
        }
    }

    #[inline]
    pub fn marks(&self) -> &[Var] {
        &self.marks
    }
}
