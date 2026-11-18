//! Per-frame string arena: turns `&str` into NUL-terminated C strings and
//! pointer arrays without per-call allocation.
//!
//! The arena is cleared by [`App::frame`](crate::App::frame) after CUI has
//! presented the previous frame, so every pointer it hands out stays valid
//! for exactly one frame — which is all an immediate-mode call needs.
//!
//! Storage is a list of chunks that are never moved or resized: when the
//! current chunk is full a new one is opened, so a pointer handed out earlier
//! in the frame stays valid however much is allocated afterwards (a single
//! growable buffer would move every earlier pointer on reallocation).
//!
//! Methods take `&self` with an `UnsafeCell` because every `Ui` handle in a
//! frame shares one arena. This is sound: the arena is only reachable from
//! `App`/`Ui`, which are confined to the UI thread (the arena itself is
//! neither `Send` nor `Sync`), and accesses are strictly sequential within a
//! frame.

use core::cell::UnsafeCell;
use core::ffi::c_char;
use core::ptr::NonNull;
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};

/// Size of a regular chunk. Bigger requests get a chunk of their own.
const CHUNK: usize = 64 * 1024;
/// Alignment of every chunk's base address.
const CHUNK_ALIGN: usize = 16;

/// One raw allocation. Raw (not `Box<[u8]>`) so that moving the `Chunk`
/// value around inside the `Vec` cannot invalidate pointers into it.
struct Chunk {
    ptr: NonNull<u8>,
    cap: usize,
    used: usize,
}

impl Chunk {
    fn new(cap: usize) -> Chunk {
        let layout = Layout::from_size_align(cap, CHUNK_ALIGN).expect("arena chunk too large");
        // SAFETY: `cap` is non-zero (callers pass at least CHUNK or size + align).
        let raw = unsafe { alloc(layout) };
        let Some(ptr) = NonNull::new(raw) else { handle_alloc_error(layout) };
        Chunk { ptr, cap, used: 0 }
    }

    /// `size` bytes aligned to `align` (a power of two), if they fit.
    fn try_alloc(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let base = self.ptr.as_ptr() as usize;
        let start = (base + self.used).next_multiple_of(align) - base;
        let end = start.checked_add(size)?;
        if end > self.cap {
            return None;
        }
        self.used = end;
        // SAFETY: start <= end <= cap, inside this allocation.
        Some(unsafe { self.ptr.as_ptr().add(start) })
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        // SAFETY: allocated in `Chunk::new` with this exact layout.
        unsafe { dealloc(self.ptr.as_ptr(), Layout::from_size_align_unchecked(self.cap, CHUNK_ALIGN)) }
    }
}

/// The per-frame string arena behind [`Ui::arena`](crate::Ui::arena): the
/// pointers it hands out stay valid for exactly one frame.
pub struct Arena {
    chunks: UnsafeCell<Vec<Chunk>>,
}

impl Arena {
    pub(crate) fn new() -> Arena {
        Arena { chunks: UnsafeCell::new(vec![Chunk::new(CHUNK)]) }
    }

    /// Called after `cui_app_frame` returns: the previous frame has been
    /// presented, so nothing reads this frame's strings anymore. A frame
    /// that fit in one chunk keeps it; one that spilled into several gets a
    /// single chunk as large as all of them, so the next frame of the same
    /// size allocates nothing (keeping only the largest would free and
    /// reallocate the overflow every frame).
    pub(crate) fn reset(&mut self) {
        let chunks = self.chunks.get_mut();
        if chunks.len() > 1 {
            let total = chunks.iter().fold(0usize, |n, c| n.saturating_add(c.cap));
            chunks.clear(); // free first: never hold both generations at once
            chunks.push(Chunk::new(total));
        } else if let Some(only) = chunks.first_mut() {
            only.used = 0;
        }
    }

