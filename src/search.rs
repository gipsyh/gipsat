use crate::{propagate::Watcher, Conflict, Model, SatResult, Solver};
use logic_form::{Clause, Lit, Var};

impl Solver {
    #[inline]
    pub fn highest_level(&self) -> usize {
        self.pos_in_trail.len()
    }

    pub fn assign(&mut self, lit: Lit, reason: Option<usize>) {
        self.trail.push(lit);
        self.value[lit] = Some(true);
        self.value[!lit] = Some(false);
        self.reason[lit] = reason;
        self.level[lit] = self.highest_level();
    }

    pub fn print_value(&self) {
        for v in 0..self.num_var() {
            let lit = Var::new(v).lit();
            match self.value[lit] {
                Some(true) => print!("{:?}", lit),
                Some(false) => print!("{:?}", !lit),
                None => print!("X"),
            };
            print!("\t");
        }
        println!();
    }

    pub fn decide(&mut self) -> bool {
        while let Some(decide) = self.vsids.pop() {
            if self.value[decide.lit()].is_none() {
                if self.args.verbose {
                    dbg!(decide);
                }
                self.pos_in_trail.push(self.trail.len());
                self.assign(decide.lit(), None);
                return true;
            }
        }
        false
    }

    pub fn backtrack(&mut self, level: usize) {
        assert!(self.highest_level() > level);
        while self.trail.len() > self.pos_in_trail[level] {
            let bt = self.trail.pop().unwrap();
            self.value[bt] = None;
            self.value[!bt] = None;
            self.vsids.push(bt.var());
        }
        self.propagated = self.pos_in_trail[level];
        self.pos_in_trail.truncate(level);
    }

    pub fn add_clause_inner(&mut self, clause: Clause) -> usize {
        assert!(clause.len() > 1);
        let id = self.clauses.len();
        self.watchers[!clause[0]].push(Watcher::new(id, clause[1]));
        self.watchers[!clause[1]].push(Watcher::new(id, clause[0]));
        self.clauses.push(clause);
        self.clauses.len() - 1
    }

    pub fn search(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        loop {
            if self.args.verbose {
                self.print_value();
            }
            if let Some(conflict) = self.propagate() {
                if self.args.verbose {
                    println!("{:?}", &self.clauses[conflict]);
                }
                if self.highest_level() == 0 {
                    return SatResult::Unsat(Conflict { solver: self });
                }
                let (learnt, btl) = self.analyze(conflict);
                if self.args.verbose {
                    dbg!(btl);
                }
                self.backtrack(btl);
                if learnt.len() == 1 {
                    self.assign(learnt[0], None);
                } else {
                    let learnt_idx = self.add_clause_inner(learnt);
                    self.assign(self.clauses[learnt_idx][0], Some(learnt_idx));
                }
            } else if !self.decide() {
                return SatResult::Sat(Model { solver: self });
            }
        }
    }
}
