mod analyze;
mod cdb;
mod domain;
mod propagate;
mod search;
mod simplify;
mod statistic;
mod ts;
mod utils;
mod vsids;

use crate::utils::Lbool;
use analyze::Analyze;
use cdb::{CRef, ClauseDB, ClauseKind, CREF_NONE};
use domain::Domain;
use giputils::gvec::Gvec;
use logic_form::{Clause, Cube, Lit, LitSet, Var, VarMap};
use propagate::Watchers;
use satif::{SatResult, SatifSat, SatifUnsat};
use search::Value;
use statistic::Statistic;
use ts::TransitionSystem;
use vsids::Vsids;

#[derive(Default)]
pub struct Solver {
    name: String,
    cdb: ClauseDB,
    watchers: Watchers,
    value: Value,
    trail: Gvec<Lit>,
    pos_in_trail: Vec<u32>,
    level: VarMap<u32>,
    reason: VarMap<CRef>,
    propagated: u32,
    vsids: Vsids,
    phase_saving: VarMap<Lbool>,
    analyze: Analyze,
    unsat_core: LitSet,

    domain: Domain,
    temporary_domain: bool,
    lazy_clauses: Vec<Clause>,
    lazy_lemma: Vec<Clause>,
    lazy_temporary: Vec<Clause>,

    ts: Option<TransitionSystem>,

    statistic: Statistic,

    constrain_act: Option<Lit>,
}

impl Solver {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn set_ts(&mut self, num_var: usize, cnf: &[Clause], dep: &VarMap<Vec<Var>>) {
        while self.num_var() < num_var {
            self.new_var();
        }
        for cls in cnf.iter() {
            self.add_clause_inner(cls, ClauseKind::Trans);
        }
        self.ts = Some(TransitionSystem::new(dep.clone()))
    }

    pub fn new_var(&mut self) -> Var {
        let var = Var::new(self.num_var());
        self.value.reserve(var);
        self.level.reserve(var);
        self.reason.reserve(var);
        self.watchers.reserve(var);
        self.vsids.reserve(var);
        self.phase_saving.reserve(var);
        self.analyze.reserve(var);
        self.unsat_core.reserve(var);
        self.domain.reserve(var);
        var
    }

    #[inline]
    pub fn num_var(&self) -> usize {
        self.reason.len()
    }

    fn simplify_clause(&mut self, cls: &[Lit]) -> Option<logic_form::Clause> {
        assert!(self.highest_level() == 0);
        let mut clause = logic_form::Clause::new();
        for l in cls.iter() {
            while self.num_var() <= l.var().into() {
                self.new_var();
            }
            match self.value.v(*l) {
                Lbool::TRUE => return None,
                Lbool::FALSE => (),
                _ => clause.push(*l),
            }
        }
        assert!(!clause.is_empty());
        Some(clause)
    }

    pub fn add_clause_inner(&mut self, clause: &[Lit], mut kind: ClauseKind) {
        let clause = match self.simplify_clause(clause) {
            Some(clause) => clause,
            None => return,
        };
        for l in clause.iter() {
            self.domain.global.insert(l.var());
            if let Some(act) = self.constrain_act {
                if act.var() == l.var() {
                    kind = ClauseKind::Temporary;
                }
            }
        }
        if clause.len() == 1 {
            assert!(!matches!(kind, ClauseKind::Temporary));
            match self.value.v(clause[0]) {
                Lbool::TRUE | Lbool::FALSE => todo!(),
                _ => {
                    self.assign(clause[0], CREF_NONE);
                    assert!(self.propagate().is_none());
                }
            }
        } else {
            self.attach_clause(&clause, kind);
        }
    }

    pub fn add_clause_direct(&mut self, clause: &[Lit]) {
        self.add_clause_inner(clause, ClauseKind::Trans);
    }

    pub fn add_clause(&mut self, clause: &[Lit]) {
        self.lazy_clauses.push(Clause::from(clause));
    }

