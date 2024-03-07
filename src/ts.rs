use crate::search::Value;
use logic_form::{Var, VarMap};
use std::rc::Rc;

pub struct TransitionSystem {
    pub dependence: Rc<VarMap<Vec<Var>>>,
}

impl TransitionSystem {
    pub fn new(dependence: Rc<VarMap<Vec<Var>>>) -> Self {
        Self { dependence }
    }

    pub fn get_coi(
        &self,
        root: impl Iterator<Item = Var>,
        mark_stamp: u32,
        mark: &mut VarMap<u32>,
        marks: &mut Vec<Var>,
        value: &Value,
    ) {
        for r in root {
            if value.v(r.lit()).is_none() && mark[r] != mark_stamp {
                marks.push(r);
                mark[r] = mark_stamp;
            }
        }
        let mut now = 0;
        while now < marks.len() {
            let v = marks[now];
            now += 1;
            for d in self.dependence[v].iter() {
                if value.v(d.lit()).is_none() && mark[*d] != mark_stamp {
                    marks.push(*d);
                    mark[*d] = mark_stamp;
                }
            }
        }
    }
}
