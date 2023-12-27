use logic_form::Lit;

use crate::Solver;

impl Solver {
    pub fn assign(&mut self, lit: Lit) {
        self.trail.push(lit);
        self.value[lit] = Some(true);
        self.value[!lit] = Some(false);
    }
}
