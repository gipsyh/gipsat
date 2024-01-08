use crate::Solver;

impl Solver {
    pub fn verify(&mut self) -> bool {
        for cls in self.clauses.iter() {
            if !cls.iter().any(|l| matches!(self.value[*l], Some(true))) {
                dbg!(cls);
                return false;
            }
        }
        true
    }
}
