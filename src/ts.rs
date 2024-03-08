use crate::search::Value;
use logic_form::{Var, VarMap, VarSet};
use std::rc::Rc;

pub struct TransitionSystem {
    pub dependence: Rc<VarMap<Vec<Var>>>,
}

impl TransitionSystem {
    pub fn new(dependence: Rc<VarMap<Vec<Var>>>) -> Self {
        Self { dependence }
    }

    pub fn get_coi(&self, root: impl Iterator<Item = Var>, mark: &mut VarSet, value: &Value) {
        for r in root {
            if value.v(r.lit()).is_none() {
                mark.insert(r);
            }
        }
        let mut now = 0;
        while now < mark.len() {
            let v = mark[now];
            now += 1;
            for d in self.dependence[v].iter() {
                if value.v(d.lit()).is_none() {
                    mark.insert(*d);
                }
            }
        }
    }
}
