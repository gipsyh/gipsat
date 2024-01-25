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
