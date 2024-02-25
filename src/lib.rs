mod analyze;
mod cdb;
mod domain;
mod others;
mod propagate;
mod search;
mod simplify;
mod statistic;
mod ts;
mod utils;
mod vsids;

use analyze::Analyze;
use cdb::{ClauseDB, ClauseKind};
use domain::Domain;
use logic_form::{Clause, Cube, Lit, LitMap, LitSet, Var, VarMap};
use propagate::Watchers;
use statistic::Statistic;
use std::fmt::{self, Debug};
use ts::TransitionSystem;
use vsids::Vsids;

#[derive(Default)]
pub struct Solver {
    name: String,
    cdb: ClauseDB,
    watchers: Watchers,
    value: LitMap<Option<bool>>,
    trail: Vec<Lit>,
    pos_in_trail: Vec<usize>,
    level: VarMap<usize>,
    reason: VarMap<Option<usize>>,
    propagated: usize,
    vsids: Vsids,
    phase_saving: VarMap<Option<Lit>>,
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
        let size = self.level.len() + 1;
        self.value.push(None);
        self.value.push(None);
        self.level.push(0);
        self.reason.push(None);
        self.watchers.reserve(size);
        let res = Var::new(self.level.len() - 1);
        self.vsids.new_var();
        self.phase_saving.push(None);
        self.analyze.new_var();
        self.unsat_core.new_var();
        self.domain.reserve(res);
        res
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
            match self.value[*l] {
                Some(true) => return None,
                Some(false) => (),
                None => clause.push(*l),
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
            self.domain.global.mark(l.var());
            if let Some(act) = self.constrain_act {
                if act.var() == l.var() {
                    kind = ClauseKind::Temporary;
                }
            }
        }
        if clause.len() == 1 {
            assert!(!matches!(kind, ClauseKind::Temporary));
            match self.value[clause[0]] {
                None => {
                    self.assign(clause[0], None);
                    assert!(self.propagate().is_none());
                }
                _ => todo!(),
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
            self.domain.lemma.mark(l.var());
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
                self.value[bt] = None;
                self.value[!bt] = None;
                self.phase_saving[bt] = Some(bt);
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
                assert!(self.domain.local[self.constrain_act.unwrap()] != self.domain.local_stamp);
                self.domain.local[self.constrain_act.unwrap()] = self.domain.local_stamp;
                self.domain
                    .local_marks
                    .push(self.constrain_act.unwrap().var());
            }
            self.vsids.clear();
            for d in self.domain.domains() {
                if self.value[d.lit()].is_none() {
                    self.vsids.push(*d);
                }
            }
        }
    }

    pub fn solve(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        self.new_round(None::<std::option::IntoIter<Var>>);
        self.statistic.num_solve += 1;
        if self.statistic.num_solve % 1000 == 1 {
            self.clean_leanrt();
        }
        if self.statistic.num_solve % 1000 == 1 {
            self.simplify();
        }
        if self.search(assumption) {
            SatResult::Sat(Model { solver: self })
        } else {
            SatResult::Unsat(Conflict { solver: self })
        }
    }

    pub fn solve_with_domain(
        &mut self,
        assumption: &[Lit],
        domain: impl Iterator<Item = Var>,
    ) -> SatResult<'_> {
        self.new_round(Some(domain));
        self.statistic.num_solve += 1;
        if self.statistic.num_solve % 1000 == 1 {
            self.clean_leanrt();
        }
        if self.statistic.num_solve % 1000 == 1 {
            self.simplify();
        }
        if self.search(assumption) {
            SatResult::Sat(Model { solver: self })
        } else {
            SatResult::Unsat(Conflict { solver: self })
        }
    }

    pub fn solve_with_constrain(
        &mut self,
        assump: &[Lit],
        mut constrain: Clause,
        domain: bool,
    ) -> SatResult<'_> {
        if self.constrain_act.is_none() {
            let constrain_act = self.new_var();
            self.constrain_act = Some(constrain_act.lit());
            self.domain.global.mark(constrain_act);
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
        }
        if self.statistic.num_solve % 1000 == 1 {
            self.simplify();
        }
        if self.search(&assumption) {
            SatResult::Sat(Model { solver: self })
        } else {
            SatResult::Unsat(Conflict { solver: self })
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
                self.value[bt] = None;
                self.value[!bt] = None;
                self.phase_saving[bt] = Some(bt);
            }
            self.propagated = self.pos_in_trail[0];
            self.pos_in_trail.truncate(0);
        }
        self.vsids.clear();
        for d in self.domain.domains() {
            if self.value[d.lit()].is_none() {
                self.vsids.push(*d);
            }
        }
    }

    pub fn unset_domain(&mut self) {
        self.temporary_domain = false;
    }

    /// # Safety
    /// unsafe get sat model
    pub unsafe fn get_model(&self) -> Model<'static> {
        let solver = unsafe { &*(self as *const Self) };
        Model { solver }
    }

    /// # Safety
    /// unsafe get unsat core
    pub unsafe fn get_conflict(&self) -> Conflict<'static> {
        let solver = unsafe { &*(self as *const Self) };
        Conflict { solver }
    }
}

pub struct Model<'a> {
    solver: &'a Solver,
}

impl Model<'_> {
    pub fn lit_value(&self, lit: Lit) -> Option<bool> {
        self.solver.value[lit]
    }
}

pub struct Conflict<'a> {
    solver: &'a Solver,
}

impl Conflict<'_> {
    pub fn has(&self, lit: Lit) -> bool {
        self.solver.unsat_core.has(lit)
    }
}

pub enum SatResult<'a> {
    Sat(Model<'a>),
    Unsat(Conflict<'a>),
}

impl Debug for SatResult<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sat(_) => "Sat".fmt(f),
            Self::Unsat(_) => "Unsat".fmt(f),
        }
    }
}
