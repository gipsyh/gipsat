use crate::{
    cdb::{CRef, ClauseKind, CREF_NONE},
    utils::Lbool,
    Sat, Solver, Unsat,
};
use logic_form::{Lit, Var, VarMap};
use satif::SatResult;

#[derive(Default)]
pub struct Value {
    data: VarMap<Lbool>,
}

impl Value {
    #[inline]
    pub fn reserve(&mut self, var: Var) {
        self.data.reserve(var)
    }

    #[inline]
    pub fn v(&self, lit: Lit) -> Lbool {
        Lbool(self.data[lit].0 ^ (!lit.polarity() as u8))
    }

    #[inline]
    pub fn set(&mut self, lit: Lit) {
        self.data[lit] = Lbool(lit.polarity() as u8)
    }

    #[inline]
    pub fn set_none(&mut self, var: Var) {
        self.data[var] = Lbool::NONE
    }
}

impl Solver {
    #[inline]
    pub fn highest_level(&self) -> usize {
        self.pos_in_trail.len()
    }

    #[inline]
    pub fn assign(&mut self, lit: Lit, reason: CRef) {
        self.trail.push(lit);
        self.value.set(lit);
        self.reason[lit] = reason;
        self.level[lit] = self.highest_level() as u32;
    }

    #[inline]
    fn new_level(&mut self) {
        self.pos_in_trail.push(self.trail.len())
    }

    pub fn backtrack(&mut self, level: usize) {
        if self.highest_level() <= level {
            return;
        }
        while self.trail.len() > self.pos_in_trail[level] {
            let bt = self.trail.pop().unwrap();
            self.value.set_none(bt.var());
            self.vsids.push(bt.var());
            self.phase_saving[bt] = Lbool::from(bt.polarity());
        }
        self.propagated = self.pos_in_trail[level];
        self.pos_in_trail.truncate(level);
    }

    pub fn search_with_restart(&mut self, assumption: &[Lit]) -> SatResult<Sat, Unsat> {
        let mut restarts = 0;
        let rest_base = luby(2.0, restarts);
        loop {
            match self.search(assumption, Some(rest_base * 100.0)) {
                Some(true) => return SatResult::Sat(Sat { solver: self }),
                Some(false) => return SatResult::Unsat(Unsat { solver: self }),
                None => {
                    restarts += 1;
                }
            }
        }
    }

    pub fn search(&mut self, assumption: &[Lit], noc: Option<f64>) -> Option<bool> {
        let mut num_conflict = 0.0_f64;
        'ml: loop {
            let conflict = self.propagate();
            if conflict != CREF_NONE {
                num_conflict += 1.0;
                if self.highest_level() == 0 {
                    return Some(false);
                }
                let (learnt, btl) = self.analyze(conflict);
                self.backtrack(btl);
                if learnt.len() == 1 {
                    assert!(btl == 0);
                    self.assign(learnt[0], CREF_NONE);
                } else {
                    let mut kind = ClauseKind::Learnt;
                    for l in learnt.iter() {
                        if let Some(act) = self.constrain_act {
                            if act.var() == l.var() {
                                kind = ClauseKind::Temporary;
                            }
                        }
                    }
                    let learnt_id = self.attach_clause(&learnt, kind);
                    self.cdb.bump(learnt_id);
                    let assign = self.cdb.get(learnt_id)[0];
                    self.assign(assign, learnt_id);
                }
                self.vsids.decay();
                self.cdb.decay();
            } else {
                if let Some(noc) = noc {
                    if num_conflict >= noc {
                        self.backtrack(assumption.len());
                        return None;
                    }
                }
                self.clean_leanrt();
                while self.highest_level() < assumption.len() {
                    let a = assumption[self.highest_level()];
                    match self.value.v(a) {
                        Lbool::TRUE => self.new_level(),
                        Lbool::FALSE => {
                            self.analyze_unsat_core(a);
                            return Some(false);
                        }
                        _ => {
                            self.new_level();
                            self.assign(a, CREF_NONE);
                            continue 'ml;
                        }
                    }
                }
                if !self.decide() {
                    return Some(true);
                }
            }
        }
    }
}

fn luby(y: f64, mut x: u32) -> f64 {
    let mut size = 1;
    let mut seq = 0;
    while size < x + 1 {
        seq += 1;
        size = 2 * size + 1
    }
    while size - 1 != x {
        size = (size - 1) >> 1;
        seq -= 1;
        x %= size;
    }
    y.powi(seq)
}
