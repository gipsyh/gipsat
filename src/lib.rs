mod analyze;
mod clause;
mod command;
mod domain;
mod others;
mod propagate;
mod search;
mod simplify;
mod statistic;
#[cfg(test)]
mod tests;
mod ts;
mod utils;
mod vsids;

pub use command::Args;

use analyze::Analyze;
use clause::{ClauseDB, ClauseKind};
use domain::Domain;
use logic_form::{Clause, Lit, LitMap, LitSet, Var, VarMap};
use propagate::Watchers;
use statistic::Statistic;
use std::fmt::{self, Debug};
use ts::TransitionSystem;
use vsids::Vsids;

#[derive(Default)]
pub struct Solver {
    args: Args,
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
    lazy_clauses: Vec<Clause>,

    ts: Option<TransitionSystem>,

    statistic: Statistic,
}

impl Solver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_args(&mut self, args: Args) {
        self.args = args
    }

    pub fn set_ts(&mut self, dep: VarMap<Vec<Var>>) {
        self.ts = Some(TransitionSystem::new(dep))
    }

    pub fn new_var(&mut self) -> Var {
        self.value.push(None);
        self.value.push(None);
        self.level.push(0);
        self.reason.push(None);
        self.watchers.push(Vec::new());
        self.watchers.push(Vec::new());
        let res = Var::new(self.level.len() - 1);
        self.vsids.new_var();
        self.phase_saving.push(None);
        self.analyze.new_var();
        self.unsat_core.new_var();
        self.cdb.new_var();
        self.domain.new_var();
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

    pub fn add_clause_inner(&mut self, clause: &[Lit]) {
        let clause = match self.simplify_clause(clause) {
            Some(clause) => clause,
            None => return,
        };
        for l in clause.iter() {
            self.domain.global.mark(l.var());
            self.vsids.push(l.var());
        }
        if clause.len() == 1 {
            match self.value[clause[0]] {
                None => self.assign(clause[0], None),
                _ => todo!(),
            }
        } else {
            let clause = clause::Clause::new(clause, ClauseKind::Origin);
            self.attach_clause(clause);
        }
    }

    pub fn add_clause(&mut self, clause: &[Lit]) {
        self.lazy_clauses.push(Clause::from(clause));
    }

    fn new_round(&mut self, domain: Option<impl Iterator<Item = Var>>) {
        self.domain.disable_local();
        // if !self.pos_in_trail.is_empty() {
        //     while self.trail.len() > self.pos_in_trail[0] {
        //         let bt = self.trail.pop().unwrap();
        //         self.value[bt] = None;
        //         self.value[!bt] = None;
        //         self.phase_saving[bt] = Some(bt);
        //     }
        //     self.propagated = self.pos_in_trail[0];
        //     self.pos_in_trail.truncate(0);
        // }
        self.backtrack(0);

        while let Some(lc) = self.lazy_clauses.pop() {
            self.add_clause_inner(&lc);
        }

        // if let Some(domain) = domain {
        //     self.domain.enable_local(domain, self.ts.as_ref().unwrap());
        // }

        // self.vsids.clear();
        // for d in self.domain.domains() {
        //     if self.value[d.lit()].is_none() {
        //         self.vsids.push(*d);
        //     }
        // }
    }

    pub fn solve(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        self.new_round(None::<std::option::IntoIter<Var>>);
        self.statistic.num_solve += 1;
        if self.statistic.num_solve % 100 == 0 {
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
        if self.statistic.num_solve % 100 == 0 {
            self.simplify();
        }
        if self.search(assumption) {
            SatResult::Sat(Model { solver: self })
        } else {
            SatResult::Unsat(Conflict { solver: self })
        }
    }

    pub fn solve_with_constrain(&mut self, assumption: &[Lit], constrain: &[Lit]) -> SatResult<'_> {
        todo!();
        // self.reset();
        // let mut assumption = Clause::from(assumption);
        // if let Some(clause) = self.simplify_clause(constrain) {
        //     if clause.len() == 1 {
        //         assumption.push(clause[0]);
        //     } else {
        //         todo!();
        //         let mut constrain = Clause::from(constrain);
        //         self.attach_clause(clause::Clause::new(constrain, ClauseKind::Learnt));
        //     }
        // }
        // assert!(self.lazy_clauses.is_empty());
        // if self.search(&assumption) {
        //     SatResult::Sat(Model { solver: self })
        // } else {
        //     SatResult::Unsat(Conflict { solver: self })
        // }
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
