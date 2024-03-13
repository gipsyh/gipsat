#![feature(get_mut_unchecked)]

mod analyze;
mod cdb;
mod domain;
mod propagate;
mod search;
mod simplify;
mod statistic;
mod utils;
mod vsids;

use crate::utils::Lbool;
use analyze::Analyze;
use cdb::{CRef, ClauseDB, ClauseKind, CREF_NONE};
use domain::Domain;
use giputils::gvec::Gvec;
use logic_form::{Clause, Cube, Lit, LitSet, Var, VarMap};
use propagate::Watchers;
use satif::{SatResult, SatifSat, SatifUnsat};
use search::Value;
use simplify::Simplify;
use statistic::Statistic;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};
use transys::Model;
use utils::Rng;
use vsids::Vsids;

#[derive(Default)]
pub struct Solver {
    id: usize,
    cdb: ClauseDB,
    watchers: Watchers,
    value: Value,
    trail: Gvec<Lit>,
    pos_in_trail: Vec<u32>,
    level: VarMap<u32>,
    reason: VarMap<CRef>,
    propagated: u32,
    vsids: Vsids,
    phase_saving: VarMap<Lbool>,
    analyze: Analyze,
    simplify: Simplify,
    unsat_core: LitSet,

    domain: Domain,
    temporary_domain: bool,
    lazy_temporary: Vec<Clause>,

    ts: Rc<Model>,
    frame: Frame,

    statistic: Statistic,

    constrain_act: Option<Lit>,

    rng: Rng,
}

impl Solver {
    pub fn new(id: usize, ts: &Rc<Model>, frame: &Frame) -> Self {
        let mut solver = Self {
            id,
            ts: ts.clone(),
            frame: frame.clone(),
            ..Default::default()
        };
        while solver.num_var() < solver.ts.num_var {
            solver.new_var();
        }
        for cls in ts.trans.iter() {
            solver.add_clause_inner(cls, ClauseKind::Trans);
        }
        solver
    }

    pub fn new_var(&mut self) -> Var {
        let var = Var::new(self.num_var());
        self.value.reserve(var);
        self.level.reserve(var);
        self.reason.reserve(var);
        self.watchers.reserve(var);
        self.vsids.reserve(var);
        self.phase_saving.reserve(var);
        self.analyze.reserve(var);
        self.unsat_core.reserve(var);
        self.domain.reserve(var);
        var
    }

    #[inline]
    pub fn num_var(&self) -> usize {
        self.reason.len()
    }

    fn simplify_clause(&mut self, cls: &[Lit]) -> Option<logic_form::Clause> {
        assert!(self.highest_level() == 0);
        let mut clause = logic_form::Clause::new();
        for l in cls.iter() {
            while self.num_var() <= l.var().into() {
                self.new_var();
            }
            match self.value.v(*l) {
                Lbool::TRUE => return None,
                Lbool::FALSE => (),
                _ => clause.push(*l),
            }
        }
        assert!(!clause.is_empty());
        Some(clause)
    }

    pub fn add_clause_inner(&mut self, clause: &[Lit], mut kind: ClauseKind) -> CRef {
        let clause = match self.simplify_clause(clause) {
            Some(clause) => clause,
            None => return CREF_NONE,
        };
        for l in clause.iter() {
            if let Some(act) = self.constrain_act {
                if act.var() == l.var() {
                    kind = ClauseKind::Temporary;
                }
            }
        }
        if clause.len() == 1 {
            assert!(!matches!(kind, ClauseKind::Temporary));
            match self.value.v(clause[0]) {
                Lbool::TRUE | Lbool::FALSE => todo!(),
                _ => {
                    self.assign(clause[0], CREF_NONE);
                    assert!(self.propagate() == CREF_NONE);
                    CREF_NONE
                }
            }
        } else {
            self.attach_clause(&clause, kind)
        }
    }

    fn add_lemma(&mut self, lemma: &[Lit]) -> CRef {
        self.backtrack(0, false);
        self.clean_temporary();
        for l in lemma.iter() {
            self.domain.lemma.insert(l.var());
        }
        self.add_clause_inner(lemma, ClauseKind::Lemma)
    }

