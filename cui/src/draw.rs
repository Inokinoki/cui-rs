//! Custom painting: [`Ui::canvas`](crate::Ui::canvas) and the [`CanvasUi`] draw surface, plus
//! per-widget persistent state (C's `cui_state`).

use crate::opts2::Corners;
use crate::types::*;
use cui_sys as ffi;
use core::ffi::{c_char, c_void};
use core::marker::PhantomData;

/// What a [`Ui::canvas`](crate::Ui::canvas) reports this frame (C's `cui_canvas_result`).
#[derive(Clone, Copy, Debug, Default)]
pub struct CanvasResult {
    /// Where the canvas landed on the screen.
    pub rect: Rect,
    /// The pointer is over it.
    pub hovered: bool,
    /// The pointer is pressed on it.
    pub pressed: bool,
    /// It was clicked this frame.
    pub clicked: bool,
    /// The pointer's position, relative to the canvas.
    pub mouse: Vec2,
}

impl CanvasResult {
    fn from_ffi(r: ffi::cui_canvas_result) -> CanvasResult {
        CanvasResult {
            rect: Rect { x: r.rect.x, y: r.rect.y, w: r.rect.w, h: r.rect.h },
            hovered: r.hovered,
            pressed: r.pressed,
            clicked: r.clicked,
            mouse: Vec2 { x: r.mouse.x, y: r.mouse.y },
        }
    }
}

/// A paint-only handle: the drawing surface a [`Ui::canvas`](crate::Ui::canvas) closure draws
/// with. Only CUI's immediate draw calls live here — building widgets inside
/// a paint callback is not supported (C calls the paint function after
/// layout, outside the widget pass).
pub struct CanvasUi<'a> {
    ctx: *mut ffi::cui_ctx,
    _frame: PhantomData<&'a ()>,
}

impl Token {
    /// The token as an unresolved semantic color, for [`crate::Color`]
    /// pipelines that resolve later.
    pub fn color(self) -> Color {
        Color { r: self as i32 as u8, g: 0xC5, b: 0x1D, a: 0 }
    }
}

