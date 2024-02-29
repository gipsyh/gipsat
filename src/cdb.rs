use crate::Solver;
use bitfield_struct::bitfield;
use logic_form::Lit;
use std::{
    mem::take,
    ops::{AddAssign, Index, MulAssign},
    ptr,
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

#[derive(Clone, Copy)]
pub struct Clause {
    data: *mut Data,
}

impl Clause {
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { (*self.data).header.len() }
    }

    #[inline]
    fn is_learnt(&self) -> bool {
        unsafe { (*self.data).header.learnt() }
    }

    #[inline]
    fn get_act(&self) -> f32 {
        assert!(self.is_learnt());
        unsafe { (*self.data.add(self.len() + 1)).act }
    }

    #[inline]
    fn get_mut_act(&mut self) -> &mut f32 {
        assert!(self.is_learnt());
        unsafe { &mut (*self.data.add(self.len() + 1)).act }
    }

    #[inline]
    pub fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            ptr::swap(self.data.add(a + 1), self.data.add(b + 1));
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) {
        let len = self.len();
        unsafe {
            *self.data.add(1 + index) = *self.data.add(len);
            (*self.data).header.set_len(len - 1);
        };
    }
}

impl Index<usize> for Clause {
    type Output = Lit;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*(self.data.add(index + 1) as *const Lit) }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CRef(u32);

pub const CREF_NONE: CRef = CRef(u32::MAX);

impl From<usize> for CRef {
    fn from(value: usize) -> Self {
        Self(value as _)
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
    pub fn get(&self, cref: CRef) -> Clause {
        Clause {
            data: unsafe { self.data.get_unchecked(cref.0 as usize) as *const Data as *mut Data },
        }
    }

    #[inline]
    fn alloc(&mut self, clause: &[Lit], learnt: bool) -> CRef {
        let cid = self.data.len();
        let mut additional = clause.len() + 1;
        if learnt {
            additional += 1;
        }
        self.data.reserve(additional);
        unsafe { self.data.set_len(self.data.len() + additional) };
        self.data[cid].header = Header::new().with_len(clause.len()).with_learnt(learnt);
        for (i, lit) in clause.iter().enumerate() {
            self.data[cid + 1 + i].lit = *lit;
        }
        if learnt {
            self.data[cid + clause.len() + 1].act = 0.0;
        }
        CRef::from(cid)
    }

    fn alloc_from(&mut self, from: &[Data]) -> CRef {
        let cid = self.data.len();
        self.data.reserve(from.len());
        self.data.extend_from_slice(from);
        cid.into()
    }

    pub fn free(&mut self, cref: CRef) {
        let cref = cref.0 as usize;
        let mut len = unsafe { self.data[cref].header.len() } + 1;
        if unsafe { self.data[cref].header.learnt() } {
            len += 1;
        }
        // if self.data.len() == cref + len {
        //     self.data.truncate(cref)
        // } else {
        self.wasted += len
        // }
    }

    pub fn reloc(&mut self, cid: CRef, to: &mut Allocator) -> CRef {
        let cid = cid.0 as usize;
        unsafe {
            if self.data[cid].header.reloced() {
                return CRef(self.data[cid + 1].cid);
            }
            let mut len = self.data[cid].header.len() + 1;
            if self.data[cid].header.learnt() {
                len += 1;
            }
            let rcid = to.alloc_from(&self.data[cid..cid + len]);
            self.data[cid].header.set_reloced(true);
            self.data[cid + 1].cid = rcid.0;
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
    trans: Vec<CRef>,
    lemma: Vec<CRef>,
    learnt: Vec<CRef>,
    temporary: Vec<CRef>,
    act_inc: f32,
}

impl ClauseDB {
    #[inline]
    pub fn get(&self, cref: CRef) -> Clause {
        self.allocator.get(cref)
    }

    #[inline]
    pub fn alloc(&mut self, clause: &[Lit], kind: ClauseKind) -> CRef {
        let cid = self
            .allocator
            .alloc(clause, matches!(kind, ClauseKind::Learnt));
        match kind {
            ClauseKind::Trans => self.trans.push(cid),
            ClauseKind::Lemma => self.lemma.push(cid),
            ClauseKind::Learnt => self.learnt.push(cid),
            ClauseKind::Temporary => self.temporary.push(cid),
        }
        cid
    }

    #[inline]
    pub fn free(&mut self, cref: CRef) {
        self.allocator.free(cref)
    }

    #[inline]
    pub fn bump(&mut self, cref: CRef) {
        let mut cls = self.get(cref);
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

impl Solver {
    #[inline]
    fn clause_satisfied(&self, cls: CRef) -> bool {
        let cls = self.cdb.get(cls);
        for i in 0..cls.len() {
            if self.value.v(cls[i]).is_true() {
                return true;
            }
        }
        false
    }

    pub fn attach_clause(&mut self, clause: &[Lit], kind: ClauseKind) -> CRef {
        assert!(clause.len() > 1);
        let id = self.cdb.alloc(clause, kind);
        self.watchers.attach(id, self.cdb.get(id));
        id
    }

    fn remove_clause(&mut self, cref: CRef) {
        self.watchers.detach(cref, self.cdb.get(cref));
        self.cdb.free(cref);
    }

    pub fn clean_temporary(&mut self) {
        while let Some(t) = self.cdb.temporary.pop() {
            self.remove_clause(t);
        }
    }

    fn locked(&self, cls: Clause) -> bool {
        self.value.v(cls[0]).is_true() && self.reason[cls[0]] != CREF_NONE
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
            let cls = self.cdb.get(l);
            if !self.locked(cls) && cls.len() > 2 {
                self.remove_clause(l);
            } else {
                self.cdb.learnt.push(l);
            }
        }
    }

    fn simplify_clauses(&mut self, mut clauses: Vec<CRef>) -> Vec<CRef> {
        let mut i: usize = 0;
        while i < clauses.len() {
            let cid = clauses[i];
            if self.clause_satisfied(cid) {
                clauses.swap_remove(i);
                self.remove_clause(cid);
                continue;
            }
            let mut j = 2;
            let mut cls = self.cdb.get(cid);
            while j < cls.len() {
                if self.value.v(cls[j]).is_false() {
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
                if self.reason[*l] != CREF_NONE {
                    self.reason[*l] = self.cdb.allocator.reloc(self.reason[*l], &mut to)
                }
            }

            self.cdb.allocator = to;
        }
    }
}
