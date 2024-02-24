use logic_form::{Var, VarMap};

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
