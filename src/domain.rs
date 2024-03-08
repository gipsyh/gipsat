use crate::{search::Value, ts::TransitionSystem};
use logic_form::{Var, VarSet};
use std::slice;

pub struct Domain {
    pub lemma: VarSet,
    pub local: VarSet,
}

impl Domain {
    pub fn reserve(&mut self, var: Var) {
        self.lemma.reserve(var);
        self.local.reserve(var);
    }

    pub fn enable_local(
        &mut self,
        domain: impl Iterator<Item = Var>,
        ts: &TransitionSystem,
        value: &Value,
    ) {
        self.local.clear();
        ts.get_coi(domain, &mut self.local, value);
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

impl Default for Domain {
    fn default() -> Self {
        Self {
            lemma: Default::default(),
            local: Default::default(),
        }
    }
}