    fn remove_lemma(&mut self, cref: CRef) {
        self.backtrack(0, false);
        self.clean_temporary();
        if !self.locked(self.cdb.get(cref)) {
            self.remove_clause(cref)
        }
    }

    fn new_round(&mut self, domain: Option<impl Iterator<Item = Var>>, bucket: bool) {
        if !self.pos_in_trail.is_empty() {
            while self.trail.len() > self.pos_in_trail[0] {
                let bt = self.trail.pop().unwrap();
                self.value.set_none(bt.var());
                self.phase_saving[bt] = Lbool::from(bt.polarity());
                if self.temporary_domain {
                    self.vsids.push(bt.var());
                }
            }
            self.propagated = self.pos_in_trail[0];
            self.pos_in_trail.truncate(0);
        }
        self.clean_temporary();

        // dbg!(&self.name);
        // self.vsids.activity.dbg();
        // dbg!(self.num_var());
        // dbg!(self.trail.len());
        // dbg!(self.cdb.num_leanrt());
        // dbg!(self.cdb.num_lemma());

        while let Some(lc) = self.lazy_temporary.pop() {
            self.add_clause_inner(&lc, ClauseKind::Temporary);
        }

        if !self.temporary_domain {
            if let Some(domain) = domain {
                self.domain.enable_local(domain, &self.ts, &self.value);
                if self.constrain_act.is_some() {
                    assert!(!self.domain.local.has(self.constrain_act.unwrap().var()));
                    self.domain.local.insert(self.constrain_act.unwrap().var());
                }
            }
            if bucket {
                self.vsids.enable_bucket = true;
                self.vsids.bucket.clear();
            } else {
                self.vsids.enable_bucket = false;
                self.vsids.heap.clear();
            }
            for d in self.domain.domains() {
                if self.value.v(d.lit()).is_none() {
                    self.vsids.push(*d);
                }
            }
        }
    }

    pub fn solve_with_domain(&mut self, assumption: &[Lit], bucket: bool) -> SatResult<Sat, Unsat> {
        if self.temporary_domain {
            assert!(bucket);
        }
        self.new_round(Some(assumption.iter().map(|l| l.var())), bucket);
        self.statistic.num_solve += 1;
        self.clean_leanrt();
        self.simplify();
        self.garbage_collect();
        self.search_with_restart(assumption)
    }

    pub fn solve_with_constrain(
        &mut self,
        assump: &[Lit],
        mut constrain: Clause,
        bucket: bool,
    ) -> SatResult<Sat, Unsat> {
        if self.temporary_domain {
            assert!(bucket);
        }
        if self.constrain_act.is_none() {
            let constrain_act = self.new_var();
            self.constrain_act = Some(constrain_act.lit());
        }
        let act = self.constrain_act.unwrap();
        let mut assumption = Cube::new();
        assumption.extend_from_slice(assump);
        assumption.push(act);
        let cc = constrain.clone();
        constrain.push(!act);
        self.lazy_temporary.push(constrain);
        self.new_round(
            Some(assump.iter().chain(cc.iter()).map(|l| l.var())),
            bucket,
        );
        self.statistic.num_solve += 1;
        self.clean_leanrt();
        self.simplify();
        self.garbage_collect();
        self.search_with_restart(&assumption)
    }

    pub fn set_domain(&mut self, domain: impl Iterator<Item = Lit>) {
        self.temporary_domain = true;
        if !self.pos_in_trail.is_empty() {
            while self.trail.len() > self.pos_in_trail[0] {
                let bt = self.trail.pop().unwrap();
                self.value.set_none(bt.var());
                self.phase_saving[bt] = Lbool::from(bt.polarity());
            }
            self.propagated = self.pos_in_trail[0];
            self.pos_in_trail.truncate(0);
        }
        self.clean_temporary();
        self.domain
            .enable_local(domain.map(|l| l.var()), &self.ts, &self.value);
        assert!(!self.domain.local.has(self.constrain_act.unwrap().var()));
        self.domain.local.insert(self.constrain_act.unwrap().var());
        self.vsids.enable_bucket = true;
        self.vsids.bucket.clear();
        for d in self.domain.domains() {
            self.vsids.push(*d);
        }
    }

