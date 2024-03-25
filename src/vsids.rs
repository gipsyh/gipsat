use crate::{cdb::CREF_NONE, utils::Lbool, Solver};
use giputils::{gvec::Gvec, OptionU32, OptionU8};
use logic_form::{Cube, Lit, Var, VarMap};
use rand::Rng;
use std::{
    mem::swap,
    ops::{Index, MulAssign},
};

#[derive(Default)]
pub struct BinaryHeap {
    heap: Gvec<Var>,
    pos: VarMap<OptionU32>,
}

impl BinaryHeap {
    #[inline]
    fn reserve(&mut self, var: Var) {
        self.pos.reserve(var);
    }

    #[inline]
    pub fn clear(&mut self) {
        for v in self.heap.iter() {
            self.pos[*v] = OptionU32::NONE;
        }
        self.heap.clear();
    }

    #[inline]
    fn up(&mut self, v: Var, activity: &Activity) {
        let mut idx = match self.pos[v] {
            OptionU32::NONE => return,
            idx => *idx,
        };
        while idx != 0 {
            let pidx = (idx - 1) >> 1;
            if activity[self.heap[pidx]] >= activity[v] {
                break;
            }
            self.heap[idx] = self.heap[pidx];
            *self.pos[self.heap[idx]] = idx;
            idx = pidx;
        }
        self.heap[idx] = v;
        *self.pos[v] = idx;
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
            *self.pos[self.heap[idx]] = idx;
            idx = child;
        }
        self.heap[idx] = v;
        *self.pos[v] = idx;
    }

    #[inline]
    pub fn push(&mut self, var: Var, activity: &Activity) {
        if self.pos[var].is_some() {
            return;
        }
        let idx = self.heap.len();
        self.heap.push(var);
        *self.pos[var] = idx;
        self.up(var, activity);
    }

    #[inline]
    pub fn pop(&mut self, activity: &Activity) -> Option<Var> {
        if self.heap.is_empty() {
            return None;
        }
        let value = self.heap[0];
        self.heap[0] = self.heap[self.heap.len() - 1];
        *self.pos[self.heap[0]] = 0;
        self.pos[value] = OptionU32::NONE;
        self.heap.pop();
        if self.heap.len() > 1 {
            self.down(0, activity);
        }
        Some(value)
    }
}

#[derive(Default)]
pub struct IntervalHeap {
    heap: Gvec<Var>,
}

impl IntervalHeap {
    #[inline]
    fn len(&self) -> u32 {
        self.heap.len()
    }

    #[inline]
    fn left(x: u32) -> u32 {
        x & !1
    }

    #[inline]
    fn right(x: u32) -> u32 {
        x | 1
    }

    #[inline]
    fn parent_left(x: u32) -> u32 {
        Self::left((x - 2) >> 1)
    }

    #[inline]
    fn parent_right(x: u32) -> u32 {
        Self::right((x - 2) >> 1)
    }

    #[inline]
    fn up_max(&mut self, mut idx: u32, act: &mut Activity) {
        let v = self.heap[idx];
        while idx > 1 {
            let pidx = Self::parent_right(idx);
            if act[self.heap[pidx]] >= act[v] {
                break;
            }
            self.heap[idx] = self.heap[pidx];
            act.pos[self.heap[idx]] = idx;
            idx = pidx;
        }
        self.heap[idx] = v;
        act.pos[v] = idx;
    }

    #[inline]
    fn up_min(&mut self, mut idx: u32, act: &mut Activity) {
        let v = self.heap[idx];
        while idx > 1 {
            let pidx = Self::parent_left(idx);
            if act[self.heap[pidx]] <= act[v] {
                break;
            }
            self.heap[idx] = self.heap[pidx];
            act.pos[self.heap[idx]] = idx;
            idx = pidx;
        }
        self.heap[idx] = v;
        act.pos[v] = idx;
    }

    #[inline]
    fn down_max(&mut self, mut right: u32, act: &mut Activity) {
        debug_assert!(Self::right(right) == right);
        let mut right_v = self.heap[right];
        loop {
            let c1 = right * 2 + 1;
            if self.heap.len() <= c1 {
                break;
            }
            let c2 = right * 2 + 3;
            let ch = if self.heap.len() <= c2 || act[self.heap[c1]] > act[self.heap[c2]] {
                c1
            } else {
                c2
            };
            if act[self.heap[ch]] > act[right_v] {
                self.heap[right] = self.heap[ch];
                act.pos[self.heap[right]] = right;
                right = ch;
                let left = right - 1;
                if act[self.heap[left]] > act[right_v] {
                    swap(&mut right_v, &mut self.heap[left]);
                    act.pos[self.heap[left]] = left;
                }
            } else {
                break;
            }
        }
        self.heap[right] = right_v;
        act.pos[right_v] = right;
    }

