use crate::{ts::TransitionSystem, utils::Mark};
use logic_form::Var;

#[derive(Default)]
pub struct Domain {
    pub global: Mark,
    pub lemma: Mark,
    local: Mark,
    enable_local: bool,
}

impl Domain {
    pub fn new_var(&mut self) {
        self.global.new_var();
        self.lemma.new_var();
        self.local.new_var();
    }

    pub fn enable_local(&mut self, domain: impl Iterator<Item = Var>, ts: &TransitionSystem) {
        self.local.clean();
        ts.get_coi(domain, &mut self.local, &self.global);
        for l in self.lemma.marks() {
            self.local.mark(*l);
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