    pub fn unset_domain(&mut self) {
        self.temporary_domain = false;
    }
}

pub struct Sat {
    solver: *mut Solver,
}

impl SatifSat for Sat {
    #[inline]
    fn lit_value(&self, lit: Lit) -> Option<bool> {
        let solver = unsafe { &*self.solver };
        match solver.value.v(lit) {
            Lbool::TRUE => Some(true),
            Lbool::FALSE => Some(false),
            _ => None,
        }
    }
}

pub struct Unsat {
    solver: *mut Solver,
}

impl SatifUnsat for Unsat {
    #[inline]
    fn has(&self, lit: Lit) -> bool {
        let solver = unsafe { &*self.solver };
        solver.unsat_core.has(lit)
    }
}

pub enum BlockResult {
    Yes(BlockResultYes),
    No(BlockResultNo),
}

pub struct BlockResultYes {
    pub unsat: Unsat,
    pub cube: Cube,
    pub assumption: Cube,
}

pub struct BlockResultNo {
    pub sat: Sat,
    pub assumption: Cube,
}

impl BlockResultNo {
    #[inline]
    pub fn lit_value(&self, lit: Lit) -> Option<bool> {
        self.sat.lit_value(lit)
    }
}

#[derive(Debug, Clone)]
pub struct Lemma {
    pub lemma: logic_form::Lemma,
    cref: Vec<CRef>,
}

impl Deref for Lemma {
    type Target = logic_form::Lemma;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lemma
    }
}

#[derive(Clone, Default)]
pub struct Frame {
    frames: Rc<Vec<Vec<Lemma>>>,
}

