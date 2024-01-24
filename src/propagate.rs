use crate::Solver;
use logic_form::{Lit, LitMap};
use std::ops::{Deref, DerefMut};

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

#[derive(Default)]
pub struct Watchers {
    watchers: LitMap<Vec<Watcher>>,
}

impl Watchers {
    pub fn remove(&mut self, lit: Lit, clause: usize) {
        self.watchers[lit].retain(|w| w.clause != clause);
    }
}

impl Deref for Watchers {
    type Target = LitMap<Vec<Watcher>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.watchers
    }
}

impl DerefMut for Watchers {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.watchers
    }
}

impl Solver {
    pub fn propagate(&mut self) -> Option<usize> {
        let propagate_full = self.highest_level() == 0;
        while self.propagated < self.trail.len() {
            let p = self.trail[self.propagated];
            self.propagated += 1;
            let mut new = 0;
            for w in 0..self.watchers[p].len() {
                let watchers = &mut self.watchers[p];
                let blocker = watchers[w].blocker;
                if (!self.domain.has(blocker)
                    && !matches!(self.value[blocker], Some(false))
                    && !propagate_full)
                    || matches!(self.value[blocker], Some(true))
                {
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
                if (!self.domain.has(cref[0])
                    && !matches!(self.value[cref[0]], Some(false))
                    && !propagate_full)
                    || matches!(self.value[cref[0]], Some(true))
                {
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
                        if self.domain.has(cref[0]) || propagate_full {
                            let assign = cref[0];
                            self.assign(assign, Some(cid));
                        }
                    }
                }
            }
            self.watchers[p].truncate(new);
        }
        None
    }
}
