use logic_form::Clause;

use crate::Solver;

impl Solver {
    pub fn analyze(&mut self, mut conflict: usize) -> (Clause, usize) {
        let mut learnt = Clause::new();
        let mut path = 0;
        // if self.level(cref[0]) == 0 {
        //     // return false;
        //     todo!()
        // }
        let mut trail_idx = self.trail.len() - 1;
        let mut resolve_lit = None;
        let mut conflict = Some(conflict);
        loop {
            let mut cref = &self.clauses[conflict.unwrap()];
            for l in 0..cref.len() {
                let lit = cref[l];
                if !self.seen[lit] && self.level(lit) > 0 {
                    if self.level(lit) >= self.highest_level() {
                        path += 1;
                    } else {
                        learnt.push(lit);
                    }
                }
            }
            while !self.seen[self.trail[trail_idx]] {
                trail_idx -= 1;
            }
            self.seen[self.trail[trail_idx]] = false;
            conflict = self.reason[self.trail[trail_idx].var()];
            path -= 1;
            if path == 0 {
                break;
            }
        }
        todo!()
    }
}
