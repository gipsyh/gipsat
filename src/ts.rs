use logic_form::{Var, VarMap};

pub struct TransitionSystem {
    dependence: VarMap<Vec<Var>>,
}

impl TransitionSystem {
    pub fn new(dependence: VarMap<Vec<Var>>) -> Self {
        Self { dependence }
    }

    pub fn get_coi(
        &self,
        root: impl Iterator<Item = Var>,
        mark_stamp: u32,
        mark: &mut VarMap<u32>,
        marks: &mut Vec<Var>,
    ) {
        for r in root {
            if mark[r] != mark_stamp {
                marks.push(r);
                mark[r] = mark_stamp;
            }
        }
        let mut now = 0;
        while now < marks.len() {
            let v = marks[now];
            now += 1;
            for d in self.dependence[v].iter() {
                if mark[*d] != mark_stamp {
                    marks.push(*d);
                    mark[*d] = mark_stamp;
                }
            }
        }
    }
}