impl<'a> CanvasUi<'a> {
    fn new(ctx: *mut ffi::cui_ctx) -> CanvasUi<'a> {
        CanvasUi { ctx, _frame: PhantomData }
    }

    /// A filled rectangle with per-corner radii (C's `cui_draw_rect`).
    pub fn draw_rect(&mut self, r: Rect, radius: Corners, fill: Color) {
        unsafe {
            ffi::cui_draw_rect(
                self.ctx,
                ffi::cui_rect { x: r.x, y: r.y, w: r.w, h: r.h },
                radius.to_ffi(),
                color_to_ffi(fill),
            )
        }
    }

    /// A soft shadow behind a rectangle (C's `cui_draw_shadow`).
    pub fn draw_shadow(&mut self, r: Rect, radius: Corners, blur: f32, spread: f32, offset: Vec2, color: Color) {
        unsafe {
            ffi::cui_draw_shadow(
                self.ctx,
                ffi::cui_rect { x: r.x, y: r.y, w: r.w, h: r.h },
                radius.to_ffi(),
                blur,
                spread,
                ffi::cui_vec2 { x: offset.x, y: offset.y },
                color_to_ffi(color),
            )
        }
    }

    /// A rectangle filled with a vertical gradient between two colors
    /// (C's `cui_draw_rect_gradient`).
    pub fn draw_rect_gradient(&mut self, r: Rect, radius: Corners, from: Color, to: Color) {
        unsafe {
            ffi::cui_draw_rect_gradient(
                self.ctx,
                ffi::cui_rect { x: r.x, y: r.y, w: r.w, h: r.h },
                radius.to_ffi(),
                color_to_ffi(from),
                color_to_ffi(to),
            )
        }
    }

    /// A rectangle with an independent border color (C's `cui_draw_rect_border`).
    pub fn draw_rect_border(&mut self, r: Rect, radius: Corners, fill: Color, border: Color, width: f32) {
        unsafe {
            ffi::cui_draw_rect_border(
                self.ctx,
                ffi::cui_rect { x: r.x, y: r.y, w: r.w, h: r.h },
                radius.to_ffi(),
                color_to_ffi(fill),
                color_to_ffi(border),
                width,
            )
        }
    }

    /// A straight line (C's `cui_draw_line`).
    pub fn draw_line(&mut self, a: Vec2, b: Vec2, thickness: f32, color: Color) {
        unsafe {
            ffi::cui_draw_line(
                self.ctx,
                ffi::cui_vec2 { x: a.x, y: a.y },
                ffi::cui_vec2 { x: b.x, y: b.y },
                thickness,
                color_to_ffi(color),
            )
        }
    }

    /// A convex polygon (C's `cui_draw_polygon`).
    pub fn draw_polygon(&mut self, points: &[Vec2], color: Color) {
        unsafe {
            ffi::cui_draw_polygon(
                self.ctx,
                points.as_ptr() as *const ffi::cui_vec2,
                points.len() as i32,
                color_to_ffi(color),
            )
        }
    }

    /// A filled circle (C's `cui_draw_circle`).
    pub fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color) {
        unsafe { ffi::cui_draw_circle(self.ctx, ffi::cui_vec2 { x: center.x, y: center.y }, radius, color_to_ffi(color)) }
    }

    /// A ring segment from `a0` to `a1` radians (C's `cui_draw_arc`).
    pub fn draw_arc(&mut self, center: Vec2, radius: f32, a0: f32, a1: f32, thickness: f32, color: Color) {
        unsafe {
            ffi::cui_draw_arc(
                self.ctx,
                ffi::cui_vec2 { x: center.x, y: center.y },
                radius,
                a0,
                a1,
                thickness,
                color_to_ffi(color),
            )
        }
    }

    /// Raw text. `len` is the byte count, or -1 for the whole string
    /// (C's `cui_draw_text`).
    pub fn draw_text(&mut self, font: i32, size: f32, pos: Vec2, color: Color, text: &str) {
        let t = std::ffi::CString::new(text).expect("text must not contain NUL");
        unsafe {
            ffi::cui_draw_text(
                self.ctx,
                font,
                size,
                ffi::cui_vec2 { x: pos.x, y: pos.y },
                color_to_ffi(color),
                t.as_ptr() as *const c_char,
                -1,
            )
        }
    }

    /// An icon inside a rectangle (C's `cui_draw_icon`).
    pub fn draw_icon(&mut self, icon: &str, r: Rect, size: f32, color: Color) {
        let i = std::ffi::CString::new(icon).expect("icon must not contain NUL");
        unsafe {
            ffi::cui_draw_icon(
                self.ctx,
                i.as_ptr() as *const c_char,
                ffi::cui_rect { x: r.x, y: r.y, w: r.w, h: r.h },
                size,
                color_to_ffi(color),
            )
        }
    }

    /// Clips subsequent drawing to a rectangle, until [`CanvasUi::pop_clip`]
    /// (C's `cui_push_clip`).
    pub fn push_clip(&mut self, r: Rect) {
        unsafe { ffi::cui_push_clip(self.ctx, ffi::cui_rect { x: r.x, y: r.y, w: r.w, h: r.h }) }
    }

    /// Ends the clip started by [`CanvasUi::push_clip`] (C's `cui_pop_clip`).
    pub fn pop_clip(&mut self) {
        unsafe { ffi::cui_pop_clip(self.ctx) }
    }

    /// Resolves a semantic color (a token) against the active theme
    /// (C's `cui_color_of`).
    pub fn color_of(&mut self, c: Color) -> Color {
        color_from_ffi(unsafe { ffi::cui_color_of(self.ctx, color_to_ffi(c)) })
    }

    /// The color of a semantic token (C's `cui_token_color`).
    pub fn token_color(&mut self, token: Token) -> Color {
        color_from_ffi(unsafe { ffi::cui_token_color(self.ctx, token as i32) })
    }
}

