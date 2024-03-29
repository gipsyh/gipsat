use giputils::statistic::Average;
use std::ops::Add;

#[derive(Debug, Default, Clone, Copy)]
pub struct Statistic {
    pub num_solve: usize,
    pub avg_decide_var: Average,
}

impl Add for Statistic {
    type Output = Statistic;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            num_solve: self.num_solve + rhs.num_solve,
            avg_decide_var: self.avg_decide_var + rhs.avg_decide_var,
        }
    }
}