    /// `size` bytes at `align`, valid until [`Arena::reset`]. The memory is
    /// uninitialised; callers write all of it before handing it out.
    fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        debug_assert!(align.is_power_of_two());
        // Sound: confined to the UI thread, no reference into the Vec outlives
        // this call, and chunk contents never move.
        let chunks = unsafe { &mut *self.chunks.get() };
        if let Some(p) = chunks.last_mut().and_then(|c| c.try_alloc(size, align)) {
            return p;
        }
        let need = size.checked_add(align).expect("arena request too large");
        chunks.push(Chunk::new(CHUNK.max(need)));
        chunks.last_mut().and_then(|c| c.try_alloc(size, align)).expect("fresh chunk fits the request")
    }

    /// Copies `s` and NUL-terminates it. Panics on interior NUL bytes — CUI
    /// strings are NUL-terminated by contract; the `##`/`###` identity
    /// suffixes are plain text and pass through unchanged.
    pub fn cstr(&self, s: &str) -> *const c_char {
        assert!(!s.contains('\0'), "CUI strings must not contain NUL bytes: {s:?}");
        let p = self.alloc(s.len() + 1, 1);
        // SAFETY: `p` has s.len() + 1 writable bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(s.as_ptr(), p, s.len());
            *p.add(s.len()) = 0;
        }
        p as *const c_char
    }

    /// Copies a `&[&str]` into an aligned `*const *const c_char` array.
    pub fn cstr_array(&self, items: &[&str]) -> *const *const c_char {
        assert!(!items.is_empty());
        let base = self.alloc(core::mem::size_of::<*const c_char>() * items.len(), core::mem::align_of::<*const c_char>())
            as *mut *const c_char;
        for (i, item) in items.iter().enumerate() {
            // Chunks never move, so `base` stays valid across these calls.
            // SAFETY: `i < items.len()`, inside the allocation above.
            unsafe { base.add(i).write(self.cstr(item)) };
        }
        base
    }

    /// Copies a slice of fixed-size C structs into an aligned array.
    pub fn struct_array<T: Copy>(&self, items: &[T]) -> *const T {
        assert!(!items.is_empty());
        let size = core::mem::size_of_val(items);
        let p = self.alloc(size, core::mem::align_of::<T>());
        // SAFETY: `p` has `size` writable bytes aligned for T.
        unsafe { core::ptr::copy_nonoverlapping(items.as_ptr() as *const u8, p, size) };
        p.cast::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::CStr;

    fn text(p: *const c_char) -> String {
        unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string()
    }

    #[test]
    fn cstr_round_trip() {
        let a = Arena::new();
        assert_eq!(text(a.cstr("Hello")), "Hello");
    }

    #[test]
    fn cstr_rejects_nul() {
        let a = Arena::new();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.cstr("a\0b"))).is_err());
    }

    #[test]
    fn array_is_aligned_and_nul_separated() {
        let a = Arena::new();
        a.cstr("x"); // shift the cursor so the array must re-align
        let items = ["All", "Open", "Done"];
        let p = a.cstr_array(&items);
        assert_eq!(p as usize % core::mem::align_of::<*const c_char>(), 0);
        for (i, want) in items.iter().enumerate() {
            assert_eq!(text(unsafe { *p.add(i) }), *want);
        }
    }

    #[test]
    fn pointers_survive_growth_past_many_chunks() {
        let a = Arena::new();
        let first = a.cstr("first");
        let mut kept = Vec::new();
        // ~1.5 MiB in 1 KiB strings: dozens of chunks
        for i in 0..1500 {
            let s = format!("{i:04}-{}", "x".repeat(1000));
            kept.push((a.cstr(&s), s));
        }
        assert_eq!(text(first), "first");
        for (p, s) in &kept {
            assert_eq!(&text(*p), s);
        }
    }

    #[test]
    fn single_request_larger_than_a_chunk() {
        let a = Arena::new();
        let before = a.cstr("before");
        let big = "y".repeat(CHUNK * 3 + 17);
        let p = a.cstr(&big);
        let after = a.cstr("after");
        assert_eq!(text(before), "before");
        assert_eq!(text(p), big);
        assert_eq!(text(after), "after");
    }

    #[test]
    fn cstr_array_with_more_text_than_a_chunk() {
        let a = Arena::new();
        // the strings alone overflow the chunk the pointer table was carved from
        let owned: Vec<String> = (0..200).map(|i| format!("{i}-{}", "z".repeat(1024))).collect();
        let items: Vec<&str> = owned.iter().map(String::as_str).collect();
        let p = a.cstr_array(&items);
        for (i, want) in owned.iter().enumerate() {
            assert_eq!(&text(unsafe { *p.add(i) }), want);
        }
    }

    #[test]
    fn struct_arrays_are_aligned_across_chunk_boundaries() {
        #[derive(Clone, Copy, PartialEq, Debug)]
        #[repr(C, align(16))]
        struct Wide([u64; 2]);
        let a = Arena::new();
        for round in 0..64u64 {
            a.cstr(&"p".repeat(1 + (round as usize * 977) % 5000)); // vary the cursor
            let items = [Wide([round, !round]); 7];
            let p = a.struct_array(&items);
            assert_eq!(p as usize % 16, 0);
            assert_eq!(unsafe { core::slice::from_raw_parts(p, 7) }, &items);
        }
    }

    #[test]
    fn a_steady_multi_chunk_frame_stops_allocating() {
        let mut a = Arena::new();
        let frame = |a: &Arena| {
            for _ in 0..300 {
                a.cstr(&"s".repeat(1000)); // ~300 KiB: several regular chunks
            }
        };
        frame(&a);
        assert!(unsafe { &*a.chunks.get() }.len() > 1);
        a.reset();
        let base = unsafe { &*a.chunks.get() }[0].ptr;
        for _ in 0..3 {
            frame(&a);
            assert_eq!(unsafe { &*a.chunks.get() }.len(), 1, "the frame should fit the chunk kept by reset");
            a.reset();
            assert_eq!(unsafe { &*a.chunks.get() }[0].ptr, base, "reset must keep the same chunk");
        }
    }

    #[test]
    fn a_single_chunk_frame_keeps_its_chunk() {
        let mut a = Arena::new();
        let base = unsafe { &*a.chunks.get() }[0].ptr;
        a.cstr("small");
        a.reset();
        let chunks = unsafe { &*a.chunks.get() };
        assert_eq!((chunks.len(), chunks[0].used, chunks[0].ptr), (1, 0, base));
    }

    #[test]
    fn reset_leaves_one_chunk_that_holds_the_whole_frame() {
        let mut a = Arena::new();
        a.cstr(&"a".repeat(CHUNK * 2));
        a.cstr("frame 1");
        a.reset();
        let chunks = unsafe { &*a.chunks.get() };
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].used, 0);
        assert!(chunks[0].cap >= CHUNK * 2);
        assert_eq!(text(a.cstr("frame 2")), "frame 2");
    }
}