/// Semantic color tokens, for [`CanvasUi::token_color`] (C's `cui_token`).
#[allow(missing_docs)] // variant names say what they mean
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Token {
    Background = ffi::CUI_TOKEN_BACKGROUND,
    Surface = ffi::CUI_TOKEN_SURFACE,
    SurfaceRaised = ffi::CUI_TOKEN_SURFACE_RAISED,
    SurfaceMuted = ffi::CUI_TOKEN_SURFACE_MUTED,
    Foreground = ffi::CUI_TOKEN_FOREGROUND,
    TextSecondary = ffi::CUI_TOKEN_TEXT_SECONDARY,
    TextMuted = ffi::CUI_TOKEN_TEXT_MUTED,
    Border = ffi::CUI_TOKEN_BORDER,
    BorderSubtle = ffi::CUI_TOKEN_BORDER_SUBTLE,
    BorderStrong = ffi::CUI_TOKEN_BORDER_STRONG,
    Accent = ffi::CUI_TOKEN_ACCENT,
    AccentStrong = ffi::CUI_TOKEN_ACCENT_STRONG,
    AccentSubtle = ffi::CUI_TOKEN_ACCENT_SUBTLE,
    AccentForeground = ffi::CUI_TOKEN_ACCENT_FOREGROUND,
    Success = ffi::CUI_TOKEN_SUCCESS,
    Warning = ffi::CUI_TOKEN_WARNING,
    Danger = ffi::CUI_TOKEN_DANGER,
    Info = ffi::CUI_TOKEN_INFO,
    Hover = ffi::CUI_TOKEN_HOVER,
    Active = ffi::CUI_TOKEN_ACTIVE,
    Scrim = ffi::CUI_TOKEN_SCRIM,
    FocusRing = ffi::CUI_TOKEN_FOCUS_RING,
}

impl<'a> crate::Ui<'a> {
    /// A custom-painted area (C's `cui_canvas`). The `paint` function runs
    /// after layout with the surface to draw on and the (copied) `data`.
    /// `T` must be plain copyable data, as CUI snapshots it, and `'static`:
    /// `paint` runs when the frame is rendered, inside the next
    /// [`App::frame`](crate::App::frame), after anything `data` could borrow
    /// from this frame may be gone. Copy what the painter needs into `T`
    /// (an array, indices into state the painter can reach).
    ///
    /// # Panics
    ///
    /// When `T` is over-aligned (more than 16 bytes) for CUI's snapshot.
    ///
    /// Borrowed data does not compile:
    ///
    /// ```compile_fail
    /// # use cui::{CanvasUi, Rect, Size, Ui, Vec2};
    /// fn paint(_c: &mut CanvasUi, _r: Rect, _pts: &&[Vec2]) {}
    /// fn plot(ui: &mut Ui) {
    ///     let pts = vec![Vec2 { x: 0., y: 0. }; 8];
    ///     ui.canvas("plot", Size::FILL, Size::FILL, paint, &pts.as_slice());
    /// }
    /// ```
    ///
    /// Returns what happened this frame: clicks, hover and where the canvas
    /// landed. A common companion is [`Ui::state`](crate::Ui::state), which keeps per-canvas
    /// state (like a hover flag) between frames.
    pub fn canvas<T: Copy + 'static>(
        &mut self,
        id: impl AsRef<str>,
        width: Size,
        height: Size,
        paint: fn(&mut CanvasUi, Rect, &T),
        data: &T,
    ) -> CanvasResult {
        struct Job<T: Copy> {
            paint: fn(&mut CanvasUi, Rect, &T),
            data: T,
        }

        extern "C" fn trampoline<T: Copy>(ui: *mut ffi::cui_ctx, rect: ffi::cui_rect, user: *mut c_void) {
            let job = unsafe { &*(user as *const Job<T>) };
            let mut surface = CanvasUi::new(ui);
            (job.paint)(
                &mut surface,
                Rect { x: rect.x, y: rect.y, w: rect.w, h: rect.h },
                &job.data,
            );
        }

        // C copies the job into its frame arena, which aligns to 16 bytes.
        assert!(core::mem::align_of::<Job<T>>() <= 16, "Ui::canvas: the data type is over-aligned for CUI's frame arena");
        let job = Job { paint, data: *data };
        let res = unsafe {
            ffi::cui_canvas(
                self.ctx,
                self.cs(&id),
                size_to_ffi(width),
                size_to_ffi(height),
                Some(trampoline::<T> as extern "C" fn(*mut ffi::cui_ctx, ffi::cui_rect, *mut c_void)),
                &job as *const Job<T> as *const c_void,
                core::mem::size_of::<Job<T>>(),
            )
        };
        CanvasResult::from_ffi(res)
    }

    /// Persistent per-widget storage keyed by an id (C's `cui_state`).
    ///
    /// The value starts as `T::default()` and CUI garbage-collects it (the
    /// next use starts from `T::default()` again) after roughly five seconds
    /// of not being asked for. The reference borrows `self`, so it cannot be
    /// held across another `Ui` call or alias a second `state` borrow.
    ///
    /// # Panics
    ///
    /// When `id` already holds a value of a different type `T`, or `T` is
    /// over-aligned for C's allocator. (The id space is shared with CUI's
    /// own widgets: use hashes of your own names, not small integers.)
    pub fn state<T: Copy + Default + 'static>(&mut self, id: u32) -> &mut T {
        assert!(
            core::mem::align_of::<StateSlot<T>>() <= 16,
            "Ui::state: the type is over-aligned for C's allocator"
        );
        let slot = unsafe { ffi::cui_state(self.ctx, id, core::mem::size_of::<StateSlot<T>>()) } as *mut StateSlot<T>;
        assert!(!slot.is_null() && slot.is_aligned());
        let tag = type_tag::<T>();
        // SAFETY: `slot` is C-owned memory of at least `size_of::<StateSlot<T>>()`
        // bytes, aligned (checked above), and zero-filled when fresh, so the
        // tag reads as 0 then. No reference into it survives past the borrow
        // of `self`; C only reallocates it on a later `cui_state` call.
        unsafe {
            match (*slot).tag {
                0 => slot.write(StateSlot { tag, value: T::default() }),
                t if t == tag => {}
                _ => panic!(
                    "Ui::state: id {id:#x} already holds a value of another type (or collides with a CUI widget's own state)"
                ),
            }
            &mut (*slot).value
        }
    }
}

