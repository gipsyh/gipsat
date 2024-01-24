use crate::utils::Mark;
use logic_form::{Var, VarMap};
use std::collections::VecDeque;

pub struct TransitionSystem {
    dependence: VarMap<Vec<Var>>,
}

impl TransitionSystem {
    pub fn new(dependence: VarMap<Vec<Var>>) -> Self {
        Self { dependence }
    }

    pub fn get_coi(&self, root: impl Iterator<Item = Var>, mark: &mut Mark, domain: &Mark) {
        let mut queue = VecDeque::from_iter(root);
        for v in queue.iter() {
            if domain.is_marked(*v) {
                mark.mark(*v);
            } else {
                mark.weak_mark(*v);
            }
        }
        while let Some(v) = queue.pop_front() {
            for d in self.dependence[v].iter() {
                if !mark.is_marked(*d) {
                    if domain.is_marked(*d) {
                        mark.mark(*d);
                    } else {
                        mark.weak_mark(*d);
                    }
                    queue.push_back(*d);
                }
            }
        }
    }
}
