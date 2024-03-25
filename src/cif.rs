use crate::GipSAT;
use std::{ffi::c_int, mem::forget, os::raw::c_void};
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
pub extern "C" fn gipsat_extend(gipsat: *mut c_void) {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.extend()
}

#[no_mangle]
pub extern "C" fn gipsat_propagate(gipsat: *mut c_void) -> c_int {
    let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
    gipsat.propagate() as _
}

// #[no_mangle]
// pub extern "C" fn gipsat_get_bad(gipsat: *mut c_void) -> c_int {
//     let gipsat = unsafe { &mut *(gipsat as *mut GipSAT) };
//     gipsat.get_bad()
// }
