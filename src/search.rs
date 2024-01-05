use crate::{Conflict, Model, SatResult, Solver};
use logic_form::Lit;

impl Solver {
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
        while let Some(decide) = self.vsids.pop() {
            if self.value[decide.lit()].is_none() {
                self.assign(decide.lit(), None);
                return true;
            }
        }
        false
    }

    pub fn backtrack(&mut self, level: usize) {
        assert!(level != 0);
        while self.trail.len() >= self.pos_in_trail[level] {
            let bt = self.trail.pop().unwrap();
            self.value[bt] = None;
            self.value[!bt] = None;
            self.vsids.push(bt.var());
        }
        self.pos_in_trail.truncate(level);
    }

    pub fn search(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        loop {
            if let Some(conflict) = self.propagate() {
                if self.level[self.clauses[conflict][0]] == 0 {
                    return SatResult::Unsat(Conflict { solver: self });
                }
                let (learnt, btl) = self.analyze(conflict);
                self.backtrack(btl);
                if learnt.len() == 1 {
                    self.assign(learnt[0], None);
                } else {
                    let learnt_idx = self.add_clause(&learnt);
                    self.assign(learnt[0], Some(learnt_idx));
                }
            } else if !self.decide() {
                return SatResult::Sat(Model { solver: self });
            }
        }
    }
}
