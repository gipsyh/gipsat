use crate::{propagate::Watcher, Solver};
use bitfield_struct::bitfield;
use logic_form::Lit;
use std::{
    mem::{take, transmute},
    ops::{Index, IndexMut},
};

#[bitfield(u32)]
struct Header {
    learnt: bool,
    remove: bool,
    #[bits(30)]
    size: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
union Data {
    header: Header,
    lit: Lit,
    act: f32,
}

struct Allocator {
    data: Vec<Data>,
    wasted: usize,
}

impl Allocator {
    #[inline]
    fn alloc(&mut self, clause: &[Lit], learnt: bool) -> usize {
        let use_extra = learnt;
        let cid = self.data.len();
        let additional = clause.len() + 2;
        self.data.reserve(additional);
        unsafe { self.data.set_len(self.data.len() + additional) };
        self.data[cid].header = Header::new().with_learnt(learnt).with_size(clause.len());
        for i in 0..clause.len() {
            self.data[cid + 1 + i].lit = clause[i];
        }
        cid
    }
}

impl Default for Allocator {
    fn default() -> Self {
        let data = Vec::with_capacity(1024 * 1024);
        Self { data, wasted: 0 }
    }
}

pub struct ClauseDB {
    allocator: Allocator,
    origin: Vec<usize>,
    learnt: Vec<usize>,
    act_inc: f32,
}

impl ClauseDB {
    #[inline]
    pub fn alloc(&mut self, clause: &[Lit], learnt: bool) -> usize {
        let id = self.allocator.alloc(clause, learnt);
        if learnt {
            self.learnt.push(id);
        } else {
            self.origin.push(id);
        }
        id
    }

    #[inline]
    pub fn num_learnt(&self) -> usize {
        self.learnt.len()
    }

    // #[inline]
    // pub fn bump(&mut self, cid: usize) {
    //     if !self.clauses[cid].is_leanrt() {
    //         return;
    //     }
    //     self.clauses[cid].activity += self.act_inc;
    //     if self.clauses[cid].activity > 1e20 {
    //         for l in self.learnt.iter() {
    //             if self.clauses[*l].is_valid() {
    //                 self.clauses[*l].activity.mul_assign(1e-20);
    //             }
    //         }
    //         self.act_inc *= 1e-20;
    //     }
    // }

    const DECAY: f32 = 0.999;

    #[inline]
    pub fn decay(&mut self) {
        self.act_inc *= 1.0 / Self::DECAY
    }
}

impl Default for ClauseDB {
    fn default() -> Self {
        Self {
            allocator: Default::default(),
            origin: Default::default(),
            learnt: Default::default(),
            act_inc: 1.0,
        }
    }
}

impl Index<usize> for ClauseDB {
    type Output = [Lit];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let len = unsafe { self.allocator.data[index].header.size() };
        unsafe { transmute(&self.allocator.data[index + 1..index + 1 + len]) }
    }
}

impl IndexMut<usize> for ClauseDB {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = unsafe { self.allocator.data[index].header.size() };
        unsafe { transmute(&mut self.allocator.data[index + 1..index + 1 + len]) }
    }
}

impl Solver {
    #[inline]
    pub fn clause_satisfied(&self, cls: usize) -> bool {
        for l in self.cdb[cls].iter() {
            if let Some(true) = self.value[*l] {
                return true;
            }
        }
        false
    }

    pub fn attach_clause(&mut self, clause: &[Lit], learnt: bool) -> usize {
        assert!(clause.len() > 1);
        let id = self.cdb.alloc(clause, learnt);
        self.watchers[!clause[0]].push(Watcher::new(id, clause[1]));
        self.watchers[!clause[1]].push(Watcher::new(id, clause[0]));
        id
    }

    fn remove_clause(&mut self, cidx: usize) {
        // let clause = take(&mut self.cdb[cidx]);
        // self.watchers.remove(!clause[0], cidx);
        // self.watchers.remove(!clause[1], cidx);
        todo!()
    }

    // fn locked(&self, cls: &Clause) -> bool {
    //     matches!(self.value[cls[0]], Some(true)) && self.reason[cls[0]].is_some()
    // }

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

    // fn simplify_clauses(&mut self, mut cls: Vec<usize>) -> Vec<usize> {
    //     let mut i: usize = 0;
    //     while i < cls.len() {
    //         let cid = cls[i];
    //         if self.clause_satisfied(cid) {
    //             cls[i] = *cls.last().unwrap();
    //             cls.pop();
    //             self.remove_clause(cid);
    //             continue;
    //         }
    //         let cls = &mut self.cdb[cid];
    //         assert!(self.value[cls[0]].is_none() && self.value[cls[1]].is_none());
    //         let mut j = 2;
    //         while j < cls.len() {
    //             if let Some(false) = self.value[cls[j]] {
    //                 cls[j] = *cls.last().unwrap();
    //                 cls.pop();
    //                 continue;
    //             }
    //             j += 1;
    //         }
    //         i += 1;
    //     }
    //     cls
    // }

    pub fn clausedb_simplify_satisfied(&mut self) {
        // assert!(self.highest_level() == 0);
        // assert!(self.propagate().is_none());
        // // dbg!("====");
        // // dbg!(self.clauses.learnt.len());
        // // dbg!(self.clauses.origin.len());
        // // dbg!("----");
        // let leant = take(&mut self.cdb.learnt);
        // self.cdb.learnt = self.simplify_clauses(leant);
        // let origin = take(&mut self.cdb.origin);
        // self.cdb.origin = self.simplify_clauses(origin);
        // dbg!(self.clauses.learnt.len());
        // dbg!(self.clauses.origin.len());
    }
}
