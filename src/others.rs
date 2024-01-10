use crate::Solver;
use logic_form::Var;

impl Solver {
    pub fn print_value(&self) {
        for v in 0..self.num_var() {
            let lit = Var::new(v).lit();
            match self.value[lit] {
                Some(true) => print!("{:?}", lit),
                Some(false) => print!("{:?}", !lit),
                None => print!("X"),
            };
            print!("\t");
        }
        println!();
    }
}
