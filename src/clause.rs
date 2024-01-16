use crate::{propagate::Watcher, Solver};
use std::{
    mem::take,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Default)]
pub enum ClauseKind {
    Origin,
    Learnt,
    #[default]
    Removed,
}

#[derive(Debug, Default)]
pub struct Clause {
    clause: logic_form::Clause,
    kind: ClauseKind,
    activity: f64,
}

impl Clause {
    pub fn new(clause: logic_form::Clause, kind: ClauseKind) -> Self {
        Self {
            clause,
            kind,
            activity: 0_f64,
        }
    }

    // #[inline]
    // pub fn is_leanrt(&self) -> bool {
    //     matches!(self, ClauseKind::Learnt |)
    // }

    // #[inline]
    // pub fn is_valid(&self) -> bool {
    //     !self.kind.contains(ClauseKind::Removed)
    // }
}

impl Deref for Clause {
    type Target = logic_form::Clause;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.clause
    }
}

impl DerefMut for Clause {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.clause
    }
}

#[derive(Default, Debug)]
pub struct ClauseDB {
    clauses: Vec<Clause>,
    origin: Vec<usize>,
    learnt: Vec<usize>,
}

impl ClauseDB {
    pub fn new_var(&mut self) {}
}

impl Deref for ClauseDB {
    type Target = [Clause];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.clauses
    }
}

impl DerefMut for ClauseDB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.clauses
    }
}

impl Solver {
    pub fn satisfied(&self, cls: usize) -> bool {
        for l in self.clauses[cls].iter() {
            if let Some(true) = self.value[*l] {
                return true;
            }
        }
        false
    }

    pub fn attach_clause(&mut self, clause: Clause) -> usize {
        assert!(clause.len() > 1);
        let id = self.clauses.len();
        self.watchers[!clause[0]].push(Watcher::new(id, clause[1]));
        self.watchers[!clause[1]].push(Watcher::new(id, clause[0]));
        match clause.kind {
            ClauseKind::Origin => self.clauses.origin.push(id),
            ClauseKind::Learnt => self.clauses.learnt.push(id),
            _ => todo!(),
        }
        self.clauses.clauses.push(clause);
        id
    }

    fn remove_clause(&mut self, cidx: usize) {
        let clause = take(&mut self.clauses[cidx]);
        self.watchers.remove(!clause[0], cidx);
        self.watchers.remove(!clause[1], cidx);
    }

    pub fn reduce(&mut self) {
        // self.backtrack(0);
        // self.reduces = 0;
        // self.reduce_limit += 512;
        // for l in take(&mut self.clauses.learnt) {
        //     if self.clauses[l].lbd >= 5 && self.rand.rand_bool() {
        //         self.remove_clause(l);
        //     } else {
        //         self.clauses.learnt.push(l);
        //     }
        // }
        todo!()
    }

    pub fn verify(&mut self) -> bool {
        // for i in 0..self.clauses.len() {
        //     if !self.clauses[i].removed()
        //         && !self.clauses[i]
        //             .iter()
        //             .any(|l| matches!(self.value[*l], Some(true)))
        //     {
        //         return false;
        //     }
        // }
        // true
        todo!()
    }
}
