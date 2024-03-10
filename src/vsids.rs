use crate::{cdb::CREF_NONE, utils::Lbool, Solver};
use giputils::gvec::Gvec;
use logic_form::{Lit, Var, VarMap};
use std::ops::{Index, MulAssign};

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
    fn len(&self) -> u32 {
        self.heap.len()
    }

    #[inline]
    pub fn clear(&mut self) {
        for v in self.heap.iter() {
            self.pos[*v] = None;
        }
        self.heap.clear();
    }

    #[inline]
    fn up(&mut self, v: Var, activity: &Activity) {
        let mut idx = match self.pos[v] {
            Some(idx) => idx,
            None => return,
        };
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
        self.up(var, activity);
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

pub struct Activity {
    activity: VarMap<f64>,
    act_inc: f64,
    bucket_heap: BinaryHeap,
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
    pub fn reserve(&mut self, var: Var) {
        self.activity.reserve(var);
        self.bucket_heap.reserve(var);
    }

    #[inline]
    fn check(&mut self, var: Var) {
        let activity = self as *const Self;
        if self.bucket_heap.pos[var].is_none() {
            self.bucket_heap.push(var, unsafe { &*activity });
        }
        assert!(self.bucket_heap.pos[var].is_some())
    }

    fn bucket(&self, var: Var) -> u32 {
        match self.bucket_heap.pos[var] {
            Some(b) => u32::BITS - b.leading_zeros(),
            None => u32::BITS - self.bucket_heap.len().leading_zeros() + 1,
        }
    }

    #[inline]
    pub fn bump(&mut self, var: Var) {
        self.activity[var] += self.act_inc;
        self.check(var);
        let activity = self as *const Self;
        self.bucket_heap.up(var, unsafe { &*activity });
        if self.activity[var] > 1e100 {
            self.activity.iter_mut().for_each(|a| a.mul_assign(1e-100));
            self.act_inc *= 1e-100;
        }
    }

    const DECAY: f64 = 0.95;

    #[inline]
    pub fn decay(&mut self) {
        self.act_inc *= 1.0 / Self::DECAY
    }
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            act_inc: 1.0,
            activity: Default::default(),
            bucket_heap: Default::default(),
        }
    }
}

pub struct Vsids {
    pub activity: Activity,

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
            return self.bucket.push(var, &self.activity);
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
            self.heap.up(var, &self.activity);
        }
    }

    #[inline]
    pub fn decay(&mut self) {
        self.activity.decay();
    }
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
    pub fn push(&mut self, var: Var, activity: &Activity) {
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
