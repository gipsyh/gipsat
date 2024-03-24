use crate::search::Value;
use logic_form::{Var, VarSet};
use std::{rc::Rc, slice};
use transys::Transys;

#[derive(Default)]
pub struct Domain {
    pub lemma: VarSet,
    pub local: VarSet,
}

impl Domain {
    pub fn reserve(&mut self, var: Var) {
        self.lemma.reserve(var);
        self.local.reserve(var);
    }

    pub fn get_coi(&mut self, root: impl Iterator<Item = Var>, ts: &Rc<Transys>, value: &Value) {
        for r in root {
            if value.v(r.lit()).is_none() {
                self.local.insert(r);
            }
        }
        let mut now = 0;
        while now < self.local.len() {
            let v = self.local[now];
            now += 1;
            for d in ts.dependence[v].iter() {
                if value.v(d.lit()).is_none() {
                    self.local.insert(*d);
                }
            }
        }
    }

    pub fn enable_local(
        &mut self,
        domain: impl Iterator<Item = Var>,
        ts: &Rc<Transys>,
        value: &Value,
    ) {
        self.local.clear();
        self.get_coi(domain, ts, value);
        for l in self.lemma.iter() {
            if value.v(l.lit()).is_none() {
                self.local.insert(*l);
            }
        }
    }

    #[inline]
    pub fn has(&self, var: Var) -> bool {
        self.local.has(var)
    }

    pub fn domains(&self) -> slice::Iter<Var> {
        self.local.iter()
    }
}