    pub fn add_lemma(&mut self, lemma: &[Lit]) {
        for l in lemma.iter() {
            self.domain.lemma.insert(l.var());
        }
        self.lazy_lemma.push(Clause::from(lemma));
    }

    fn new_round(&mut self, domain: Option<impl Iterator<Item = Var>>) {
        if !self.temporary_domain {
            self.domain.disable_local();
        }
        self.clean_temporary();
        if !self.pos_in_trail.is_empty() {
            while self.trail.len() > self.pos_in_trail[0] {
                let bt = self.trail.pop().unwrap();
                self.value.set_none(bt.var());
                self.phase_saving[bt] = Lbool::from(bt.polarity());
                if self.temporary_domain {
                    self.vsids.push(bt.var());
                }
            }
            self.propagated = self.pos_in_trail[0];
            self.pos_in_trail.truncate(0);
        }
        // dbg!(&self.name);
        // dbg!(self.num_var());
        // dbg!(self.trail.len());
        // dbg!(self.cdb.num_learnt());
        // dbg!(self.cdb.num_origin());

        while let Some(lc) = self.lazy_clauses.pop() {
            self.add_clause_inner(&lc, ClauseKind::Trans);
        }

        while let Some(lc) = self.lazy_lemma.pop() {
            self.add_clause_inner(&lc, ClauseKind::Lemma);
        }

        while let Some(lc) = self.lazy_temporary.pop() {
            self.add_clause_inner(&lc, ClauseKind::Temporary);
        }

        if !self.temporary_domain {
            if let Some(domain) = domain {
                self.domain.enable_local(domain, self.ts.as_ref().unwrap());
                if self.constrain_act.is_some() {
                    assert!(
                        self.domain.local[self.constrain_act.unwrap()] != self.domain.local_stamp
                    );
                    self.domain.local[self.constrain_act.unwrap()] = self.domain.local_stamp;
                    self.domain
                        .local_marks
                        .push(self.constrain_act.unwrap().var());
                }
            }
            self.vsids.clear();
            for d in self.domain.domains() {
                if self.value.v(d.lit()).is_none() {
                    self.vsids.push(*d);
                }
            }
        }
    }

    pub fn solve(&mut self, assumption: &[Lit]) -> SatResult<Sat, Unsat> {
        self.new_round(None::<std::option::IntoIter<Var>>);
        self.statistic.num_solve += 1;
        if self.statistic.num_solve % 1000 == 1 {
            self.clean_leanrt();
            self.simplify();
        }
        self.garbage_collect();
        if self.search(assumption) {
            SatResult::Sat(Sat { solver: self })
        } else {
            SatResult::Unsat(Unsat { solver: self })
        }
    }

    pub fn solve_with_domain(&mut self, assumption: &[Lit], domain: bool) -> SatResult<Sat, Unsat> {
        if domain {
            self.new_round(Some(assumption.iter().map(|l| l.var())));
        } else {
            self.new_round(None::<std::option::IntoIter<Var>>);
        };
        self.statistic.num_solve += 1;
        if self.statistic.num_solve % 1000 == 1 {
            self.clean_leanrt();
            self.simplify();
        }
        self.garbage_collect();
        if self.search(assumption) {
            SatResult::Sat(Sat { solver: self })
        } else {
            SatResult::Unsat(Unsat { solver: self })
        }
    }

