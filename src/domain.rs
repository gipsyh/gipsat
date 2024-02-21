use crate::ts::TransitionSystem;
use logic_form::{Var, VarMap};

pub struct Domain {
    pub global: VarMap<bool>,
    pub global_marks: Vec<Var>,
    pub lemma: VarMap<bool>,
    pub lemma_marks: Vec<Var>,
    local_stamp: u32,
    local: VarMap<u32>,
    local_marks: Vec<Var>,
    enable_local: bool,
}

impl Domain {
    pub fn new_var(&mut self) {
        self.global.push(false);
        self.lemma.push(false);
        self.local.push(0);
    }

    pub fn enable_local(&mut self, domain: impl Iterator<Item = Var>, ts: &TransitionSystem) {
        self.local_stamp += 1;
        self.local_marks.clear();
        ts.get_coi(
            domain,
            self.local_stamp,
            &mut self.local,
            &mut self.local_marks,
        );
        for l in self.lemma_marks.iter() {
            if self.local[*l] != self.local_stamp {
                self.local[*l] = self.local_stamp;
                self.local_marks.push(*l);
            }
        }
        self.enable_local = true;
    }

    pub fn disable_local(&mut self) {
        self.enable_local = false;
    }

    #[inline]
    pub fn has(&self, var: Var) -> bool {
        if self.enable_local {
            self.local[var] == self.local_stamp
        } else {
            self.global[var]
        }
    }

    pub fn domains(&self) -> impl Iterator<Item = &Var> {
        if self.enable_local {
            self.local_marks.iter()
        } else {
            self.global_marks.iter()
        }
    }
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            global: Default::default(),
            global_marks: Default::default(),
            lemma: Default::default(),
            lemma_marks: Default::default(),
            local_stamp: 1,
            local: Default::default(),
            local_marks: Default::default(),
            enable_local: Default::default(),
        }
    }
}
