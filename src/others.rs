use crate::{utils::Lbool, Solver};
use logic_form::Var;

impl Solver {
    pub fn print_value(&self) {
        for v in 0..self.num_var() {
            let lit = Var::new(v).lit();
            match self.value.v(lit) {
                Lbool::TRUE => print!("{:?}", lit),
                Lbool::FALSE => print!("{:?}", !lit),
                _ => print!("X"),
            };
            print!("\t");
        }
        println!();
    }
}
