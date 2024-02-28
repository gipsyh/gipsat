use crate::Solver;
use bitfield_struct::bitfield;
use logic_form::Lit;
use std::{
    mem::{take, transmute},
    ops::{AddAssign, Index, IndexMut, MulAssign},
};

#[bitfield(u32)]
struct Header {
    learnt: bool,
    reloced: bool,
    #[bits(30)]
    len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
union Data {
    header: Header,
    lit: Lit,
    act: f32,
    cid: u32,
}

struct Clause {
    data: &'static mut [Data],
}

impl Clause {
    #[inline]
    fn len(&self) -> usize {
        unsafe { self.data[0].header.len() }
    }

    #[inline]
    fn is_learnt(&self) -> bool {
        unsafe { self.data[0].header.learnt() }
    }

    #[inline]
    fn get_act(&self) -> f32 {
        unsafe { self.data[self.len() + 1].act }
    }

    #[inline]
    fn get_mut_act(&mut self) -> &mut f32 {
        unsafe { &mut self.data[self.len() + 1].act }
    }

    fn swap_remove(&mut self, index: usize) {
        let len = self.len();
        self.data[1 + index] = self.data[len];
        unsafe {
            self.data[0].header.set_len(len - 1);
        };
    }
}

impl Index<usize> for Clause {
    type Output = Lit;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { transmute(&self.data[index + 1]) }
    }
}

struct Allocator {
    data: Vec<Data>,
    wasted: usize,
}

impl Allocator {
    fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1024 * 1024);
        let data = Vec::with_capacity(capacity);
        Self { data, wasted: 0 }
    }

    #[inline]
    fn len(&mut self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn get(&mut self, cid: usize) -> Clause {
        let len = unsafe { self.data[cid].header.len() };
        let data: &'static mut [Data] = unsafe { transmute(&mut self.data[cid..cid + 2 + len]) };
        Clause { data }
    }

    #[inline]
    fn alloc(&mut self, clause: &[Lit]) -> usize {
        let cid = self.data.len();
        let additional = clause.len() + 2;
        self.data.reserve(additional);
        unsafe { self.data.set_len(self.data.len() + additional) };
        self.data[cid].header = Header::new().with_len(clause.len());
        for (i, lit) in clause.iter().enumerate() {
            self.data[cid + 1 + i].lit = *lit;
        }
        self.data[cid + clause.len() + 1].act = 0.0;
        cid
    }

    fn alloc_from(&mut self, from: &[Data]) -> usize {
        let cid = self.data.len();
        self.data.reserve(from.len());
        self.data.extend_from_slice(from);
        cid
    }

    pub fn free(&mut self, cid: usize) {
        self.wasted += unsafe { self.data[cid].header.len() } + 2;
    }

    pub fn reloc(&mut self, cid: usize, to: &mut Allocator) -> usize {
        unsafe {
            if self.data[cid].header.reloced() {
                return self.data[cid + 1].cid as usize;
            }
            let len = self.data[cid].header.len() + 2;
            let rcid = to.alloc_from(&self.data[cid..cid + len]);
            self.data[cid].header.set_reloced(true);
            self.data[cid + 1].cid = rcid as u32;
            rcid
        }
    }
}

impl Default for Allocator {
    fn default() -> Self {
        let data = Vec::with_capacity(1024 * 1024);
        Self { data, wasted: 0 }
    }
}

pub enum ClauseKind {
    Trans,
    Lemma,
    Learnt,
    Temporary,
}

pub struct ClauseDB {
    allocator: Allocator,
    trans: Vec<usize>,
    lemma: Vec<usize>,
    learnt: Vec<usize>,
    temporary: Vec<usize>,
    act_inc: f32,
}

impl ClauseDB {
    #[inline]
    fn get(&mut self, cid: usize) -> Clause {
        self.allocator.get(cid)
    }

    #[inline]
    pub fn alloc(&mut self, clause: &[Lit], kind: ClauseKind) -> usize {
        let cid = self.allocator.alloc(clause);
        match kind {
            ClauseKind::Trans => self.trans.push(cid),
            ClauseKind::Lemma => self.lemma.push(cid),
            ClauseKind::Learnt => self.learnt.push(cid),
            ClauseKind::Temporary => self.temporary.push(cid),
        }
        cid
    }

    #[inline]
    pub fn free(&mut self, cid: usize) {
        self.allocator.free(cid)
    }

    #[inline]
    pub fn bump(&mut self, cid: usize) {
        let mut cls = self.get(cid);
        if !cls.is_learnt() {
            return;
        }
        cls.get_mut_act().add_assign(self.act_inc);
        if cls.get_act() > 1e20 {
            for i in 0..self.learnt.len() {
                let l = self.learnt[i];
                let mut cls = self.get(l);
                cls.get_mut_act().mul_assign(1e-20);
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
            allocator: Default::default(),
            lemma: Default::default(),
            trans: Default::default(),
            learnt: Default::default(),
            temporary: Default::default(),
            act_inc: 1.0,
        }
    }
}

impl Index<usize> for ClauseDB {
    type Output = [Lit];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let len = unsafe { self.allocator.data[index].header.len() };
        unsafe { transmute(&self.allocator.data[index + 1..index + 1 + len]) }
    }
}

