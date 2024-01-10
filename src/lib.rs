mod analyze;
mod basic;
mod command;
mod others;
mod propagate;
mod search;
mod simplify;
#[cfg(test)]
mod tests;
mod utils;
mod verify;
mod vsids;

use analyze::Analyze;
pub use command::Args;

use basic::Clause;
use logic_form::{Lit, Var};
use propagate::Watcher;
use std::fmt::{self, Debug};
use utils::{LitMap, VarMap};
use vsids::Vsids;

#[derive(Default)]
pub struct Solver {
    args: Args,
    value: LitMap<Option<bool>>,
    trail: Vec<Lit>,
    pos_in_trail: Vec<usize>,
    level: VarMap<usize>,
    propagated: usize,
    watchers: LitMap<Vec<Watcher>>,
    clauses: Vec<Clause>,
    reason: VarMap<Option<usize>>,
    vsids: Vsids,
    phase_saving: VarMap<Option<Lit>>,
    analyze: Analyze,
    reduces: usize,
    reduce_limit: usize,
}

impl Solver {
    pub fn new(args: Args) -> Self {
        Self {
            args,
            reduce_limit: 8192,
            ..Default::default()
        }
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
        self.vsids.push(res);
        self.phase_saving.push(None);
        self.analyze.new_var();
        res
    }

    #[inline]
    pub fn num_var(&self) -> usize {
        self.reason.len()
    }

    pub fn add_clause(&mut self, clause: &[Lit]) {
        assert!(self.highest_level() == 0);
        let clause = Clause::from(clause);
        for l in clause.iter() {
            while self.num_var() <= l.var().into() {
                self.new_var();
            }
        }
        if clause.len() == 1 {
            assert!(!matches!(self.value[clause[0]], Some(false)));
            self.assign(clause[0], None);
        } else {
            self.add_clause_inner(clause);
        }
    }

    pub fn solve(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        self.search(assumption)
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
        todo!()
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
