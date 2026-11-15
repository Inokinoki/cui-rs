//! Compile-time coverage for `view!`: every function below must type-check
//! against the real method signatures. Nothing is run (a `Ui` needs a window).
#![allow(dead_code, unused_variables, unused_mut)]

use cui::{view, Align, ButtonVariant, Color, Size, Ui};

fn containers(ui: &mut Ui, open: &mut bool, ratio: &mut f32) {
    view! { ui,
        Column(pad: 8., gap: 4., width: Size::FILL) { Text("a"); }
        Column() { Spacer(); }
        Row(gap: 2.) { Separator(); }
        Box(gap: 2.) { Separator(); }
        Scroll(id: Some("s")) { Gap(4.); }
        let clicked = Card(hoverable: true) { Label("in card"); };
        Card() { Muted("x"); }
        Dialog("Title" => *open) { DialogFooter() { Button("ok"); } }
        Dialog("Title" => *open, width: 300.) { Text("body"); }
        Split("sp", ratio) { Text("l"); }
    }
}

fn text(ui: &mut Ui) {
    view! { ui,
        Text("a");
        Text("b", color: Color::BACKGROUND);
        Heading("h", 2);
        Label("l");
        Secondary("s");
        Muted("m");
        Code("c");
        Kbd("Ctrl+S");
        Link("link") { ui.toast("clicked"); }
        Icon(cui::ICON_X, 16., Color::BACKGROUND);
        Spacer();
        Gap(8.);
        Separator();
        Badge("new");
        Tooltip("tip");
        NavSection("Section");
    }
}

fn clicks(ui: &mut Ui, n: &mut i32) {
    view! { ui,
        Button("go") { *n += 1; }
        Button("go", variant: ButtonVariant::Primary) { *n += 1; }
        Button("plain");
        let b = Button("x", disabled: true);
        IconButton(cui::ICON_X) { *n += 1; }
        IconButton(cui::ICON_X, tooltip: Some("t")) { }
        NavItem("home") { *n = 0; }
        NavItem("home", selected: true);
        Toast("hi");
        let t = Toast("hi", description: Some("d"));
    }
}

fn changes(ui: &mut Ui, b: &mut bool, i: &mut i32, f: &mut f32, d: &mut f64) {
    let items = ["a", "b"];
    view! { ui,
        Checkbox("c" => *b) { *i += 1; }
        Switch("s" => *b);
        Switch("s" => *b, disabled: true) { }
        Chip("chip" => *b);
        Radio("r" => *i, 1);
        Radio("r" => *i, 1, disabled: false) { }
        Slider("v" => *f, 0., 1.) { }
        Slider("v" => *f, 0., 1., step: 0.1);
        Number("n" => *d, 0., 10.);
        Segmented("seg", &items => *i);
        Select("sel", &items => *i) { *f = 0.; }
        Tabs("tabs", &items => *i);
    }
}

fn inputs(ui: &mut Ui, s: &mut String) {
    view! { ui,
        let a = Input("a" => *s);
        let b = Input("b" => *s, placeholder: Some("p"), width: Size::FILL);
        let c = Textarea("c" => *s);
        let d = Search("d" => *s, placeholder: Some("q"));
        if a.submitted() || b.changed() { Toast("saved"); }
    }
}

fn flow(ui: &mut Ui, items: &[String], mode: i32, maybe: Option<i32>) {
    let mut count = 0;
    view! { ui,
        count += 1;
        let _x = Some(1);
        let r: Result<i32, ()> = Ok(2);
        if mode == 0 { Text("zero"); } else if mode == 1 { Text("one"); } else { Text("many"); }
        if let Some(n) = maybe { Text(format!("{n}")); }
        for (i, item) in items.iter().enumerate() {
            let _g = ui.push_id_int(i as i32);
            Row(gap: 8., align: Align::Center) {
                Text(item);
                if i > 0 { Button("up") { count += 1; } }
            }
        }
        match mode {
            0 => { Text("a"); }
            1 | 2 if count > 0 => Text("b"),
            _ => ui.text("raw call"),
        }
        while count < 3 { count += 1; }
        ui.text("bare call");
    }
}

fn custom_ui_name(app_ui: &mut Ui) {
    view! { app_ui,
        Column(gap: 1.) { Row(gap: 1.) { app_ui.text("inner handle"); } }
    }
}
