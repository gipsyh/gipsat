use crate::Solver;
use logic_form::Lit;

impl Solver {
    #[inline]
    pub fn level(&self, lit: Lit) -> usize {
        self.level[lit.var()]
    }

    #[inline]
    pub fn highest_level(&self) -> usize {
        self.pos_in_trail.len()
    }

    pub fn assign(&mut self, lit: Lit, reason: Option<usize>) {
        self.trail.push(lit);
        self.value[lit] = Some(true);
        self.value[!lit] = Some(false);
        self.reason[lit.var()] = reason;
    }

    pub fn decide(&mut self) -> bool {
        todo!()
    }

    pub fn search(&mut self) -> bool {
        loop {
            if let Some(cref) = self.propagate() {
            } else {
                if !self.decide() {
                    return true;
                }
            }
        }
    }
}
