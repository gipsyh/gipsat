use crate::{cdb::CREF_NONE, utils::Lbool, Solver};
use logic_form::{Lit, Var, VarMap};
use std::ops::MulAssign;

pub struct Vsids {
    activity: VarMap<f64>,
    heap: Vec<Var>,
    pos: VarMap<Option<usize>>,
    act_inc: f64,

    pub bucket: Bucket,
    pub fast: bool,
}

impl Default for Vsids {
    fn default() -> Self {
        Self {
            activity: Default::default(),
            heap: Default::default(),
            pos: Default::default(),
            act_inc: 1.0,
            bucket: Default::default(),
            fast: false,
        }
    }
}

impl Vsids {
    #[inline]
    pub fn reserve(&mut self, var: Var) {
        self.pos.reserve(var);
        self.bucket.reserve(var);
        self.activity.reserve(var);
    }

    #[inline]
    pub fn clear(&mut self) {
        for v in self.heap.iter() {
            self.pos[*v] = None;
        }
        self.heap.clear();
    }

    #[inline]
    fn up(&mut self, mut idx: usize) {
        let v = self.heap[idx];
        while idx != 0 {
            let pidx = (idx - 1) >> 1;
            if self.activity[self.heap[pidx]] >= self.activity[v] {
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
    fn down(&mut self, mut idx: usize) {
        let v = self.heap[idx];
        loop {
            let left = (idx << 1) + 1;
            if left >= self.heap.len() {
                break;
            }
            let right = left + 1;
            let child = if right < self.heap.len()
                && self.activity[self.heap[right]] > self.activity[self.heap[left]]
            {
                right
            } else {
                left
            };
            if self.activity[v] >= self.activity[self.heap[child]] {
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
    pub fn push(&mut self, var: Var) {
        if self.fast {
            return self.bucket.push(var);
        }
        if self.pos[var].is_some() {
            return;
        }
        let idx = self.heap.len();
        self.heap.push(var);
        self.pos[var] = Some(idx);
        self.up(idx);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Var> {
        if self.fast {
            return self.bucket.pop();
        }
        if self.heap.is_empty() {
            return None;
        }
        let value = self.heap[0];
        self.heap[0] = self.heap[self.heap.len() - 1];
        self.pos[self.heap[0]] = Some(0);
        self.pos[value] = None;
        self.heap.pop();
        if self.heap.len() > 1 {
            self.down(0);
        }
        Some(value)
    }

    #[inline]
    pub fn bump(&mut self, var: Var) {
        self.activity[var] += self.act_inc;
        if self.activity[var] > 1e100 {
            self.activity.iter_mut().for_each(|a| a.mul_assign(1e-100));
            self.act_inc *= 1e-100;
        }
        if let Some(pos) = self.pos[var] {
            self.up(pos)
        }
    }

    const DECAY: f64 = 0.95;

    #[inline]
    pub fn decay(&mut self) {
        self.act_inc *= 1.0 / Self::DECAY
    }

    pub fn enable_fast(&mut self, mut vars: Vec<Var>) {
        assert!(!self.fast);
        self.clear();
        vars.sort_unstable_by(|a, b| self.activity[*b].partial_cmp(&self.activity[*a]).unwrap());
        self.fast = true;
        self.bucket.create(vars);
    }

    pub fn disable_fast(&mut self) {
        self.fast = false;
        self.bucket.clear();
    }
}

#[derive(Default)]
pub struct Bucket {
    buckets: Vec<Vec<Var>>,
    var_bucket: VarMap<usize>,
    in_bucket: VarMap<bool>,
    head: usize,
}

impl Bucket {
    pub fn create(&mut self, vars: Vec<Var>) {
        self.clear();
        let num_bucket = 20;
        self.buckets.resize_with(num_bucket, Default::default);
        let bicket_len = vars.len() / num_bucket + 1;
        self.head = 0;
        for (i, var) in vars.into_iter().enumerate() {
            let bucket = i / bicket_len;
            self.var_bucket[var] = bucket;
            self.buckets[bucket].push(var);
            assert!(!self.in_bucket[var]);
            self.in_bucket[var] = true;
        }
    }

    pub fn reserve(&mut self, var: Var) {
        self.var_bucket.reserve(var);
        self.in_bucket.reserve(var);
    }

    #[inline]
    pub fn push(&mut self, var: Var) {
        if self.in_bucket[var] {
            return;
        }
        let bucket = self.var_bucket[var];
        if self.head > bucket {
            self.head = bucket;
        }
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
