use crate::utils::Mark;
use logic_form::{Var, VarMap};

pub struct TransitionSystem {
    dependence: VarMap<Vec<Var>>,
}

impl TransitionSystem {
    pub fn new(dependence: VarMap<Vec<Var>>) -> Self {
        Self { dependence }
    }

    pub fn get_coi(&self, root: impl Iterator<Item = Var>, mark: &mut Mark, domain: &Mark) {
        let mut queue = Vec::from_iter(root);
        for v in queue.iter() {
            mark.mark(*v);
        }
        while let Some(v) = queue.pop() {
            for d in self.dependence[v].iter() {
                if !mark.is_marked(*d) {
                    mark.mark(*d);
                    queue.push(*d);
                }
            }
        }
    }
}
