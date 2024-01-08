use crate::utils::VarMap;
use logic_form::Var;

#[derive(Default)]
pub struct Vsids {
    activity: VarMap<f64>,
    heap: Vec<Var>,
    pos: VarMap<Option<usize>>,
}

impl Vsids {
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

    pub fn push(&mut self, var: Var) {
        if self.pos[var].is_some() {
            return;
        }
        let mut idx = self.heap.len();
        self.heap.push(var);
        self.pos[var] = Some(idx);
        while idx > 0 {
            let pidx = (idx - 1) / 2;
            if self.activity[self.heap[pidx]] >= self.activity[self.heap[idx]] {
                break;
            }
            self.swap(pidx, idx);
            idx = pidx;
        }
    }

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
            self.heap.swap(idx, smallest);
            idx = smallest;
        }
        value
    }
}
