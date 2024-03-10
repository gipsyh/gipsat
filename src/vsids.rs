use crate::{cdb::CREF_NONE, utils::Lbool, Solver};
use giputils::gvec::Gvec;
use logic_form::{Lit, Var, VarMap};
use std::{
    collections::BTreeSet,
    fmt::Debug,
    mem::take,
    ops::{Index, MulAssign},
    rc::Rc,
};

#[derive(Default)]
pub struct BinaryHeap {
    heap: Gvec<Var>,
    pos: VarMap<Option<u32>>,
}

impl BinaryHeap {
    #[inline]
    fn reserve(&mut self, var: Var) {
        self.pos.reserve(var);
    }

    #[inline]
    pub fn clear(&mut self) {
        for v in self.heap.iter() {
            self.pos[*v] = None;
        }
        self.heap.clear();
    }

    #[inline]
    fn up(&mut self, mut idx: u32, activity: &Activity) {
        let v = self.heap[idx];
        while idx != 0 {
            let pidx = (idx - 1) >> 1;
            if activity[self.heap[pidx]] >= activity[v] {
                break;
            }
            self.heap[idx] = self.heap[pidx];
            self.pos[self.heap[idx]] = Some(idx);
            idx = pidx;
        }
        self.heap[idx] = v;
        self.pos[v] = Some(idx);
    }

    #[inline]
    fn down(&mut self, mut idx: u32, activity: &Activity) {
        let v = self.heap[idx];
        loop {
            let left = (idx << 1) + 1;
            if left >= self.heap.len() {
                break;
            }
            let right = left + 1;
            let child = if right < self.heap.len()
                && activity[self.heap[right]] > activity[self.heap[left]]
            {
                right
            } else {
                left
            };
            if activity[v] >= activity[self.heap[child]] {
                break;
            }
            self.heap[idx] = self.heap[child];
            self.pos[self.heap[idx]] = Some(idx);
            idx = child;
        }
        self.heap[idx] = v;
        self.pos[v] = Some(idx);
    }

    #[inline]
    pub fn push(&mut self, var: Var, activity: &Activity) {
        if self.pos[var].is_some() {
            return;
        }
        let idx = self.heap.len();
        self.heap.push(var);
        self.pos[var] = Some(idx);
        self.up(idx, activity);
    }

    #[inline]
    pub fn pop(&mut self, activity: &Activity) -> Option<Var> {
        if self.heap.is_empty() {
            return None;
        }
        let value = self.heap[0];
        self.heap[0] = self.heap[self.heap.len() - 1];
        self.pos[self.heap[0]] = Some(0);
        self.pos[value] = None;
        self.heap.pop();
        if self.heap.len() > 1 {
            self.down(0, activity);
        }
        Some(value)
    }
}

struct BucketElement {
    act: Rc<Activity>,
    var: Var,
}

impl PartialEq for BucketElement {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.act[self.var] == self.act[other.var] && self.var == other.var
    }
}

impl Eq for BucketElement {}

impl PartialOrd for BucketElement {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.act[self.var] == self.act[other.var] {
            self.var.partial_cmp(&other.var)
        } else {
            self.act[self.var].partial_cmp(&self.act[other.var])
        }
    }
}

impl Ord for BucketElement {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Debug for BucketElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BucketElement")
            .field("act", &self.act[self.var])
            .field("var", &self.var)
            .finish()
    }
}

const NUM_BUCKET: u32 = 20;

pub struct Activity {
    activity: VarMap<f64>,
    act_inc: f64,
    bucket_heap: Gvec<BTreeSet<BucketElement>>,
    bucket: VarMap<Option<u32>>,
}

impl Index<Var> for Activity {
    type Output = f64;

    #[inline]
    fn index(&self, index: Var) -> &Self::Output {
        &self.activity[index]
    }
}

impl Activity {
    #[inline]
    pub fn reserve(self: &mut Rc<Self>, var: Var) {
        let s = unsafe { Rc::get_mut_unchecked(self) };
        s.activity.reserve(var);
        s.bucket.reserve(var);
    }

    #[inline]
    fn check(self: &mut Rc<Self>, var: Var) {
        if self.bucket[var].is_none() {
            let clone = self.clone();
            let s = unsafe { Rc::get_mut_unchecked(self) };
            s.bucket_heap[NUM_BUCKET - 1].insert(BucketElement { act: clone, var });
            s.bucket[var] = Some(NUM_BUCKET - 1);
            s.update();
        }
        assert!(self.bucket[var].is_some());
    }

    fn bucket(&self, var: Var) -> u32 {
        match self.bucket[var] {
            Some(b) => b,
            None => NUM_BUCKET,
        }
    }

    #[inline]
    pub fn bump(self: &mut Rc<Self>, var: Var) {
        self.check(var);
        let clone = self.clone();
        let s = unsafe { Rc::get_mut_unchecked(self) };
        let bucket = s.bucket[var].unwrap();
        let element = BucketElement { act: clone, var };
        if !s.bucket_heap[bucket].remove(&element) {
            dbg!(s.bucket_heap[bucket].iter().any(|b| b.var == var));
            todo!();
        }
        s.activity[var] += s.act_inc;
        s.bucket_heap[bucket].insert(element);
        s.up(var);
        if s.activity[var] > 1e100 {
            s.activity.iter_mut().for_each(|a| a.mul_assign(1e-100));
            s.act_inc *= 1e-100;
            for i in 0..NUM_BUCKET {
                let heap = take(&mut s.bucket_heap[i]);
                for e in heap {
                    s.bucket_heap[i].insert(e);
                }
            }
        }
    }

