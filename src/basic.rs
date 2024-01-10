use logic_form::Lit;
use std::ops::{Deref, DerefMut};

use crate::Solver;

#[derive(Default, Debug)]
pub struct Clause {
    clause: logic_form::Clause,
}

impl Deref for Clause {
    type Target = logic_form::Clause;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.clause
    }
}

impl DerefMut for Clause {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.clause
    }
}

impl<F: Into<logic_form::Clause>> From<F> for Clause {
    fn from(value: F) -> Self {
        Self {
            clause: value.into(),
        }
    }
}

impl FromIterator<Lit> for Clause {
    fn from_iter<T: IntoIterator<Item = Lit>>(iter: T) -> Self {
        Self {
            clause: logic_form::Clause::from_iter(iter),
        }
    }
}

impl Solver {
    pub fn satisfied(&self, cls: usize) -> bool {
        for l in self.clauses[cls].iter() {
            if let Some(true) = self.value[*l] {
                return true;
            }
        }
        false
    }
}
