use logic_form::{Var, VarMap};

pub struct Mark {
    timestamp: usize,
    marks: VarMap<usize>,
    marked: Vec<Var>,
}

impl Mark {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_var(&mut self) {
        self.marks.push(0);
    }

    #[inline]
    pub fn num_marked(&self) -> usize {
        self.marked.len()
    }

    #[inline]
    pub fn is_marked<V: Into<Var>>(&self, var: V) -> bool {
        self.marks[var.into()] == self.timestamp
    }

    #[inline]
    pub fn mark(&mut self, var: Var) {
        if !self.is_marked(var) {
            self.marks[var] = self.timestamp;
            self.marked.push(var);
        }
    }

    #[inline]
    pub fn weak_mark(&mut self, var: Var) {
        if !self.is_marked(var) {
            self.marks[var] = self.timestamp;
        }
    }

    #[inline]
    pub fn clean(&mut self) {
        self.timestamp += 1;
        self.marked.clear();
    }

    pub fn marks(&self) -> impl Iterator<Item = &Var> {
        self.marked.iter()
    }
}

impl Default for Mark {
    fn default() -> Self {
        Self {
            timestamp: 1,
            marks: Default::default(),
            marked: Default::default(),
        }
    }
}
