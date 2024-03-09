use crate::{cdb::CREF_NONE, utils::Lbool, Solver};
use giputils::gvec::Gvec;
use logic_form::{Lit, Var, VarMap};
use std::{
    ops::{Index, MulAssign},
    rc::Rc,
};

#[derive(Default)]
struct BinaryHeap {
    heap: Gvec<Var>,
    pos: VarMap<Option<u32>>,
}

impl BinaryHeap {
    #[inline]
    pub fn reserve(&mut self, var: Var) {
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

pub struct Activity {
    activity: VarMap<f64>,
    act_inc: f64,
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
        self.activity.reserve(var)
    }

    #[inline]
    pub fn bump(&mut self, var: Var) {
        self.activity[var] += self.act_inc;
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
            activity: Default::default(),
            act_inc: 1.0,
        }
    }
}

#[derive(Default)]
pub struct Vsids {
    pub activity: Rc<Activity>,

    heap: BinaryHeap,
    pub bucket: Bucket,
    pub fast: bool,
}

impl Vsids {
    #[inline]
    pub fn reserve(&mut self, var: Var) {
        self.heap.reserve(var);
        self.bucket.reserve(var);
        unsafe { Rc::get_mut_unchecked(&mut self.activity).reserve(var) };
    }

    #[inline]
    pub fn push(&mut self, var: Var) {
        if self.fast {
            return self.bucket.push(var);
        }
        self.heap.push(var, &self.activity)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Var> {
        if self.fast {
            return self.bucket.pop();
        }
        self.heap.pop(&self.activity)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.heap.clear();
    }

    #[inline]
    pub fn bump(&mut self, var: Var) {
        unsafe { Rc::get_mut_unchecked(&mut self.activity) }.bump(var);
        if let Some(pos) = self.heap.pos[var] {
            self.heap.up(pos, &self.activity)
        }
    }

    #[inline]
    pub fn decay(&mut self) {
        unsafe { Rc::get_mut_unchecked(&mut self.activity) }.decay();
    }

    pub fn enable_fast(&mut self, mut vars: Vec<Var>) {
        assert!(!self.fast);
        self.heap.clear();
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
    buckets: Gvec<Gvec<Var>>,
    var_bucket: VarMap<u32>,
    in_bucket: VarMap<bool>,
    head: u32,
}

impl Bucket {
    pub fn create(&mut self, vars: Vec<Var>) {
        self.clear();
        let num_bucket = 20;
        self.buckets.reserve(num_bucket);
        let bicket_len = vars.len() as u32 / num_bucket + 1;
        self.head = 0;
        for (i, var) in vars.into_iter().enumerate() {
            let bucket = i as u32 / bicket_len;
            self.var_bucket[var] = bucket;
            self.buckets[bucket].push(var);
            assert!(!self.in_bucket[var]);
            self.in_bucket[var] = true;
        }
    }

    #[inline]
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
