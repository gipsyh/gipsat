#![feature(get_mut_unchecked, c_size_t)]

mod analyze;
mod cdb;
mod cif;
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
use rand::{rngs::StdRng, SeedableRng};
use satif::{SatResult, SatifSat, SatifUnsat};
use search::Value;
use simplify::Simplify;
use statistic::{GipSATStatistic, SolverStatistic};
use std::{
    mem::take,
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Instant,
};
use transys::Transys;
use vsids::Vsids;

pub struct Solver {
    id: Option<usize>,
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
    constrain_act: Option<Lit>,

    ts: Rc<Transys>,
    frame: Frame,

    rng: StdRng,
    statistic: SolverStatistic,
}

impl Solver {
    pub fn new(id: Option<usize>, ts: &Rc<Transys>, frame: &Frame) -> Self {
        let mut solver = Self {
            id,
            ts: ts.clone(),
            frame: frame.clone(),
            cdb: Default::default(),
            watchers: Default::default(),
            value: Default::default(),
            trail: Default::default(),
            pos_in_trail: Default::default(),
            level: Default::default(),
            reason: Default::default(),
            propagated: Default::default(),
            vsids: Default::default(),
            phase_saving: Default::default(),
            analyze: Default::default(),
            simplify: Default::default(),
            unsat_core: Default::default(),
            domain: Default::default(),
            temporary_domain: Default::default(),
            statistic: Default::default(),
            constrain_act: None,
            rng: StdRng::seed_from_u64(0),
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

    fn add_clause_inner(&mut self, clause: &[Lit], mut kind: ClauseKind) -> CRef {
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

    #[inline]
    fn add_lemma(&mut self, lemma: &[Lit]) -> CRef {
        self.backtrack(0, false);
        self.clean_temporary();
        for l in lemma.iter() {
            self.domain.lemma.insert(l.var());
        }
        self.add_clause_inner(lemma, ClauseKind::Lemma)
    }

    #[inline]
    fn remove_lemma(&mut self, cref: CRef) {
        self.backtrack(0, false);
        self.clean_temporary();
        if !self.locked(cref) {
            self.remove_clause(cref)
        }
    }

    fn new_round(
        &mut self,
        domain: Option<impl Iterator<Item = Var>>,
        constrain: Option<Clause>,
        bucket: bool,
    ) {
        self.backtrack(0, self.temporary_domain);
        self.clean_temporary();
        // dbg!(&self.name);
        // self.vsids.activity.print();
        // dbg!(self.num_var());
        // dbg!(self.trail.len());
        // dbg!(self.cdb.num_leanrt());
        // dbg!(self.cdb.num_lemma());

        if let Some(constrain) = constrain {
            if let Some(constrain) = self.simplify_clause(&constrain) {
                if constrain.len() > 1 {
                    self.add_clause_inner(&constrain, ClauseKind::Temporary);
                }
            }
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
        self.statistic.avg_decide_var += self.domain.domains().len() as f64
            / (self.ts.num_var - self.trail.len() as usize) as f64
    }

    pub fn solve_with_domain(&mut self, assumption: &[Lit], bucket: bool) -> SatResult<Sat, Unsat> {
        if self.temporary_domain {
            assert!(bucket);
        }
        self.new_round(Some(assumption.iter().map(|l| l.var())), None, bucket);
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
        self.new_round(
            Some(assump.iter().chain(cc.iter()).map(|l| l.var())),
            Some(constrain),
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
        self.backtrack(0, false);
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
    begin: usize,
    cref: Vec<CRef>,
}

impl Deref for Lemma {
    type Target = logic_form::Lemma;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lemma
    }
}

impl Lemma {
    #[inline]
    fn get_cref(&self, id: usize) -> CRef {
        if id < self.begin {
            CREF_NONE
        } else {
            self.cref[id - self.begin]
        }
    }

    #[inline]
    fn set_cref(&mut self, id: usize, cref: CRef) {
        assert!(id >= self.begin);
        self.cref[id - self.begin] = cref
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
    ts: Rc<Transys>,
    pub frame: Frame,
    solvers: Vec<Solver>,
    lift: Solver,
    tmp_lit_set: LitSet,
    early: usize,
    last_ind: Option<BlockResult>,
    statistic: GipSATStatistic,
}

impl GipSAT {
    /// create a new GipSAT instance from a transition system
    pub fn new(ts: Transys) -> Self {
        let ts = Rc::new(ts);
        let mut tmp_lit_set = LitSet::new();
        tmp_lit_set.reserve(ts.max_latch);
        let frame = Default::default();
        let lift = Solver::new(None, &ts, &frame);
        Self {
            ts,
            frame,
            solvers: Default::default(),
            lift,
            tmp_lit_set,
            early: 1,
            last_ind: None,
            statistic: Default::default(),
        }
    }

    /// get the highest level of GipSAT
    #[inline]
    pub fn level(&self) -> usize {
        self.frame.len() - 1
    }

    /// extend with a new frame    
    #[inline]
    pub fn extend(&mut self) {
        self.solvers
            .push(Solver::new(Some(self.frame.len()), &self.ts, &self.frame));
        self.frame.push(Vec::new());
        if self.level() == 0 {
            for cube in self.ts.inits() {
                self.add_lemma(0, cube)
            }
        }
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

    /// add a lemma to frame
    #[inline]
    pub fn add_lemma(&mut self, frame: usize, lemma: Cube) {
        let lemma = logic_form::Lemma::new(lemma);
        if frame == 0 {
            assert!(self.frame.len() == 1);
            assert!(self.solvers[0].add_lemma(&!lemma.cube()) == CREF_NONE);
            self.frame[0].push(Lemma {
                lemma,
                cref: Vec::new(),
                begin: 1,
            });
            return;
        }
        if self.trivial_contained(frame, &lemma) {
            return;
        }
        assert!(!self.ts.cube_subsume_init(lemma.cube()));
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
                        self.frame[frame].push(eq_lemma);
                        self.early = self.early.min(i + 1);
                        return;
                    } else {
                        begin = Some(i + 1);
                        break 'fl;
                    }
                }
                if lemma.subsume(l) {
                    for k in l.begin..=i {
                        let cref = l.get_cref(k);
                        if cref != CREF_NONE {
                            self.solvers[k].remove_lemma(cref);
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
        let mut cref = Vec::new();
        for i in begin..=frame {
            cref.push(self.solvers[i].add_lemma(&clause))
        }
        self.frame[frame].push(Lemma { lemma, cref, begin });
        self.early = self.early.min(begin);
    }

    pub fn parent_lemma(&self, cube: &Cube, frame: usize) -> Vec<&logic_form::Lemma> {
        let lemma = logic_form::Lemma::new(cube.clone());
        let mut res = Vec::new();
        if frame == 1 {
            return res;
        }
        for c in self.frame[frame - 1].iter() {
            if c.subsume(&lemma) {
                res.push(&c.lemma);
            }
        }
        res
    }

    /// query whether the cube is inductively relative to the frame
    pub fn inductive(&mut self, frame: usize, cube: &[Lit], strengthen: bool) -> bool {
        let start = Instant::now();
        self.statistic.num_sat += 1;
        let solver_idx = frame - 1;
        let assumption = self.ts.cube_next(cube);
        let res = if strengthen {
            let constrain = Clause::from_iter(cube.iter().map(|l| !*l));
            self.solvers[solver_idx].solve_with_constrain(&assumption, constrain, true)
        } else {
            self.solvers[solver_idx].solve_with_domain(&assumption, true)
        };
        self.last_ind = Some(match res {
            SatResult::Sat(sat) => BlockResult::No(BlockResultNo { sat, assumption }),
            SatResult::Unsat(unsat) => BlockResult::Yes(BlockResultYes {
                unsat,
                cube: Cube::from(cube),
                assumption,
            }),
        });
        self.statistic.avg_sat_time += start.elapsed();
        matches!(self.last_ind.as_ref().unwrap(), BlockResult::Yes(_))
    }

    /// get the inductive core
    pub fn inductive_core(&mut self) -> Cube {
        let last_ind = take(&mut self.last_ind);
        let block = match last_ind.unwrap() {
            BlockResult::Yes(block) => block,
            BlockResult::No(_) => panic!(),
        };
        let mut ans = Cube::new();
        for i in 0..block.cube.len() {
            if block.unsat.has(block.assumption[i]) {
                ans.push(block.cube[i]);
            }
        }
        if self.ts.cube_subsume_init(&ans) {
            ans = Cube::new();
            let new = *block
                .cube
                .iter()
                .find(|l| {
                    self.ts
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
            assert!(!self.ts.cube_subsume_init(&ans));
        }
        ans
    }

    /// get the predecessor
    pub fn get_predecessor(&mut self) -> Cube {
        let last_ind = take(&mut self.last_ind);
        let unblock = match last_ind.unwrap() {
            BlockResult::Yes(_) => panic!(),
            BlockResult::No(unblock) => unblock,
        };
        let mut assumption = Cube::new();
        let cls = !&unblock.assumption;
        for input in self.ts.inputs.iter() {
            let lit = input.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => assumption.push(lit),
                Some(false) => assumption.push(!lit),
                None => (),
            }
        }
        let mut latchs = Cube::new();
        for latch in self.ts.latchs.iter() {
            let lit = latch.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => latchs.push(lit),
                Some(false) => latchs.push(!lit),
                None => (),
            }
        }
        let solver = unsafe { &*unblock.sat.solver };
        solver.vsids.activity.sort_by_activity(&mut latchs, false);
        assumption.extend_from_slice(&latchs);
        let res: Cube = match self.lift.solve_with_constrain(&assumption, cls, false) {
            SatResult::Sat(_) => panic!(),
            SatResult::Unsat(conflict) => latchs.into_iter().filter(|l| conflict.has(*l)).collect(),
        };
        res
    }

    /// perform the propagation.
    pub fn propagate(&mut self) -> bool {
        for frame_idx in self.early..self.level() {
            self.frame[frame_idx].sort_by_key(|x| x.len());
            let frame = self.frame[frame_idx].clone();
            for lemma in frame {
                if self.frame[frame_idx].iter().all(|l| l.lemma != lemma.lemma) {
                    continue;
                }
                if self.inductive(frame_idx + 1, &lemma, false) {
                    let core = self.inductive_core();
                    self.add_lemma(frame_idx + 1, core);
                }
            }
            if self.frame[frame_idx].is_empty() {
                return true;
            }
        }
        self.early = self.level();
        false
    }

    pub fn has_bad(&mut self) -> bool {
        let start = Instant::now();
        self.statistic.num_sat += 1;
        let res = match self
            .solvers
            .last_mut()
            .unwrap()
            .solve_with_domain(&self.ts.bad, false)
        {
            SatResult::Sat(sat) => {
                self.last_ind = Some(BlockResult::No(BlockResultNo {
                    sat,
                    assumption: self.ts.bad.clone(),
                }));
                true
            }
            SatResult::Unsat(_) => false,
        };
        self.statistic.avg_sat_time += start.elapsed();
        res
    }

    pub fn set_domain(&mut self, frame: usize, domain: impl Iterator<Item = Lit>) {
        self.solvers[frame].set_domain(domain)
    }

    pub fn unset_domain(&mut self, frame: usize) {
        self.solvers[frame].unset_domain()
    }

    pub fn statistic(&self) {
        for f in self.frame.iter() {
            print!("{} ", f.len());
        }
        println!();
        let mut statistic = SolverStatistic::default();
        for s in self.solvers.iter() {
            statistic = statistic + s.statistic;
        }
        println!("{:#?}", statistic);
        println!("{:#?}", self.statistic);
    }
}