/// What `Ui::state` keeps in C's allocation: a hash of the value's type
/// (0 = fresh, as C zeroes new memory) in front of the value itself.
#[repr(C)]
struct StateSlot<T> {
    tag: u64,
    value: T,
}

fn type_tag<T: 'static>() -> u64 {
    use core::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    core::any::TypeId::of::<T>().hash(&mut h);
    h.finish() | 1 // never 0
}

#[cfg(test)]
mod state_tests {
    use crate::arena::Arena;
    use crate::Ui;
    use cui_sys as ffi;

    fn with_ui(f: impl FnOnce(&mut Ui)) {
        let desc = ffi::cui_ctx_desc { no_default_fonts: true, ..Default::default() };
        let ctx = unsafe { ffi::cui_create(&desc) };
        assert!(!ctx.is_null());
        let arena = Arena::new();
        f(&mut Ui::new(ctx, &arena));
        unsafe { ffi::cui_destroy(ctx) };
    }

    #[test]
    fn starts_at_default_and_persists() {
        with_ui(|ui| {
            assert_eq!(*ui.state::<u32>(7), 0);
            *ui.state::<u32>(7) = 41;
            *ui.state::<u32>(7) += 1;
            assert_eq!(*ui.state::<u32>(7), 42);
        });
    }

    #[test]
    fn ids_are_independent_and_default_is_not_zero_bits() {
        #[derive(Clone, Copy, Default, PartialEq, Debug)]
        struct Pair {
            label: &'static str, // all-zero bits would be an invalid &str
            n: i32,
        }
        with_ui(|ui| {
            *ui.state::<bool>(1) = true;
            assert!(!*ui.state::<bool>(2));
            let p = *ui.state::<Pair>(3);
            assert_eq!(p, Pair { label: "", n: 0 });
        });
    }

    #[test]
    fn a_different_type_under_the_same_id_panics() {
        with_ui(|ui| {
            *ui.state::<u8>(9) = 5;
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // would otherwise read 5u8 back as a bool: not a valid bool
                let _ = *ui.state::<bool>(9);
            }));
            assert!(r.is_err());
            assert_eq!(*ui.state::<u8>(9), 5);
        });
    }
}
