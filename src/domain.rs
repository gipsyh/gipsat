use crate::search::Value;
use logic_form::{Var, VarSet};
use std::{collections::HashSet, rc::Rc, slice};
use transys::Transys;

pub struct Domain {
    pub lemma: VarSet,
    pub constrain: VarSet,
    pub local: VarSet,
}

impl Domain {
    pub fn new() -> Self {
        Self {
            lemma: Default::default(),
            constrain: Default::default(),
            local: Default::default(),
        }
    }

    pub fn reserve(&mut self, var: Var) {
        self.lemma.reserve(var);
        self.constrain.reserve(var);
        self.local.reserve(var);
    }

    pub fn calculate_constrain(&mut self, ts: &Rc<Transys>, value: &Value) {
        let mut marked = HashSet::new();
        let mut queue = Vec::new();
        for c in ts.constraints.iter() {
            if !marked.contains(&c.var()) {
                marked.insert(c.var());
                queue.push(c.var());
            }
        }
        while let Some(v) = queue.pop() {
            for d in ts.dependence[v].iter() {
                if !marked.contains(d) {
                    marked.insert(*d);
                    queue.push(*d);
                }
            }
        }
        for v in marked.iter() {
            if value.v(v.lit()).is_none() {
                self.constrain.insert(*v);
            }
        }
    }

    fn get_coi(&mut self, root: impl Iterator<Item = Var>, ts: &Rc<Transys>, value: &Value) {
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
        for l in self.lemma.iter().chain(self.constrain.iter()) {
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
