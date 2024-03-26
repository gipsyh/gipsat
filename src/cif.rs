use crate::GipSAT;
use core::ffi::c_size_t;
use giputils::crffi::RustVec;
use logic_form::{Cube, Lit};
use std::{
    ffi::{c_int, c_uint},
    mem::forget,
    os::raw::c_void,
    slice::from_raw_parts,
};
use transys::Transys;

#[no_mangle]
pub extern "C" fn gipsat_new(ts: *const c_void) -> *mut c_void {
    assert!(!ts.is_null());
    let ts = unsafe { &*(ts as *const Transys) };
    let gipsat = Box::new(GipSAT::new(ts.clone()));
    let ptr = gipsat.as_ref() as *const GipSAT as *mut c_void;
    forget(gipsat);
    ptr
}

#[no_mangle]
pub extern "C" fn gipsat_drop(gipsat: *mut c_void) {
    let gipsat: Box<GipSAT> = unsafe { Box::from_raw(gipsat as *mut _) };
    drop(gipsat)
}

#[no_mangle]
pub extern "C" fn gipsat_level(gipsat: *mut c_void) -> c_size_t {
    let gipsat = unsafe { &*(gipsat as *const GipSAT) };
    gipsat.level() as _
}

#[no_mangle]
pub extern "C" fn gipsat_extend(gipsat: *mut c_void) {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.extend()
}

#[no_mangle]
pub extern "C" fn gipsat_add_lemma(
    gipsat: *mut c_void,
    frame: c_int,
    cube_ptr: *const c_uint,
    cube_len: c_uint,
) {
    let mut lemma = Cube::new();
    let cube_ptr = cube_ptr as *const Lit;
    let cube_len = cube_len as usize;
    for i in 0..cube_len {
        lemma.push(unsafe { *cube_ptr.add(i) })
    }
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.add_lemma(frame as _, lemma)
}

#[no_mangle]
pub extern "C" fn gipsat_inductive(
    gipsat: *mut c_void,
    frame: c_uint,
    cube_ptr: *const c_uint,
    cube_len: c_uint,
    strengthen: c_int,
) -> c_int {
    let cube = unsafe { from_raw_parts(cube_ptr as *const Lit, cube_len as _) };
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.inductive(frame as _, cube, strengthen == 1) as _
}

#[no_mangle]
pub extern "C" fn gipsat_inductive_core(gipsat: *mut c_void) -> RustVec {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    let core: Vec<Lit> = gipsat.inductive_core().into();
    RustVec::new(core)
}

#[no_mangle]
pub extern "C" fn gipsat_get_predecessor(gipsat: *mut c_void) -> RustVec {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    let core: Vec<Lit> = gipsat.get_predecessor().into();
    RustVec::new(core)
}

#[no_mangle]
pub extern "C" fn gipsat_propagate(gipsat: *mut c_void) -> c_int {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.propagate() as _
}

#[no_mangle]
pub extern "C" fn gipsat_has_bad(gipsat: *mut c_void) -> c_int {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.has_bad() as _
}

#[no_mangle]
pub extern "C" fn gipsat_set_domain(
    gipsat: *mut c_void,
    frame: c_int,
    d_ptr: *const c_uint,
    d_len: c_uint,
) {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    let d = unsafe { from_raw_parts(d_ptr as *const Lit, d_len as _) };
    gipsat.set_domain(frame as _, d.iter().copied())
}

#[no_mangle]
pub extern "C" fn gipsat_unset_domain(gipsat: *mut c_void, frame: c_int) {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.unset_domain(frame as _);
}
