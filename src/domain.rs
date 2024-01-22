use crate::utils::Mark;
use logic_form::Var;

#[derive(Default)]
pub struct Domain {
    pub global: Mark,
    local: Mark,
    enable_local: bool,
}

impl Domain {
    pub fn new_var(&mut self) {
        self.global.new_var();
        self.local.new_var();
    }

    pub fn enable_local(&mut self, domain: &[Var]) {
        self.local.clean_all();
        for v in domain.iter() {
            if self.global.is_marked(*v) {
                self.local.mark(*v);
            }
        }
        self.enable_local = true;
    }

    pub fn disable_local(&mut self) {
        self.enable_local = false;
    }

    #[inline]
    pub fn has<V: Into<Var>>(&self, var: V) -> bool {
        if self.enable_local {
            self.local.is_marked(var)
        } else {
            self.global.is_marked(var)
        }
    }

    pub fn domains(&self) -> impl Iterator<Item = &Var> {
        if self.enable_local {
            self.local.marks()
        } else {
            self.global.marks()
        }
    }
}
