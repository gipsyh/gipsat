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
mod vsids;

pub use command::Args;

use analyze::Analyze;
use clause::{ClauseDB, ClauseKind, LbdQueue};
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

    pub fn add_clause(&mut self, clause: &[Lit]) {
        self.reset();
        let clause = match self.simplify_clause(clause) {
            Some(clause) => clause,
            None => return,
        };
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

    pub fn add_lazy_clause(&mut self, clause: &[Lit]) {
        self.lazy_clauses.push(Clause::from(clause));
    }

    pub fn reset(&mut self) {
        self.backtrack(0);
        self.remove_temporay();
    }

    pub fn solve(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        self.reset();
        while let Some(lc) = self.lazy_clauses.pop() {
            self.add_clause(&lc);
        }
        if self.search(assumption) {
            SatResult::Sat(Model { solver: self })
        } else {
            SatResult::Unsat(Conflict { solver: self })
        }
    }

    pub fn solve_with_constrain(&mut self, assumption: &[Lit], constrain: &[Lit]) -> SatResult<'_> {
        self.reset();
        let mut assumption = Clause::from(assumption);
        if let Some(clause) = self.simplify_clause(constrain) {
            if clause.len() == 1 {
                // assumption.insert(0, clause[0]);
                assumption.push(clause[0]);
            } else {
                self.attach_clause(clause::Clause::new(
                    Clause::from(constrain),
                    ClauseKind::Temporary,
                ));
            }
        }
        assert!(self.lazy_clauses.is_empty());
        if self.search(&assumption) {
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
