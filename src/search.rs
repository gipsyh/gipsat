use crate::{
    cdb::{CRef, ClauseKind, CREF_NONE},
    Solver,
};
use logic_form::{Lit, Var, VarMap};

#[derive(Default)]
pub struct Value {
    data: VarMap<Option<bool>>,
}

impl Value {
    pub fn new_var(&mut self) {
        self.data.push(None);
    }

    #[inline]
    pub fn v(&self, lit: Lit) -> Option<bool> {
        self.data[lit].map(|v| v ^ !lit.polarity())
    }

    #[inline]
    pub fn set(&mut self, lit: Lit) {
        self.data[lit] = Some(lit.polarity())
    }

    #[inline]
    pub fn set_none(&mut self, var: Var) {
        self.data[var] = None
    }
}

// impl Index<Lit> for Value {
//     type Output = Option<bool>;

//     #[inline]
//     fn index(&self, index: Lit) -> &Self::Output {
//         &self.data[index]
//     }
// }

// impl IndexMut<Lit> for Value {

//     #[inline]
//     fn index_mut(&mut self, index: Lit) -> &mut Self::Output {
//         &mut self.data[index]
//     }
// }

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
        if self.highest_level() == level {
            return;
        }
        while self.trail.len() > self.pos_in_trail[level] {
            let bt = self.trail.pop().unwrap();
            self.value.set_none(bt.var());
            self.vsids.push(bt.var());
            self.phase_saving[bt] = Some(bt);
        }
        self.propagated = self.pos_in_trail[level];
        self.pos_in_trail.truncate(level);
    }

    pub fn search(&mut self, assumption: &[Lit]) -> bool {
        'ml: loop {
            if let Some(conflict) = self.propagate() {
                if self.highest_level() == 0 {
                    return false;
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
                    self.assign(self.cdb[learnt_id][0], learnt_id);
                }
                self.vsids.decay();
                self.cdb.decay();
            } else {
                while self.highest_level() < assumption.len() {
                    let a = assumption[self.highest_level()];
                    match self.value.v(a) {
                        Some(true) => self.new_level(),
                        Some(false) => {
                            self.analyze_unsat_core(a);
                            return false;
                        }
                        None => {
                            self.new_level();
                            self.assign(a, CREF_NONE);
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