    const DECAY: f64 = 0.95;

    #[inline]
    pub fn decay(&mut self) {
        self.act_inc *= 1.0 / Self::DECAY
    }

    #[inline]
    fn up(&mut self, var: Var) {
        let mut now = self.bucket[var].unwrap();
        while now > 0 {
            if self.bucket_heap[now].last().unwrap() > self.bucket_heap[now - 1].first().unwrap() {
                let max = self.bucket_heap[now].pop_last().unwrap();
                let min = self.bucket_heap[now - 1].pop_first().unwrap();
                self.bucket[max.var] = Some(now - 1);
                self.bucket[min.var] = Some(now);
                self.bucket_heap[now].insert(min);
                self.bucket_heap[now - 1].insert(max);
            } else {
                break;
            }
            now -= 1;
        }
    }

    #[inline]
    fn update(&mut self) {
        let mut now = NUM_BUCKET - 1;
        while now > 0 {
            if self.bucket_heap[now].len() > self.bucket_heap[now - 1].len() {
                assert!(self.bucket_heap[now].len() == self.bucket_heap[now - 1].len() + 1);
                let max = self.bucket_heap[now].pop_last().unwrap();
                self.bucket[max.var] = Some(now - 1);
                self.bucket_heap[now - 1].insert(max);
            } else {
                break;
            }
            now -= 1;
        }
    }

    pub fn dbg(&self) {
        dbg!("begin");
        for i in 0..NUM_BUCKET {
            dbg!(self.bucket_heap[i].last());
            dbg!(self.bucket_heap[i].first());
            dbg!(self.bucket_heap[i].len());
        }
        dbg!("end");
    }
}

impl Default for Activity {
    fn default() -> Self {
        let mut bucket_heap = Gvec::default();
        for _ in 0..NUM_BUCKET {
            bucket_heap.push(BTreeSet::new());
        }
        Self {
            activity: Default::default(),
            act_inc: 1.0,
            bucket_heap,
            bucket: Default::default(),
        }
    }
}

pub struct Vsids {
    pub activity: Rc<Activity>,

    pub heap: BinaryHeap,
    pub bucket: Bucket,
    pub enable_bucket: bool,
}

impl Vsids {
    #[inline]
    pub fn reserve(&mut self, var: Var) {
        self.heap.reserve(var);
        self.bucket.reserve(var);
        self.activity.reserve(var);
    }

    #[inline]
    pub fn push(&mut self, var: Var) {
        if self.enable_bucket {
            return self.bucket.push(var, &mut self.activity);
        }
        self.heap.push(var, &self.activity)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Var> {
        if self.enable_bucket {
            return self.bucket.pop();
        }
        self.heap.pop(&self.activity)
    }

    #[inline]
    pub fn bump(&mut self, var: Var) {
        self.activity.bump(var);
        if !self.enable_bucket {
            if let Some(pos) = self.heap.pos[var] {
                self.heap.up(pos, &self.activity)
            }
        }
    }

    #[inline]
    pub fn decay(&mut self) {
        unsafe { Rc::get_mut_unchecked(&mut self.activity) }.decay();
    }

    // pub fn enable_fast(&mut self) {
    //     assert!(!self.fast);
    //     self.fast = true;
    //     self.heap.clear();
    //     self.bucket.clear();
    // }

    // pub fn disable_fast(&mut self) {
    //     self.fast = false;
    //     self.bucket.clear();
    //     self.heap.clear();
    // }
}

impl Default for Vsids {
    fn default() -> Self {
        Self {
            activity: Default::default(),
            heap: Default::default(),
            bucket: Default::default(),
            enable_bucket: true,
        }
    }
}

#[derive(Default)]
pub struct Bucket {
    buckets: Gvec<Gvec<Var>>,
    in_bucket: VarMap<bool>,
    head: u32,
}

impl Bucket {
    #[inline]
    pub fn reserve(&mut self, var: Var) {
        self.in_bucket.reserve(var);
    }

    #[inline]
    pub fn push(&mut self, var: Var, activity: &mut Rc<Activity>) {
        if self.in_bucket[var] {
            return;
        }
        let bucket = activity.bucket(var);
        if self.head > bucket {
            self.head = bucket;
        }
        self.buckets.reserve(bucket + 1);
        self.buckets[bucket].push(var);
        self.in_bucket[var] = true;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Var> {
        while self.head < self.buckets.len() {
            if !self.buckets[self.head].is_empty() {
                let var = self.buckets[self.head].pop().unwrap();
                self.in_bucket[var] = false;
                return Some(var);
            }
            self.head += 1;
        }
        None
    }

    pub fn clear(&mut self) {
        while self.head < self.buckets.len() {
            while let Some(var) = self.buckets[self.head].pop() {
                self.in_bucket[var] = false;
            }
            self.head += 1;
        }
        self.buckets.clear();
        self.head = 0;
    }
}

impl Solver {
    #[inline]
    pub fn decide(&mut self) -> bool {
        while let Some(decide) = self.vsids.pop() {
            if self.value.v(decide.lit()).is_none() {
                let decide = Lit::new(decide, self.phase_saving[decide] != Lbool::FALSE);
                self.pos_in_trail.push(self.trail.len());
                self.assign(decide, CREF_NONE);
                return true;
            }
        }
        false
    }
}
