use crate::GipSAT;
use core::ffi::c_size_t;
use giputils::crffi::RustVec;
use logic_form::{Cube, Lit};
use std::{
    ffi::{c_int, c_uint},
    mem::{forget, transmute},
    os::raw::c_void,
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
pub extern "C" fn gipsat_propagate(gipsat: *mut c_void) -> c_int {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.propagate() as _
}

#[no_mangle]
pub extern "C" fn gipsat_get_bad(gipsat: *mut c_void) -> RustVec {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    match gipsat.get_bad() {
        Some(bad) => {
            assert!(!bad.is_empty());
            let bad: Vec<Lit> = unsafe { transmute(bad) };
            RustVec::new(bad)
        }
        None => RustVec::new(Vec::<Lit>::new()),
    }
}
