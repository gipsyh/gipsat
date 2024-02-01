use crate::{cdb::ClauseKind, Solver};
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
    fn new_level(&mut self) {
        self.pos_in_trail.push(self.trail.len())
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
            self.phase_saving[bt] = Some(bt);
        }
        self.propagated = self.pos_in_trail[level];
        self.pos_in_trail.truncate(level);
    }

    pub fn search(&mut self, assumption: &[Lit]) -> bool {
        'ml: loop {
            if let Some(conflict) = self.propagate() {
                if self.args.verbose {
                    println!("{:?}", &self.cdb[conflict]);
                }
                if self.highest_level() == 0 {
                    return false;
                }
                let (learnt, btl) = self.analyze(conflict);
                if self.args.verbose {
                    dbg!(btl);
                }
                self.backtrack(btl);
                if learnt.len() == 1 {
                    assert!(btl == 0);
                    self.assign(learnt[0], None);
                } else {
                    let mut kind = ClauseKind::Origin;
                    for l in learnt.iter() {
                        if let Some(act) = self.constrain_act {
                            if act.var() == l.var() {
                                kind = ClauseKind::Temporary;
                            }
                        }
                    }
                    let learnt_id = self.attach_clause(&learnt, kind);
                    // self.cdb.bump(learnt_idx);
                    self.assign(self.cdb[learnt_id][0], Some(learnt_id));
                }
                self.vsids.decay();
                self.cdb.decay();
            } else {
                self.reduce();
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
