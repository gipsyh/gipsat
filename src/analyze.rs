use crate::Solver;
use logic_form::{Clause, Lit};

impl Solver {
    pub fn analyze(&mut self, conflict: usize) -> (Clause, usize) {
        let mut learnt = Clause::from([Lit::default()]);
        let mut path = 0;
        let mut trail_idx = self.trail.len() - 1;
        let mut resolve_lit = None;
        let mut conflict = Some(conflict);
        loop {
            let cref = &self.clauses[conflict.unwrap()];
            let begin = if resolve_lit.is_some() { 1 } else { 0 };
            for l in begin..cref.len() {
                let lit = cref[l];
                if !self.seen[lit] && self.level[lit] > 0 {
                    self.vsids.var_bump(lit.var());
                    self.seen[lit] = true;
                    if self.level[lit] >= self.highest_level() {
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
            resolve_lit = Some(self.trail[trail_idx]);
            conflict = self.reason[self.trail[trail_idx]];
            path -= 1;
            if path == 0 {
                break;
            }
        }
        learnt[0] = !resolve_lit.unwrap();
        let btl = if learnt.len() == 1 {
            0
        } else {
            let max_idx = (1..learnt.len())
                .max_by_key(|idx| self.level[learnt[*idx]])
                .unwrap();
            learnt.swap(1, max_idx);
            self.level[learnt[1]]
        };
        for l in learnt.iter() {
            self.seen[*l] = false;
        }
        (learnt, btl)
    }
}