    #[inline]
    fn down_min(&mut self, mut left: u32, act: &mut Activity) {
        debug_assert!(Self::left(left) == left);
        let mut left_v = self.heap[left];
        loop {
            let c1 = left * 2 + 2;
            if self.heap.len() <= c1 {
                break;
            }
            let c2 = left * 2 + 4;
            let ch = if self.heap.len() <= c2 || act[self.heap[c1]] < act[self.heap[c2]] {
                c1
            } else {
                c2
            };
            if act[self.heap[ch]] < act[left_v] {
                self.heap[left] = self.heap[ch];
                act.pos[self.heap[left]] = left;
                left = ch;
                let right = left + 1;
                if right < self.heap.len() && act[left_v] > act[self.heap[right]] {
                    swap(&mut left_v, &mut self.heap[right]);
                    act.pos[self.heap[right]] = right;
                }
            } else {
                break;
            }
        }
        self.heap[left] = left_v;
        act.pos[left_v] = left;
    }

    #[inline]
    fn up(&mut self, var: Var, act: &mut Activity) {
        let mut idx = act.pos[var];
        if Self::left(idx) == idx {
            let mut left = idx;
            loop {
                let c1 = left * 2 + 2;
                if self.heap.len() <= c1 {
                    break;
                }
                let c2 = left * 2 + 4;
                let ch = if self.heap.len() <= c2 || act[self.heap[c1]] < act[self.heap[c2]] {
                    c1
                } else {
                    c2
                };
                if act[self.heap[ch]] < act[var] {
                    self.heap[left] = self.heap[ch];
                    act.pos[self.heap[left]] = left;
                    left = ch;
                } else {
                    break;
                }
            }
            idx = left;
        }
        let right = Self::right(idx);
        if right < self.len() && act[var] > act[self.heap[right]] {
            self.heap[idx] = self.heap[right];
            act.pos[self.heap[idx]] = idx;
            idx = right;
        }
        self.heap[idx] = var;
        act.pos[var] = idx;
        self.up_max(idx, act)
    }

    #[inline]
    pub fn push(&mut self, v: Var, act: &mut Activity) {
        self.heap.push(v);
        act.pos[v] = self.heap.len() - 1;
        let max = self.heap.len() - 1;
        let min = Self::left(max);
        if act[self.heap[min]] > act[v] {
            act.pos.swap(self.heap[min], self.heap[max]);
            self.heap.swap(max, min);
        }
        self.up_min(min, act);
        self.up_max(max, act);
    }

    #[inline]
    pub fn max(&self) -> Option<Var> {
        match self.heap.len() {
            0..=2 => self.heap.last().cloned(),
            _ => Some(self.heap[1]),
        }
    }

    #[inline]
    pub fn pop_max(&mut self, act: &mut Activity) -> Option<Var> {
        match self.heap.len() {
            0..=2 => self.heap.pop(),
            _ => {
                let res = self.heap.swap_remove(1);
                act.pos[self.heap[1]] = 1;
                self.down_max(1, act);
                Some(res)
            }
        }
    }

    #[inline]
    pub fn min(&self) -> Option<Var> {
        match self.heap.len() {
            0 => None,
            _ => Some(self.heap[0]),
        }
    }

    #[inline]
    pub fn pop_min(&mut self, act: &mut Activity) -> Option<Var> {
        match self.heap.len() {
            0 => None,
            1..=2 => {
                let res = Some(self.heap.swap_remove(0));
                if !self.heap.is_empty() {
                    act.pos[self.heap[0]] = 0;
                }
                res
            }
            _ => {
                let res = self.heap.swap_remove(0);
                act.pos[self.heap[0]] = 0;
                self.down_min(0, act);
                Some(res)
            }
        }
    }

    // fn valid(&self, act: &mut Activity) {
    //     let mut i = 0;
    //     while i < self.heap.len() {
    //         if i + 1 < self.len() {
    //             assert!(act[self.heap[i]] <= act[self.heap[i]]);
    //         }
    //         let c1 = i * 2 + 2;
    //         if c1 < self.len() {
    //             assert!(act[self.heap[i]] <= act[self.heap[c1]]);
    //         }
    //         let c2 = i * 2 + 4;
    //         if c2 < self.len() {
    //             assert!(act[self.heap[i]] <= act[self.heap[c2]]);
    //         }
    //         i += 2;
    //     }
    //     let mut i = 1;
    //     while i < self.heap.len() {
    //         assert!(act[self.heap[i]] >= act[self.heap[i - 1]]);
    //         let c1 = i * 2 + 1;
    //         if c1 < self.len() {
    //             assert!(act[self.heap[i]] >= act[self.heap[c1]]);
    //         }
    //         let c2 = i * 2 + 3;
    //         if c2 < self.len() {
    //             assert!(act[self.heap[i]] >= act[self.heap[c2]]);
    //         }
    //         i += 2;
    //     }
    // }

