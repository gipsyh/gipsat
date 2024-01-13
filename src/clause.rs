use crate::{propagate::Watcher, Solver};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum ClauseKind {
    Origin,
    Learnt,
    Temporary,
    TemporaryLearnt,
    Removed,
}

#[derive(Debug)]
pub struct Clause {
    clause: logic_form::Clause,
    kind: ClauseKind,
}

impl Clause {
    pub fn new(clause: logic_form::Clause, kind: ClauseKind) -> Self {
        Self { clause, kind }
    }

    // #[inline]
    // pub fn is_leanrt(&self) -> bool {
    //     matches!(self, ClauseKind::Learnt |)
    // }

    #[inline]
    pub fn is_temporary(&self) -> bool {
        matches!(
            self.kind,
            ClauseKind::Temporary | ClauseKind::TemporaryLearnt
        )
    }

    #[inline]
    pub fn set_kind(&mut self, kind: ClauseKind) {
        self.kind = kind
    }

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

pub struct LbdQueue {
    queue: [usize; 50],
    full: bool,
    pos: usize,
    fast_sum: usize,
    slow_sum: usize,
}

impl LbdQueue {
    pub fn restart(&self, conflicts: usize) -> bool {
        self.full && 0.8 * self.fast_sum as f32 / 50.0 > self.slow_sum as f32 / conflicts as f32
    }

    pub fn push(&mut self, lbd: usize) {
        if self.full {
            self.fast_sum -= self.queue[self.pos];
        } else if self.pos == 49 {
            self.full = true;
        }
        self.fast_sum += lbd;
        self.queue[self.pos] = lbd;
        self.pos += 1;
        if self.pos == 50 {
            self.pos = 0;
        }
        self.pos = (self.pos + 1) / 50;
        self.slow_sum += lbd.min(50);
    }
}

impl Default for LbdQueue {
    fn default() -> Self {
        Self {
            queue: [0; 50],
            full: false,
            pos: 0,
            fast_sum: 0,
            slow_sum: 0,
        }
    }
}

#[derive(Default, Debug)]
pub struct ClauseDB {
    clauses: Vec<Clause>,
    origin: Vec<usize>,
    learnt: Vec<usize>,
    temporary: Vec<usize>,
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
            ClauseKind::Temporary => self.clauses.temporary.push(id),
            ClauseKind::TemporaryLearnt => {
                self.clauses.learnt.push(id);
                self.clauses.temporary.push(id);
            }
            _ => todo!(),
        }
        self.clauses.clauses.push(clause);
        id
    }

    fn remove_clause(&mut self, cidx: usize) {
        let cref = &mut self.clauses[cidx];
        cref.kind = ClauseKind::Removed;
        self.watchers.remove(!cref[0], cidx);
        self.watchers.remove(!cref[1], cidx);
        cref.clause = Default::default();
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

    pub fn remove_temporay(&mut self) {
        while let Some(tmp) = self.clauses.temporary.pop() {
            self.remove_clause(tmp);
        }
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