    pub fn solve_with_constrain(
        &mut self,
        assump: &[Lit],
        mut constrain: Clause,
        domain: bool,
    ) -> SatResult<Sat, Unsat> {
        if self.constrain_act.is_none() {
            let constrain_act = self.new_var();
            self.constrain_act = Some(constrain_act.lit());
            self.domain.global.insert(constrain_act);
        }
        let act = self.constrain_act.unwrap();
        let mut assumption = Cube::new();
        assumption.extend_from_slice(assump);
        assumption.push(act);
        let cc = constrain.clone();
        constrain.push(!act);
        self.lazy_temporary.push(constrain);
        if domain {
            self.new_round(Some(assump.iter().chain(cc.iter()).map(|l| l.var())));
        } else {
            self.new_round(None::<std::option::IntoIter<Var>>);
        };
        self.statistic.num_solve += 1;
        if self.statistic.num_solve % 1000 == 1 {
            self.clean_leanrt();
            self.simplify();
        }
        self.garbage_collect();
        if self.search(&assumption) {
            SatResult::Sat(Sat { solver: self })
        } else {
            SatResult::Unsat(Unsat { solver: self })
        }
    }

    pub fn set_domain(&mut self, domain: impl Iterator<Item = Lit>) {
        self.domain
            .enable_local(domain.map(|l| l.var()), self.ts.as_ref().unwrap());
        assert!(self.domain.local[self.constrain_act.unwrap()] != self.domain.local_stamp);
        self.domain.local[self.constrain_act.unwrap()] = self.domain.local_stamp;
        self.domain
            .local_marks
            .push(self.constrain_act.unwrap().var());
        self.temporary_domain = true;
        self.clean_temporary();
        if !self.pos_in_trail.is_empty() {
            while self.trail.len() > self.pos_in_trail[0] {
                let bt = self.trail.pop().unwrap();
                self.value.set_none(bt.var());
                self.phase_saving[bt] = Lbool::from(bt.polarity());
            }
            self.propagated = self.pos_in_trail[0];
            self.pos_in_trail.truncate(0);
        }
        self.vsids
            .enable_fast(self.domain.domains().copied().collect());
    }

    pub fn set_sub_domain(&mut self, domain: impl Iterator<Item = Lit>) {
        self.domain
            .enable_local(domain.map(|l| l.var()), self.ts.as_ref().unwrap());
        assert!(self.domain.local[self.constrain_act.unwrap()] != self.domain.local_stamp);
        self.domain.local[self.constrain_act.unwrap()] = self.domain.local_stamp;
        self.domain
            .local_marks
            .push(self.constrain_act.unwrap().var());
        self.temporary_domain = true;
        self.clean_temporary();

        if !self.pos_in_trail.is_empty() {
            while self.trail.len() > self.pos_in_trail[0] {
                let bt = self.trail.pop().unwrap();
                self.value.set_none(bt.var());
                self.phase_saving[bt] = Lbool::from(bt.polarity());
            }
            self.propagated = self.pos_in_trail[0];
            self.pos_in_trail.truncate(0);
        }
        assert!(self.vsids.fast);
        self.vsids.bucket.clear();
        for v in self.domain.domains() {
            self.vsids.push(*v);
        }
    }

    pub fn unset_domain(&mut self) {
        self.temporary_domain = false;
        self.vsids.disable_fast();
    }

    // /// # Safety
    // /// unsafe get sat model
    // pub unsafe fn get_model(&self) -> Model<'static> {
    //     let solver = unsafe { &*(self as *const Self) };
    //     Sat { solver }
    // }

    // /// # Safety
    // /// unsafe get unsat core
    // pub unsafe fn get_conflict(&self) -> Conflict<'static> {
    //     let solver = unsafe { &*(self as *const Self) };
    //     Conflict { solver }
    // }
}

pub struct Sat {
    solver: *mut Solver,
}

impl SatifSat for Sat {
    fn lit_value(&self, lit: Lit) -> Option<bool> {
        let solver = unsafe { &*self.solver };
        match solver.value.v(lit) {
            Lbool::TRUE => Some(true),
            Lbool::FALSE => Some(false),
            _ => None,
        }
    }
}

pub struct Unsat {
    solver: *mut Solver,
}

impl SatifUnsat for Unsat {
    fn has(&self, lit: Lit) -> bool {
        let solver = unsafe { &*self.solver };
        solver.unsat_core.has(lit)
    }
}
