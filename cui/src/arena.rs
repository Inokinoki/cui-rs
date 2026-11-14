//! Per-frame string arena: turns `&str` into NUL-terminated C strings and
//! pointer arrays without per-call allocation.
//!
//! The arena is cleared by [`App::frame`](crate::App::frame) after CUI has
//! presented the previous frame, so every pointer it hands out stays valid
//! for exactly one frame — which is all an immediate-mode call needs.
//!
//! Methods take `&self` with an `UnsafeCell` because every `Ui` handle in a
//! frame shares one arena. This is sound: the arena is only reachable from
//! `App`/`Ui`, which are confined to the UI thread (they wrap non-`Send` C
//! pointers), and accesses are strictly sequential within a frame.

use core::cell::UnsafeCell;
use core::ffi::c_char;

pub struct Arena {
    buf: UnsafeCell<Vec<u8>>,
}

impl Arena {
    pub(crate) fn new() -> Arena {
        Arena { buf: UnsafeCell::new(Vec::with_capacity(8192)) }
    }

    /// Called after `cui_app_frame` returns: the previous frame has been
    /// presented, so nothing reads this frame's strings anymore.
    pub(crate) fn reset(&mut self) {
        self.buf.get_mut().clear();
    }

    // Sound: the arena is only reachable from `App`/`Ui`, which are confined
    // to the UI thread, and accesses are sequential within a frame.
    #[allow(clippy::mut_from_ref)]
    fn buf(&self) -> &mut Vec<u8> {
        unsafe { &mut *self.buf.get() }
    }

    fn align(&self, align: usize) {
        let buf = self.buf();
        let pad = buf.len().next_multiple_of(align) - buf.len();
        buf.extend(core::iter::repeat_n(0u8, pad));
    }

    /// Copies `s` and NUL-terminates it. Panics on interior NUL bytes — CUI
    /// strings are NUL-terminated by contract; the `##`/`###` identity
    /// suffixes are plain text and pass through unchanged.
    pub fn cstr(&self, s: &str) -> *const c_char {
        assert!(!s.contains('\0'), "CUI strings must not contain NUL bytes: {s:?}");
        let buf = self.buf();
        let start = buf.len();
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
        unsafe { buf.as_ptr().add(start) as *const c_char }
    }

    /// Copies a `&[&str]` into an aligned `*const *const c_char` array.
    pub fn cstr_array(&self, items: &[&str]) -> *const *const c_char {
        assert!(!items.is_empty());
        self.align(core::mem::align_of::<*const c_char>());
        let buf = self.buf();
        let start = buf.len();
        buf.extend(core::iter::repeat_n(0u8, core::mem::size_of::<*const c_char>() * items.len()));
        let base = unsafe { buf.as_ptr().add(start) as *mut *const c_char };
        for (i, item) in items.iter().enumerate() {
            unsafe { *base.add(i) = self.cstr(item) };
        }
        base
    }

    /// Copies a slice of fixed-size C structs into an aligned array.
    pub fn struct_array<T: Copy>(&self, items: &[T]) -> *const T {
        assert!(!items.is_empty());
        self.align(core::mem::align_of::<T>());
        let buf = self.buf();
        let start = buf.len();
        buf.extend_from_slice(unsafe {
            core::slice::from_raw_parts(items.as_ptr() as *const u8, core::mem::size_of_val(items))
        });
        unsafe { buf.as_ptr().add(start).cast::<T>() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstr_round_trip() {
        let a = Arena::new();
        let p = a.cstr("Hello");
        let s = unsafe { core::ffi::CStr::from_ptr(p) };
        assert_eq!(s.to_str().unwrap(), "Hello");
    }

    #[test]
    fn cstr_rejects_nul() {
        let a = Arena::new();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.cstr("a\0b"))).is_err());
    }

    #[test]
    fn array_is_aligned_and_nul_separated() {
        let a = Arena::new();
        a.cstr("x"); // shift the buffer so the array must re-align
        let items = ["All", "Open", "Done"];
        let p = a.cstr_array(&items);
        assert_eq!(p as usize % core::mem::align_of::<*const c_char>(), 0);
        for (i, want) in items.iter().enumerate() {
            let s = unsafe { core::ffi::CStr::from_ptr(*p.add(i)) };
            assert_eq!(s.to_str().unwrap(), *want);
        }
    }

    #[test]
    fn reset_empties() {
        let mut a = Arena::new();
        a.cstr("frame 1");
        a.reset();
        assert!(a.buf().is_empty());
    }
}
