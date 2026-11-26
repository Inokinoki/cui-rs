//! Headless frames for the tests: a context and a frame around a closure,
//! the way the safe layer's tests drive widgets without a window.

use crate::{Arena, Ui};
use cui_sys as ffi;
use std::ptr;

/// One headless frame around `f`: a fresh context, `cui_begin_frame`, the
/// closure, `cui_end_frame`, then the context is destroyed — on a panic too.
pub(crate) fn frame(f: impl FnOnce(&mut Ui)) {
    let ctx = unsafe { ffi::cui_create(&ffi::cui_ctx_desc::default()) };
    assert!(!ctx.is_null());
    let arena = Arena::new();
    unsafe {
        ffi::cui_set_viewport(ctx, 800., 600., 1.);
        ffi::cui_begin_frame(ctx, 1. / 60.);
    }
    // The tests that check for a panic unwind through here mid-frame.
    let _destroy = DestroyOnDrop(ptr::NonNull::new(ctx).unwrap());
    f(&mut Ui { ctx, arena: &arena });
    unsafe { ffi::cui_end_frame(ctx) };
}

struct DestroyOnDrop(ptr::NonNull<ffi::cui_ctx>);

impl Drop for DestroyOnDrop {
    fn drop(&mut self) {
        unsafe { ffi::cui_destroy(self.0.as_ptr()) }
    }
}
