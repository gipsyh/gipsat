use crate::{propagate::Watcher, Solver};
use std::{
    mem::take,
    ops::{Deref, DerefMut, MulAssign},
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
    activity: f32,
}

impl Clause {
    pub fn new(clause: logic_form::Clause, kind: ClauseKind) -> Self {
        Self {
            clause,
            kind,
            activity: 0_f32,
        }
    }

    #[inline]
    pub fn is_leanrt(&self) -> bool {
        matches!(self.kind, ClauseKind::Learnt)
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        !matches!(self.kind, ClauseKind::Removed)
    }
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

#[derive(Debug)]
pub struct ClauseDB {
    clauses: Vec<Clause>,
    origin: Vec<usize>,
    learnt: Vec<usize>,
    act_inc: f32,
}

impl ClauseDB {
    pub fn new_var(&mut self) {}

    #[inline]
    pub fn num_learnt(&self) -> usize {
        self.learnt.len()
    }

    #[inline]
    pub fn bump(&mut self, cid: usize) {
        if !self.clauses[cid].is_leanrt() {
            return;
        }
        self.clauses[cid].activity += self.act_inc;
        if self.clauses[cid].activity > 1e20 {
            for l in self.learnt.iter() {
                if self.clauses[*l].is_valid() {
                    self.clauses[*l].activity.mul_assign(1e-20);
                }
            }
            self.act_inc *= 1e-20;
        }
    }

    const DECAY: f32 = 0.999;

    #[inline]
    pub fn decay(&mut self) {
        self.act_inc *= 1.0 / Self::DECAY
    }
}

impl Default for ClauseDB {
    fn default() -> Self {
        Self {
            clauses: Default::default(),
            origin: Default::default(),
            learnt: Default::default(),
            act_inc: 1.0,
        }
    }
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

    fn locked(&self, cls: &Clause) -> bool {
        matches!(self.value[cls[0]], Some(true)) && self.reason[cls[0]].is_some()
    }

    pub fn reduce(&mut self) {
        // if self.clauses.learnt.len() < self.trail.len() {
        //     return;
        // }
        // if self.clauses.learnt.len() - self.trail.len() < 10000 {
        //     return;
        // }
        // dbg!(self.clauses.learnt.len());
        // let limit = self.clauses.act_inc / self.clauses.learnt.len() as f32;
        // for l in take(&mut self.clauses.learnt) {
        //     let cls = &self.clauses[l];
        //     if !self.locked(cls) && cls.len() > 2 && cls.activity < limit {
        //         self.remove_clause(l);
        //     } else {
        //         self.clauses.learnt.push(l);
        //     }
        // }
        // dbg!(self.clauses.learnt.len());
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
