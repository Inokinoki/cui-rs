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