impl IndexMut<usize> for ClauseDB {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = unsafe { self.allocator.data[index].header.len() };
        unsafe { transmute(&mut self.allocator.data[index + 1..index + 1 + len]) }
    }
}

impl Solver {
    fn clause_satisfied(&self, cls: usize) -> bool {
        for l in self.cdb[cls].iter() {
            if let Some(true) = self.value[*l] {
                return true;
            }
        }
        false
    }

    pub fn attach_clause(&mut self, clause: &[Lit], kind: ClauseKind) -> usize {
        assert!(clause.len() > 1);
        let id = self.cdb.alloc(clause, kind);
        self.watchers.attach(id, &self.cdb[id]);
        id
    }

    fn remove_clause(&mut self, cref: usize) {
        self.watchers.detach(cref, &self.cdb[cref]);
        self.cdb.free(cref);
    }

    pub fn clean_temporary(&mut self) {
        while let Some(t) = self.cdb.temporary.pop() {
            self.remove_clause(t);
        }
    }

    fn locked(&self, cls: &[Lit]) -> bool {
        matches!(self.value[cls[0]], Some(true)) && self.reason[cls[0]].is_some()
    }

    pub fn clean_leanrt(&mut self) {
        // assert!(self.highest_level() == 0);
        // if self.cdb.learnt.len() * 4 < self.cdb.trans.len() {
        //     return;
        // }
        // self.cdb.learnt.sort_unstable_by(|a, b| {
        //     self.cdb
        //         .allocator
        //         .get(*b)
        //         .get_act()
        //         .partial_cmp(&self.cdb.allocator.get(*a).get_act())
        //         .unwrap()
        // });
        // let learnt = take(&mut self.cdb.learnt);
        // for i in 0..learnt.len() {
        //     let l = learnt[i];
        //     if i > learnt.len() / 2 {
        //         let cls = &self.cdb[l];
        //         if !self.locked(cls) && cls.len() > 2 {
        //             self.remove_clause(l);
        //             continue;
        //         }
        //     }
        //     self.cdb.learnt.push(l);
        // }

        assert!(self.highest_level() == 0);
        for l in take(&mut self.cdb.learnt) {
            let cls = &self.cdb[l];
            if !self.locked(cls) && cls.len() > 2 {
                self.remove_clause(l);
            } else {
                self.cdb.learnt.push(l);
            }
        }
    }

    fn simplify_clauses(&mut self, mut clauses: Vec<usize>) -> Vec<usize> {
        let mut i: usize = 0;
        while i < clauses.len() {
            let cid = clauses[i];
            if self.clause_satisfied(cid) {
                clauses[i] = *clauses.last().unwrap();
                clauses.pop();
                self.remove_clause(cid);
                continue;
            }
            let mut j = 2;
            let mut cls = self.cdb.get(cid);
            while j < cls.len() {
                if let Some(false) = self.value[cls[j]] {
                    cls.swap_remove(j);
                    continue;
                }
                j += 1;
            }
            i += 1;
        }
        clauses
    }

    pub fn clausedb_simplify_satisfied(&mut self) {
        assert!(self.highest_level() == 0);
        assert!(self.propagate().is_none());
        let learnt = take(&mut self.cdb.learnt);
        self.cdb.learnt = self.simplify_clauses(learnt);
        let origin = take(&mut self.cdb.trans);
        self.cdb.trans = self.simplify_clauses(origin);
        let origin = take(&mut self.cdb.lemma);
        self.cdb.lemma = self.simplify_clauses(origin);
        self.garbage_collect();
    }

    pub fn garbage_collect(&mut self) {
        if self.cdb.allocator.wasted * 3 > self.cdb.allocator.len() {
            let mut to =
                Allocator::with_capacity(self.cdb.allocator.len() - self.cdb.allocator.wasted);

            for ws in self.watchers.wtrs.iter_mut() {
                for w in ws.iter_mut() {
                    w.clause = self.cdb.allocator.reloc(w.clause, &mut to);
                }
            }

            let cls = self
                .cdb
                .trans
                .iter_mut()
                .chain(self.cdb.lemma.iter_mut())
                .chain(self.cdb.learnt.iter_mut())
                .chain(self.cdb.temporary.iter_mut());

            for c in cls {
                *c = self.cdb.allocator.reloc(*c, &mut to)
            }

            for l in self.trail.iter() {
                if let Some(r) = self.reason[*l].as_mut() {
                    *r = self.cdb.allocator.reloc(*r, &mut to)
                }
            }

            self.cdb.allocator = to;
        }
    }
}
