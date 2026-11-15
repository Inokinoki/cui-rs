//! Opts types under the names `view!` uses.
//!
//! `view!` turns `Switch("Notifications" => on, tooltip: Some("x"))` into
//! `ui.switch_with(.., &cui::view::Switch { tooltip: Some("x"), ..Default::default() })`.
//! Every opts struct is re-exported here under its own name; the aliases below
//! cover the widgets whose method shares an opts struct with another.

pub use crate::gen_opts::*;
pub use crate::opts::*;
pub use crate::opts2::*;

/// Options of `Box`: [`BoxOpts`].
pub type Box<'a> = BoxOpts<'a>;
/// Options of `Switch`: [`Check`].
pub type Switch<'a> = Check<'a>;
/// Options of `Checkbox`: [`Check`].
pub type Checkbox<'a> = Check<'a>;
/// Options of `Radio`: [`Check`].
pub type Radio<'a> = Check<'a>;
/// Options of `Textarea`: [`Input`].
pub type Textarea<'a> = Input<'a>;
/// Options of `Search`: [`Input`].
pub type Search<'a> = Input<'a>;
/// Options of `IconButton`: [`Button`].
pub type IconButton<'a> = Button<'a>;
/// Options of `NavItem`: [`Nav`].
pub type NavItem<'a> = Nav<'a>;
/// Options of `LineChart`: [`Chart`].
pub type LineChart<'a> = Chart<'a>;
/// Options of `BarChart`: [`Chart`].
pub type BarChart<'a> = Chart<'a>;
/// Options of `DonutChart`: [`Chart`].
pub type DonutChart<'a> = Chart<'a>;
/// Options of `Typewriter`: [`TextFx`].
pub type Typewriter = TextFx;
/// Options of `NumberRoll`: [`TextFx`].
pub type NumberRoll = TextFx;
/// Options of `WordRotate`: [`TextFx`].
pub type WordRotate = TextFx;
/// Options of `ChatComposer`: [`Composer`].
pub type ChatComposer<'a> = Composer<'a>;
/// Options of `DataGrid`: [`Grid`].
pub type DataGrid<'a> = Grid<'a>;
/// Options of `HexEdit`: [`HexView`].
pub type HexEdit<'a> = HexView<'a>;
pub use crate::ext::EmojiPicker;

#[cfg(test)]
mod tests {
    //! What the generated `flatten` methods hand to C: counts that match the
    //! slices, NULL for "none", and write-back pointers that point at the
    //! caller's variables. None of this needs a window.
    use super::*;
    use crate::{Arena, Date, Series, Tone};
    use core::ffi::CStr;

    #[test]
    fn counted_arrays_carry_their_length() {
        let a = Arena::new();
        let ticks = [-60.0_f32, -12., 0.];
        let f = Fader { ticks: &ticks, meter: &[0.5, 0.25], label: Some("Gain"), ..Default::default() }.flatten(&a);
        assert_eq!((f.tick_count, f.meter_channels), (3, 2));
        // copied into the frame arena, so C may read them after this call returns
        assert_ne!(f.ticks, ticks.as_ptr());
        assert_eq!(unsafe { core::slice::from_raw_parts(f.ticks, 3) }, &ticks);
        assert_eq!(unsafe { CStr::from_ptr(f.label) }.to_str(), Ok("Gain"));
    }

    #[test]
    fn empty_arrays_and_unset_options_are_null() {
        let a = Arena::new();
        let f = Fader::default().flatten(&a);
        assert!(f.ticks.is_null() && f.meter.is_null() && f.label.is_null());
        assert_eq!((f.tick_count, f.meter_channels), (0, 0));
        let m = Month::default().flatten(&a);
        assert!(m.selected.is_null() && m.month_names.is_null() && m.day_names.is_null());
        let h = HexView::default().flatten(&a);
        assert!(h.ranges.is_null() && h.edit.is_null());
        let t = TreeTable::default().flatten(&a);
        assert!(t.label.is_none() && t.user.is_null() && t.sort.is_null());
    }

    #[test]
    fn element_structs_are_flattened() {
        let a = Arena::new();
        let ranges = [HexRange { offset: 4, length: 2, tone: Tone::Danger, label: Some("crc") }];
        let h = HexView { base: 0x1000, ranges: &ranges, ..Default::default() }.flatten(&a);
        assert_eq!((h.base, h.range_count), (0x1000, 1));
        let r = unsafe { &*h.ranges };
        assert_eq!((r.offset, r.length, r.tone), (4, 2, Tone::Danger as i32));
        assert_eq!(unsafe { CStr::from_ptr(r.label) }.to_str(), Ok("crc"));

        let values = [1.0_f32, 2.0];
        let overlays = [Series { name: Some("MA"), values: &values, ..Default::default() }];
        let c = Candlestick { labels: &["Mon", "Tue"], overlays: &overlays, ..Default::default() }.flatten(&a);
        assert_eq!(c.overlay_count, 1);
        assert_eq!(unsafe { (*c.overlays).count }, 2);
        assert_eq!(unsafe { CStr::from_ptr(*c.labels.add(1)) }.to_str(), Ok("Tue"));
    }

    #[test]
    fn fixed_length_names() {
        let a = Arena::new();
        let days = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
        let m = Month { day_names: Some(&days), ..Default::default() }.flatten(&a);
        assert_eq!(unsafe { CStr::from_ptr(*m.day_names.add(6)) }.to_str(), Ok("Su"));
    }

    #[test]
    fn write_back_pointers_point_at_the_callers_variables() {
        let a = Arena::new();
        let mut sel = 3;
        let g = CommitGraph { selected: Some(&mut sel), ..Default::default() }.flatten(&a);
        unsafe { *g.selected = 7 }; // what C does on a click
        assert_eq!(sel, 7);

        let mut day = Date { year: 2026, month: 1, day: 2 };
        let m = Month { selected: Some(&mut day), ..Default::default() }.flatten(&a);
        unsafe { (*m.selected).day = 9 };
        assert_eq!(day.day, 9);

        let mut rows = [false, true];
        let mut cursor = 0;
        let g = Grid { selected: Some(&mut rows), cursor: Some(&mut cursor), ..Default::default() }.flatten(&a);
        unsafe { *g.selected = true };
        unsafe { *g.cursor = 1 };
        assert_eq!((rows, cursor), ([true, true], 1));
    }

    #[test]
    fn mnemonic_is_an_ascii_char() {
        let a = Arena::new();
        assert_eq!(MenuBarMenu { mnemonic: 'F', ..Default::default() }.flatten(&a).mnemonic, b'F' as _);
        assert_eq!(MenuBarMenu::default().flatten(&a).mnemonic, 0);
    }

    #[test]
    #[should_panic(expected = "ASCII")]
    fn a_non_ascii_mnemonic_is_refused() {
        MenuBarMenu { mnemonic: 'é', ..Default::default() }.flatten(&Arena::new());
    }
}
