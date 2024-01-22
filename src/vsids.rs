use crate::Solver;
use logic_form::{Var, VarMap};
use std::ops::MulAssign;

pub struct Vsids {
    activity: VarMap<f64>,
    heap: Vec<Var>,
    pos: VarMap<Option<usize>>,
    act_inc: f64,
}

impl Default for Vsids {
    fn default() -> Self {
        Self {
            activity: Default::default(),
            heap: Default::default(),
            pos: Default::default(),
            act_inc: 1.0,
        }
    }
}

impl Vsids {
    pub fn new_var(&mut self) {
        self.pos.push(None);
        self.activity.push(f64::default());
    }

    #[inline]
    pub fn clear(&mut self) {
        while let Some(v) = self.heap.pop() {
            self.pos[v] = None;
        }
    }

    #[inline]
    fn swap(&mut self, x: usize, y: usize) {
        self.pos[self.heap[x]] = Some(y);
        self.pos[self.heap[y]] = Some(x);
        self.heap.swap(x, y);
    }

    fn up(&mut self, mut idx: usize) {
        while idx > 0 {
            let pidx = (idx - 1) / 2;
            if self.activity[self.heap[pidx]] >= self.activity[self.heap[idx]] {
                break;
            }
            self.swap(pidx, idx);
            idx = pidx;
        }
    }

    #[inline]
    pub fn push(&mut self, var: Var) {
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
        if self.heap.is_empty() {
            return None;
        }
        self.swap(0, self.heap.len() - 1);
        let value = self.heap.pop();
        self.pos[value.unwrap()] = None;
        let mut idx = 0;
        loop {
            let mut smallest = idx;
            for cidx in 2 * idx + 1..(2 * idx + 3).min(self.heap.len()) {
                if self.activity[self.heap[cidx]] > self.activity[self.heap[smallest]] {
                    smallest = cidx;
                }
            }
            if smallest == idx {
                break;
            }
            self.swap(idx, smallest);
            idx = smallest;
        }
        value
    }

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

    pub fn decay(&mut self) {
        self.act_inc *= 1.0 / Self::DECAY
    }
}

impl Solver {
    #[inline]
    pub fn decide(&mut self) -> bool {
        while let Some(decide) = self.vsids.pop() {
            if self.value[decide.lit()].is_none() {
                let decide = self.phase_saving[decide].unwrap_or(decide.lit());
                self.pos_in_trail.push(self.trail.len());
                self.assign(decide, None);
                return true;
            }
        }
        false
    }
}
