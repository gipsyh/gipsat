use logic_form::{Lit, Var};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug, Default)]
pub struct VarMap<T> {
    map: Vec<T>,
}

impl<T> Index<Var> for VarMap<T> {
    type Output = T;

    fn index(&self, index: Var) -> &Self::Output {
        &self.map[Into::<usize>::into(index)]
    }
}

impl<T> IndexMut<Var> for VarMap<T> {
    fn index_mut(&mut self, index: Var) -> &mut Self::Output {
        &mut self.map[Into::<usize>::into(index)]
    }
}

impl<T> Deref for VarMap<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<T> DerefMut for VarMap<T> {
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

    fn index(&self, index: Lit) -> &Self::Output {
        &self.map[Into::<usize>::into(index)]
    }
}

impl<T> IndexMut<Lit> for LitMap<T> {
    fn index_mut(&mut self, index: Lit) -> &mut Self::Output {
        &mut self.map[Into::<usize>::into(index)]
    }
}

impl<T> Deref for LitMap<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<T> DerefMut for LitMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
