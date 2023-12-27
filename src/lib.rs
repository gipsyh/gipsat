mod decision;
mod propagate;
mod utils;

use logic_form::{Clause, Lit, Var};
use propagate::Watcher;
use utils::{LitMap, VarMap};

#[derive(Debug, Default)]
pub struct Solver {
    value: LitMap<Option<bool>>,
    trail: Vec<Lit>,
    level: VarMap<usize>,
    propagated: usize,
    watchers: LitMap<Vec<Watcher>>,
    clauses: Vec<Clause>,
}

impl Solver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_var(&mut self) -> Var {
        self.value.push(None);
        self.value.push(None);
        self.level.push(0);
        self.watchers.push(Vec::new());
        self.watchers.push(Vec::new());
        Var::new(self.level.len() - 1)
    }

    pub fn add_clause(&mut self, clause: &[Lit]) {
        assert!(clause.len() > 1);
        self.clauses.push(Clause::from(clause));
        let id = self.clauses.len() - 1;
        self.watchers[!clause[0]].push(Watcher::new(id, clause[1]));
        self.watchers[!clause[1]].push(Watcher::new(id, clause[0]));
    }

    pub fn solve(&mut self, assumption: &[Lit]) -> SatResult<'_> {
        SatResult::Sat(Model { solver: self })
    }
}

pub struct Model<'a> {
    solver: &'a Solver,
}

impl Model<'_> {
    pub fn lit_value(&self, lit: Lit) -> bool {
        todo!()
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
        solver.assign(lit2);
        solver.propagate();
        dbg!(solver.value);
    }
}
