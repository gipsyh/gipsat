use crate::Solver;
use logic_form::{Lit, LitMap};

#[derive(Clone, Copy, Debug)]
pub struct Watcher {
    pub clause: usize,
    blocker: Lit,
}

impl Watcher {
    #[inline]
    pub fn new(clause: usize, blocker: Lit) -> Self {
        Self { clause, blocker }
    }
}

#[derive(Default)]
pub struct Watchers {
    pub wtrs: LitMap<Vec<Watcher>>,
}

impl Watchers {
    #[inline]
    pub fn reserve(&mut self, size: usize) {
        self.wtrs.resize_with(size * 2, Default::default);
    }

    #[inline]
    pub fn attach(&mut self, cref: usize, cls: &[Lit]) {
        self.wtrs[!cls[0]].push(Watcher::new(cref, cls[1]));
        self.wtrs[!cls[1]].push(Watcher::new(cref, cls[0]));
    }

    #[inline]
    pub fn detach(&mut self, cref: usize, cls: &[Lit]) {
        self.wtrs[!cls[0]].retain(|w| w.clause != cref);
        self.wtrs[!cls[1]].retain(|w| w.clause != cref);
    }
}

impl Solver {
    pub fn propagate(&mut self) -> Option<usize> {
        let propagate_full = self.highest_level() == 0;
        while self.propagated < self.trail.len() {
            let p = self.trail[self.propagated];
            self.propagated += 1;

            let mut new = 0;
            for w in 0..self.watchers.wtrs[p].len() {
                let watchers = &mut self.watchers.wtrs[p];
                let blocker = watchers[w].blocker;
                match self.value[blocker] {
                    Some(true) => {
                        watchers[new] = watchers[w];
                        new += 1;
                        continue;
                    }
                    None => {
                        if !propagate_full && !self.domain.has(blocker) {
                            watchers[new] = watchers[w];
                            new += 1;
                            continue;
                        }
                    }
                    Some(false) => (),
                }
                let cid = watchers[w].clause;
                let cref = &mut self.cdb[cid];
                if cref[0] == !p {
                    cref.swap(0, 1);
                }
                assert!(cref[1] == !p);
                let new_watcher = Watcher::new(cid, cref[0]);
                match self.value[cref[0]] {
                    Some(true) => {
                        watchers[new] = new_watcher;
                        new += 1;
                        continue;
                    }
                    None => {
                        if !propagate_full && !self.domain.has(cref[0]) {
                            watchers[new] = new_watcher;
                            new += 1;
                            continue;
                        }
                    }
                    Some(false) => (),
                }
                let new_lit = cref[2..]
                    .iter()
                    .position(|l| !matches!(self.value[*l], Some(false)));
                if let Some(new_lit) = new_lit {
                    cref.swap(1, new_lit + 2);
                    self.watchers.wtrs[!cref[1]].push(new_watcher);
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
                        if propagate_full || self.domain.has(cref[0]) {
                            let assign = cref[0];
                            self.assign(assign, Some(cid));
                        }
                    }
                }
            }
            self.watchers.wtrs[p].truncate(new);
        }
        None
    }
}
