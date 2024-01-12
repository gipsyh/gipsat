mod analyze;
mod clause;
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

pub use command::Args;

use analyze::Analyze;
use clause::{ClauseDB, LbdQueue};
use logic_form::{Clause, Lit, Var};
use propagate::Watchers;
use std::fmt::{self, Debug};
use utils::{LitMap, LitSet, Rand, VarMap};
use vsids::Vsids;

#[derive(Default)]
pub struct Solver {
    args: Args,
    clauses: ClauseDB,
    watchers: Watchers,
    value: LitMap<Option<bool>>,
    trail: Vec<Lit>,
    pos_in_trail: Vec<usize>,
    level: VarMap<usize>,
    reason: VarMap<Option<usize>>,
    propagated: usize,
    vsids: Vsids,
    phase_saving: VarMap<Option<Lit>>,
    lbd_queue: LbdQueue,
    analyze: Analyze,
    rand: Rand,
    reduces: usize,
    reduce_limit: usize,
    unsat_core: LitSet,

    lazy_clauses: Vec<Clause>,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            reduce_limit: 8192,
            ..Default::default()
        }
    }

    pub fn set_args(&mut self, args: Args) {
        self.args = args
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
        self.unsat_core.new_var();
        res
    }

    #[inline]
    pub fn num_var(&self) -> usize {
        self.reason.len()
    }

    pub fn add_clause(&mut self, clause: &[Lit]) {
        if self.highest_level() > 0 {
            self.backtrack(0);
        }
        let clause = Clause::from(clause);
        for l in clause.iter() {
            while self.num_var() <= l.var().into() {
                self.new_var();
            }
        }
        if clause.len() == 1 {
            match self.value[clause[0]] {
                Some(true) => (),
                Some(false) => panic!(),
                None => self.assign(clause[0], None),
            }
        } else {
            self.add_origin_clause(clause);
        }
    }

    pub fn add_lazy_clause(&mut self, clause: &[Lit]) {
        self.lazy_clauses.push(Clause::from(clause));
    }

    pub fn solve(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        self.backtrack(0);
        while let Some(lc) = self.lazy_clauses.pop() {
            self.add_clause(&lc);
        }
        if self.search(assumption) {
            SatResult::Sat(Model { solver: self })
        } else {
            SatResult::Unsat(Conflict { solver: self })
        }
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
    pub fn lit_value(&self, lit: Lit) -> bool {
        self.solver.value[lit].unwrap()
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
