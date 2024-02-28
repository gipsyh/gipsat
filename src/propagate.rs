use crate::{cdb::CRef, Solver};
use logic_form::{Lit, LitMap};

#[derive(Clone, Copy, Debug)]
pub struct Watcher {
    pub clause: CRef,
    blocker: Lit,
}

impl Watcher {
    #[inline]
    pub fn new(clause: CRef, blocker: Lit) -> Self {
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
    pub fn attach(&mut self, cref: CRef, cls: &[Lit]) {
        self.wtrs[!cls[0]].push(Watcher::new(cref, cls[1]));
        self.wtrs[!cls[1]].push(Watcher::new(cref, cls[0]));
    }

    #[inline]
    pub fn detach(&mut self, cref: CRef, cls: &[Lit]) {
        for l in &cls[0..2] {
            for i in (0..self.wtrs[!*l].len()).rev() {
                if self.wtrs[!*l][i].clause == cref {
                    self.wtrs[!*l].swap_remove(i);
                    break;
                }
            }
        }
    }
}

impl Solver {
    pub fn propagate(&mut self) -> Option<CRef> {
        let propagate_full = self.highest_level() == 0;
        while self.propagated < self.trail.len() {
            let p = self.trail[self.propagated];
            self.propagated += 1;

            let mut w = 0;
            'next_cls: while w < self.watchers.wtrs[p].len() {
                let watchers = &mut self.watchers.wtrs[p];
                let blocker = watchers[w].blocker;
                match self.value.v(blocker) {
                    Some(true) => {
                        w += 1;
                        continue;
                    }
                    None => {
                        if !propagate_full && !self.domain.has(blocker.var()) {
                            w += 1;
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
                match self.value.v(cref[0]) {
                    Some(true) => {
                        watchers[w] = new_watcher;
                        w += 1;
                        continue;
                    }
                    None => {
                        if !propagate_full && !self.domain.has(cref[0].var()) {
                            watchers[w] = new_watcher;
                            w += 1;
                            continue;
                        }
                    }
                    Some(false) => (),
                }

                for i in 2..cref.len() {
                    let lit = cref[i];
                    if !matches!(self.value.v(lit), Some(false)) {
                        cref.swap(1, i);
                        watchers.swap_remove(w);
                        self.watchers.wtrs[!cref[1]].push(new_watcher);
                        continue 'next_cls;
                    }
                }
                watchers[w] = new_watcher;
                if let Some(false) = self.value.v(cref[0]) {
                    return Some(cid);
                }
                if propagate_full || self.domain.has(cref[0].var()) {
                    let assign = cref[0];
                    self.assign(assign, cid);
                }
                w += 1;
            }
        }
        None
    }
}