impl Deref for Frame {
    type Target = Vec<Vec<Lemma>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl DerefMut for Frame {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl Frame {
    #[inline]
    pub fn get_mut(&mut self) -> &mut Vec<Vec<Lemma>> {
        unsafe { Rc::get_mut_unchecked(&mut self.frames) }
    }
}

pub struct GipSAT {
    model: Rc<Model>,
    pub frame: Frame,
    pub solvers: Vec<Solver>,
    tmp_lit_set: LitSet,
    early: usize,
}

impl GipSAT {
    pub fn new(model: Model) -> Self {
        let mut tmp_lit_set = LitSet::new();
        tmp_lit_set.reserve(model.max_latch);
        Self {
            model: Rc::new(model),
            frame: Default::default(),
            solvers: Default::default(),
            tmp_lit_set,
            early: 1,
        }
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.frame.len() - 1
    }

    pub fn new_frame(&mut self) {
        self.solvers
            .push(Solver::new(self.frame.len(), &self.model, &self.frame));
        self.frame.push(Vec::new());
    }

    #[inline]
    pub fn trivial_contained(&mut self, frame: usize, lemma: &logic_form::Lemma) -> bool {
        for l in lemma.iter() {
            self.tmp_lit_set.insert(*l);
        }
        for i in frame..self.frame.len() {
            for l in self.frame[i].iter() {
                if l.subsume_set(lemma, &self.tmp_lit_set) {
                    self.tmp_lit_set.clear();
                    return true;
                }
            }
        }
        self.tmp_lit_set.clear();
        false
    }

    #[inline]
    pub fn add_lemma(&mut self, frame: usize, lemma: Cube) {
        let lemma = logic_form::Lemma::new(lemma);
        if frame == 0 {
            assert!(self.frame.len() == 1);
            let cref = vec![self.solvers[0].add_lemma(&!lemma.cube())];
            self.frame[0].push(Lemma { lemma, cref });
            return;
        }
        if self.trivial_contained(frame, &lemma) {
            return;
        }
        assert!(!self.model.cube_subsume_init(lemma.cube()));
        let mut begin = None;
        'fl: for i in (1..=frame).rev() {
            let mut j = 0;
            while j < self.frame[i].len() {
                let l = &self.frame[i][j];
                if begin.is_none() && l.subsume(&lemma) {
                    if l.eq(&lemma) {
                        let mut eq_lemma = self.frame[i].swap_remove(j);
                        let clause = !lemma.cube();
                        for k in i + 1..=frame {
                            eq_lemma.cref.push(self.solvers[k].add_lemma(&clause));
                        }
                        assert!(eq_lemma.cref.len() == frame + 1);
                        self.frame[frame].push(eq_lemma);
                        self.early = self.early.min(i + 1);
                        return;
                    } else {
                        begin = Some(i + 1);
                        break 'fl;
                    }
                }
                if lemma.subsume(l) {
                    assert!(l.cref.len() == i + 1);
                    for k in 0..=i {
                        if l.cref[k] != CREF_NONE {
                            self.solvers[k].remove_lemma(l.cref[k]);
                        }
                    }
                    self.frame[i].swap_remove(j);
                    continue;
                }
                j += 1;
            }
        }
        let clause = !lemma.cube();
        let begin = begin.unwrap_or(1);
        let mut cref = vec![CREF_NONE; begin];
        for i in begin..=frame {
            cref.push(self.solvers[i].add_lemma(&clause))
        }
        assert!(cref.len() == frame + 1);
        self.frame[frame].push(Lemma { lemma, cref });
        self.early = self.early.min(begin);
    }

    pub fn blocked(
        &mut self,
        frame: usize,
        cube: &Cube,
        strengthen: bool,
        bucket: bool,
    ) -> BlockResult {
        let solver_idx = frame - 1;
        let assumption = self.model.cube_next(cube);
        let res = if strengthen {
            let constrain = !cube;
            self.solvers[solver_idx].solve_with_constrain(&assumption, constrain, bucket)
        } else {
            self.solvers[solver_idx].solve_with_domain(&assumption, bucket)
        };
        match res {
            SatResult::Sat(sat) => BlockResult::No(BlockResultNo { sat, assumption }),
            SatResult::Unsat(unsat) => BlockResult::Yes(BlockResultYes {
                unsat,
                cube: cube.clone(),
                assumption,
            }),
        }
    }

    pub fn get_bad(&mut self) -> Option<BlockResultNo> {
        match self
            .solvers
            .last_mut()
            .unwrap()
            .solve_with_domain(&self.model.bad, false)
        {
            SatResult::Sat(sat) => Some(BlockResultNo {
                sat,
                assumption: self.model.bad.clone(),
            }),
            SatResult::Unsat(_) => None,
        }
    }

    pub fn blocked_conflict(&mut self, block: BlockResultYes) -> Cube {
        let mut ans = Cube::new();
        for i in 0..block.cube.len() {
            if block.unsat.has(block.assumption[i]) {
                ans.push(block.cube[i]);
            }
        }
        if self.model.cube_subsume_init(&ans) {
            ans = Cube::new();
            let new = *block
                .cube
                .iter()
                .find(|l| {
                    self.model
                        .init_map
                        .get(&l.var())
                        .is_some_and(|i| *i != l.polarity())
                })
                .unwrap();
            for i in 0..block.cube.len() {
                if block.unsat.has(block.assumption[i]) || block.cube[i] == new {
                    ans.push(block.cube[i]);
                }
            }
            assert!(!self.model.cube_subsume_init(&ans));
        }
        ans
    }

    pub fn propagate(&mut self) -> bool {
        for frame_idx in self.early..self.depth() {
            self.frame[frame_idx].sort_by_key(|x| x.len());
            let frame = self.frame[frame_idx].clone();
            for lemma in frame {
                if !self.frame[frame_idx].iter().any(|l| l.lemma == lemma.lemma) {
                    continue;
                }
                match self.blocked(frame_idx + 1, &lemma, false, true) {
                    BlockResult::Yes(blocked) => {
                        let conflict = self.blocked_conflict(blocked);
                        self.add_lemma(frame_idx + 1, conflict);
                    }
                    BlockResult::No(_) => {}
                }
            }
            if self.frame[frame_idx].is_empty() {
                return true;
            }
        }
        self.early = self.frame.len() - 1;
        false
    }

    pub fn statistic(&self) {
        for f in self.frame.iter() {
            print!("{} ", f.len());
        }
        println!();
    }
}
