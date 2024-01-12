use logic_form::{Lit, Var};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug, Default)]
pub struct VarMap<T> {
    map: Vec<T>,
}

impl<T> Index<Var> for VarMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Var) -> &Self::Output {
        &self.map[Into::<usize>::into(index)]
    }
}

impl<T> IndexMut<Var> for VarMap<T> {
    #[inline]
    fn index_mut(&mut self, index: Var) -> &mut Self::Output {
        &mut self.map[Into::<usize>::into(index)]
    }
}

impl<T> Index<Lit> for VarMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Lit) -> &Self::Output {
        &self.map[Into::<usize>::into(index.var())]
    }
}

impl<T> IndexMut<Lit> for VarMap<T> {
    #[inline]
    fn index_mut(&mut self, index: Lit) -> &mut Self::Output {
        &mut self.map[Into::<usize>::into(index.var())]
    }
}

impl<T> Deref for VarMap<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<T> DerefMut for VarMap<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[derive(Debug, Default)]
pub struct LitMap<T> {
    map: Vec<T>,
}

impl<T> Index<Lit> for LitMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Lit) -> &Self::Output {
        &self.map[Into::<usize>::into(index)]
    }
}

impl<T> IndexMut<Lit> for LitMap<T> {
    #[inline]
    fn index_mut(&mut self, index: Lit) -> &mut Self::Output {
        &mut self.map[Into::<usize>::into(index)]
    }
}

impl<T> Deref for LitMap<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<T> DerefMut for LitMap<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[derive(Default, Debug)]
pub struct LitSet {
    set: Vec<Lit>,
    has: LitMap<bool>,
}

impl LitSet {
    pub fn new_var(&mut self) {
        self.has.push(false);
        self.has.push(false);
    }

    pub fn insert(&mut self, lit: Lit) {
        if !self.has[lit] {
            self.set.push(lit);
            self.has[lit] = true;
        }
    }

    pub fn has(&self, lit: Lit) -> bool {
        self.has[lit]
    }

    pub fn clear(&mut self) {
        for l in self.set.iter() {
            self.has[*l] = false;
        }
        self.set.clear();
    }
}

pub struct Rand {
    rng: StdRng,
}

impl Rand {
    pub fn rand_bool(&mut self) -> bool {
        self.rng.gen_bool(0.5)
    }
}

impl Default for Rand {
    fn default() -> Self {
        Self {
            rng: StdRng::seed_from_u64(0),
        }
    }
}
