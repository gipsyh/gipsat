use crate::Solver;
use logic_form::Lit;

impl Solver {
    #[inline]
    pub fn highest_level(&self) -> usize {
        self.pos_in_trail.len()
    }

    #[inline]
    pub fn assign(&mut self, lit: Lit, reason: Option<usize>) {
        assert!(self.value[lit].is_none() && self.value[!lit].is_none());
        self.trail.push(lit);
        self.value[lit] = Some(true);
        self.value[!lit] = Some(false);
        self.reason[lit] = reason;
        self.level[lit] = self.highest_level();
    }

    #[inline]
    pub fn new_level(&mut self) {
        self.pos_in_trail.push(self.trail.len())
    }

    #[inline]
    pub fn decide(&mut self) -> bool {
        while let Some(decide) = self.vsids.pop() {
            if self.value[decide.lit()].is_none() {
                let decide = self.phase_saving[decide].unwrap_or(decide.lit());
                self.new_level();
                self.assign(decide, None);
                return true;
            }
        }
        false
    }

    pub fn backtrack(&mut self, level: usize) {
        if self.highest_level() == level {
            return;
        }
        while self.trail.len() > self.pos_in_trail[level] {
            let bt = self.trail.pop().unwrap();
            self.value[bt] = None;
            self.value[!bt] = None;
            self.vsids.push(bt.var());
            // self.phase_saving[bt] = Some(bt);
        }
        self.propagated = self.pos_in_trail[level];
        self.pos_in_trail.truncate(level);
    }

    // pub fn restart(&mut self) {

    // }

    pub fn search(&mut self, assumption: &[Lit]) -> bool {
        'ml: loop {
            if self.args.verbose {
                self.print_value();
            }
            if let Some(conflict) = self.propagate() {
                if self.args.verbose {
                    println!("{:?}", &self.clauses[conflict]);
                }
                if self.highest_level() == 0 {
                    return false;
                }
                let (learnt, btl) = self.analyze(conflict);
                if self.args.verbose {
                    dbg!(btl);
                }
                let lbd = self.calculate_lbd(&learnt);
                self.lbd_queue.push(lbd);
                self.backtrack(btl);
                if learnt.len() == 1 {
                    assert!(btl == 0);
                    self.assign(learnt[0], None);
                } else {
                    let learnt_idx = self.add_learnt_clause(learnt, lbd);
                    self.assign(self.clauses[learnt_idx][0], Some(learnt_idx));
                }
                self.vsids.var_decay();
                self.reduces += 1;
            } else {
                if self.reduces > self.reduce_limit {
                    self.reduce();
                }
                while self.highest_level() < assumption.len() {
                    let a = assumption[self.highest_level()];
                    match self.value[a] {
                        Some(true) => self.new_level(),
                        Some(false) => {
                            self.analyze_unsat_core(a);
                            return false;
                        }
                        None => {
                            self.new_level();
                            self.assign(a, None);
                            continue 'ml;
                        }
                    }
                }
                if !self.decide() {
                    return true;
                }
            }
        }
    }
}
