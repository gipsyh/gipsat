use crate::{basic::Clause, utils::VarMap, Solver};
use logic_form::Lit;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, Default)]
pub enum Mark {
    #[default]
    Unseen,
    Seen,
    Removable,
    Failed,
}

impl Mark {
    #[inline]
    pub fn seen(&self) -> bool {
        !matches!(self, Mark::Unseen)
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Mark::Unseen;
    }
}

#[derive(Default)]
pub struct Analyze {
    seen: VarMap<Mark>,
    clear: Vec<Lit>,
}

impl Analyze {
    pub fn new_var(&mut self) {
        self.seen.push(Default::default());
        self.clear.push(Default::default());
    }

    pub fn clear(&mut self) {
        for c in self.clear.iter() {
            self.seen[*c].clear();
        }
        self.clear.clear();
    }
}

impl Deref for Analyze {
    type Target = VarMap<Mark>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.seen
    }
}

impl DerefMut for Analyze {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.seen
    }
}

impl Solver {
    fn lit_redundant(&mut self, lit: Lit) -> bool {
        assert!(matches!(self.analyze[lit], Mark::Unseen | Mark::Seen));
        if self.reason[lit].is_none() {
            return false;
        }
        let mut stack: Vec<(Lit, usize)> = vec![(lit, 1)];
        'a: while let Some((p, b)) = stack.pop() {
            let c = self.reason[p].unwrap();
            for i in b..self.clauses[c].len() {
                let l = self.clauses[c][i];
                if self.level[l] == 0 || matches!(self.analyze[l], Mark::Seen | Mark::Removable) {
                    continue;
                }
                if self.reason[l].is_none() || matches!(self.analyze[l], Mark::Failed) {
                    stack.push((p, 0));
                    for (l, _) in stack {
                        if let Mark::Unseen = self.analyze[l] {
                            self.analyze[l] = Mark::Failed;
                            self.analyze.clear.push(l);
                        }
                    }
                    return false;
                }
                stack.push((p, i + 1));
                stack.push((l, 1));
                continue 'a;
            }
            if let Mark::Unseen = self.analyze[p] {
                self.analyze[p] = Mark::Removable;
                self.analyze.clear.push(p);
            }
        }
        true
    }

    fn minimal_learnt(&mut self, learnt: Clause) -> Clause {
        let mut minimal_learnt = Clause::from([learnt[0]]);
        for l in &learnt[1..] {
            if !self.lit_redundant(*l) {
                minimal_learnt.push(*l);
            }
        }
        minimal_learnt
    }

    pub fn analyze(&mut self, mut conflict: usize) -> (Clause, usize) {
        let mut learnt = Clause::from([Lit::default()]);
        let mut path = 0;
        let mut trail_idx = self.trail.len() - 1;
        let mut resolve_lit = None;
        loop {
            let cref = &self.clauses[conflict];
            let begin = if resolve_lit.is_some() { 1 } else { 0 };
            for l in begin..cref.len() {
                let lit = cref[l];
                if !self.analyze[lit].seen() && self.level[lit] > 0 {
                    self.vsids.var_bump(lit.var());
                    self.analyze[lit] = Mark::Seen;
                    if self.level[lit] >= self.highest_level() {
                        path += 1;
                    } else {
                        learnt.push(lit);
                    }
                }
            }
            while !self.analyze[self.trail[trail_idx]].seen() {
                trail_idx -= 1;
            }
            self.analyze[self.trail[trail_idx]].clear();
            resolve_lit = Some(self.trail[trail_idx]);
            path -= 1;
            if path == 0 {
                break;
            }
            conflict = self.reason[self.trail[trail_idx]].unwrap();
        }
        learnt[0] = !resolve_lit.unwrap();
        self.analyze.clear.extend_from_slice(&learnt);
        learnt = self.minimal_learnt(learnt);
        let btl = if learnt.len() == 1 {
            0
        } else {
            let max_idx = (1..learnt.len())
                .max_by_key(|idx| self.level[learnt[*idx]])
                .unwrap();
            learnt.swap(1, max_idx);
            self.level[learnt[1]]
        };
        self.analyze.clear();
        (learnt, btl)
    }
}
