use crate::{search::Value, ts::TransitionSystem};
use logic_form::{Var, VarMap, VarSet};
use std::slice;

pub struct Domain {
    pub lemma: VarSet,
    pub local_stamp: u32,
    pub local: VarMap<u32>,
    pub local_marks: Vec<Var>,
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
        self.local_stamp += 1;
        self.local_marks.clear();
        ts.get_coi(
            domain,
            self.local_stamp,
            &mut self.local,
            &mut self.local_marks,
            value,
        );
        for l in self.lemma.iter() {
            if value.v(l.lit()).is_none() && self.local[*l] != self.local_stamp {
                self.local[*l] = self.local_stamp;
                self.local_marks.push(*l);
            }
        }
    }

    #[inline]
    pub fn has(&self, var: Var) -> bool {
        self.local[var] == self.local_stamp
    }

    pub fn domains(&self) -> slice::Iter<Var> {
        self.local_marks.iter()
    }
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            lemma: Default::default(),
            local_stamp: 1,
            local: Default::default(),
            local_marks: Default::default(),
        }
    }
}
