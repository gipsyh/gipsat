use crate::utils::VarMap;
use logic_form::Var;
use std::ops::MulAssign;

pub struct Vsids {
    activity: VarMap<f64>,
    heap: Vec<Var>,
    pos: VarMap<Option<usize>>,
    var_inc: f64,
}

impl Default for Vsids {
    fn default() -> Self {
        Self {
            activity: Default::default(),
            heap: Default::default(),
            pos: Default::default(),
            var_inc: 1.0,
        }
    }
}

impl Vsids {
    const VAR_DECAY: f64 = 0.95;

    pub fn new_var(&mut self) {
        self.pos.push(None);
        self.activity.push(f64::default());
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

    pub fn var_bump(&mut self, var: Var) {
        self.activity[var] += self.var_inc;
        if self.activity[var] > 1e100 {
            self.activity.iter_mut().for_each(|a| a.mul_assign(1e-100));
            self.var_inc *= 1e-100;
        }
        if let Some(pos) = self.pos[var] {
            self.up(pos)
        }
    }

    pub fn var_decay(&mut self) {
        self.var_inc *= 1.0 / Self::VAR_DECAY
    }
}