    // pub fn print(&self, act: &mut Activity) {
    //     for x in self.heap.iter() {
    //         dbg!(*x, act[*x]);
    //     }
    // }
}

const NUM_BUCKET: u32 = 15;

pub struct Activity {
    activity: VarMap<f64>,
    act_inc: f64,
    bucket_heap: Gvec<IntervalHeap>,
    bucket: VarMap<OptionU8>,
    pos: VarMap<u32>,
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
        self.bucket.reserve(var);
        self.pos.reserve(var);
    }

    #[inline]
    fn check(&mut self, var: Var) {
        let act = unsafe { &mut *(self as *mut Activity) };
        if self.bucket[var].is_none() {
            assert!(act[var] == 0.0);
            self.bucket_heap[NUM_BUCKET - 1].push(var, act);
            *self.bucket[var] = NUM_BUCKET as u8 - 1;
            self.update();
        }
    }

    #[inline]
    fn bucket(&self, var: Var) -> u32 {
        match self.bucket[var] {
            OptionU8::NONE => NUM_BUCKET,
            b => *b as u32,
        }
    }

    #[inline]
    pub fn bump(&mut self, var: Var) {
        self.check(var);
        self.activity[var] += self.act_inc;
        self.up(var);
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

    #[inline]
    fn up(&mut self, var: Var) {
        let act = unsafe { &mut *(self as *mut Activity) };
        let mut now = *self.bucket[var] as u32;
        self.bucket_heap[now].up(var, act);
        while now > 0 {
            if self.activity[self.bucket_heap[now].max().unwrap()]
                > self.activity[self.bucket_heap[now - 1].min().unwrap()]
            {
                let max = self.bucket_heap[now].pop_max(act).unwrap();
                let min = self.bucket_heap[now - 1].pop_min(act).unwrap();
                *self.bucket[max] = now as u8 - 1;
                *self.bucket[min] = now as u8;
                self.bucket_heap[now].push(min, act);
                self.bucket_heap[now - 1].push(max, act);
            } else {
                break;
            }
            now -= 1;
        }
    }

    #[inline]
    fn update(&mut self) {
        let mut now = NUM_BUCKET - 1;
        let act = unsafe { &mut *(self as *mut Activity) };
        while now > 0 {
            if self.bucket_heap[now].len() > self.bucket_heap[now - 1].len() * 2 {
                let max = self.bucket_heap[now].pop_max(act).unwrap();
                *self.bucket[max] = now as u8 - 1;
                self.bucket_heap[now - 1].push(max, act);
            } else {
                break;
            }
            now -= 1;
        }
    }

    pub fn sort_by_activity(&self, cube: &mut Cube, ascending: bool) {
        if ascending {
            cube.sort_by(|a, b| self.activity[*a].partial_cmp(&self.activity[*b]).unwrap());
        } else {
            cube.sort_by(|a, b| self.activity[*b].partial_cmp(&self.activity[*a]).unwrap());
        }
    }

    // pub fn print(&self) {
    //     dbg!("begin print");
    //     for i in 0..NUM_BUCKET {
    //         dbg!(i);
    //         dbg!(self.bucket_heap[i].len());
    //         if self.bucket_heap[i].len() > 0 {
    //             let max = self.activity[self.bucket_heap[i].max().unwrap()];
    //             let min = self.activity[self.bucket_heap[i].min().unwrap()];
    //             dbg!(max);
    //             dbg!(min);
    //         }
    //     }
    //     dbg!("end");
    // }

    // pub fn valid(&self) {
    //     let mut m = None;
    //     for i in 0..NUM_BUCKET {
    //         if self.bucket_heap[i].len() > 0 {
    //             let max = self.activity[self.bucket_heap[i].max().unwrap()];
    //             let min = self.activity[self.bucket_heap[i].min().unwrap()];
    //             assert!(max >= min);
    //             if let Some(m) = m {
    //                 assert!(m >= max);
    //             }
    //             m = Some(min);
    //         }
    //     }
    // }
}

impl Default for Activity {
    fn default() -> Self {
        let mut bucket_heap = Gvec::new();
        for _ in 0..NUM_BUCKET {
            bucket_heap.push(IntervalHeap::default());
        }
        Self {
            act_inc: 1.0,
            activity: Default::default(),
            bucket_heap,
            bucket: Default::default(),
            pos: Default::default(),
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

    #[inline]
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
                let decide = if self.phase_saving[decide].is_none() {
                    Lit::new(decide, self.rng.gen_bool(0.5))
                } else {
                    Lit::new(decide, self.phase_saving[decide] != Lbool::FALSE)
                };
                self.pos_in_trail.push(self.trail.len());
                self.assign(decide, CREF_NONE);
                return true;
            }
        }
        false
    }
}
