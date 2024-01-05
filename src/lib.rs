mod analyze;
mod propagate;
mod search;
mod utils;

use logic_form::{Clause, Lit, Var};
use propagate::Watcher;
use std::collections::BinaryHeap;
use utils::{LitMap, VarMap};

#[derive(Debug, Default)]
pub struct Solver {
    value: LitMap<Option<bool>>,
    trail: Vec<Lit>,
    pos_in_trail: Vec<usize>,
    level: VarMap<usize>,
    propagated: usize,
    watchers: LitMap<Vec<Watcher>>,
    clauses: Vec<Clause>,
    reason: VarMap<Option<usize>>,
    vsids: BinaryHeap<Var>,

    seen: LitMap<bool>,
}

impl Solver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_var(&mut self) -> Var {
        self.value.push(None);
        self.value.push(None);
        self.level.push(0);
        self.reason.push(None);
        self.watchers.push(Vec::new());
        self.watchers.push(Vec::new());
        let res = Var::new(self.level.len() - 1);
        self.vsids.push(res);
        res
    }

    pub fn add_clause(&mut self, clause: &[Lit]) -> usize {
        assert!(clause.len() > 1);
        self.clauses.push(Clause::from(clause));
        let id = self.clauses.len() - 1;
        self.watchers[!clause[0]].push(Watcher::new(id, clause[1]));
        self.watchers[!clause[1]].push(Watcher::new(id, clause[0]));
        self.clauses.len() - 1
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut solver = Solver::new();
        let lit0: Lit = solver.new_var().into();
        let lit1: Lit = solver.new_var().into();
        let lit2: Lit = solver.new_var().into();
        solver.add_clause(&Clause::from([lit0, !lit2]));
        solver.add_clause(&Clause::from([lit1, !lit2]));
        solver.add_clause(&Clause::from([!lit0, !lit1, lit2]));
        match solver.solve(&[]) {
            SatResult::Sat(sat) => {
                dbg!(sat.lit_value(lit0));
                dbg!(sat.lit_value(lit1));
                dbg!(sat.lit_value(lit2));
            }
            SatResult::Unsat(_) => todo!(),
        }
    }
}
