use crate::Solver;
use logic_form::{Lit, Var};

#[derive(Clone, Copy, Debug)]
pub struct Watcher {
    clause: usize,
    blocker: Lit,
}

impl Watcher {
    pub fn new(clause: usize, blocker: Lit) -> Self {
        Self { clause, blocker }
    }
}

impl Solver {
    pub fn propagate(&mut self) -> Option<usize> {
        while self.propagated < self.trail.len() {
            let p = self.trail[self.propagated];
            self.propagated += 1;
            let mut new = 0;
            for w in 0..self.watchers[p].len() {
                let watchers = &mut self.watchers[p];
                if let Some(true) = self.value[watchers[w].blocker] {
                    watchers[new] = watchers[w];
                    new += 1;
                    continue;
                }
                let cid = watchers[w].clause;
                let cref = &mut self.clauses[cid];
                if cref[0] == !p {
                    cref.swap(0, 1);
                }
                assert!(cref[1] == !p);
                let new_watcher = Watcher::new(cid, cref[0]);
                if let Some(true) = self.value[cref[0]] {
                    watchers[new] = new_watcher;
                    new += 1;
                    continue;
                }
                let new_lit = cref[2..]
                    .iter()
                    .position(|l| !matches!(self.value[*l], Some(false)));
                if let Some(new_lit) = new_lit {
                    cref.swap(1, new_lit + 2);
                    self.watchers[!cref[1]].push(new_watcher);
                } else {
                    watchers[new] = new_watcher;
                    new += 1;
                    if let Some(false) = self.value[cref[0]] {
                        for i in w + 1..watchers.len() {
                            watchers[new] = watchers[i];
                            new += 1;
                        }
                        watchers.truncate(new);
                        return Some(cid);
                    } else {
                        let assign = cref[0];
                        self.assign(assign, Some(cid));
                    }
                }
            }
            self.watchers[p].truncate(new);
        }
        None
    }
}
