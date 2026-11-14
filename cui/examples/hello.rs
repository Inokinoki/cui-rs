//! The smallest CUI app — a 1:1 port of examples/hello.c from the C repo.

use cui::{App, AppDesc, Button, ButtonVariant, Column, Row, Size};

fn main() {
    let mut app = App::new(&AppDesc { title: Some("Hello"), width: 420, height: 300, ..Default::default() })
        .expect("could not create a window");
    let mut notifications = true;
    let mut clicks = 0;

    while app.frame() {
        let mut ui = app.ui();
        ui.column(&Column { pad: 32., gap: 16., width: Size::FILL, ..Default::default() }, |ui| {
            ui.heading("Hello, CUI", 2);
            ui.secondary("A native window, a few components and no dependencies.");
            ui.switch("Notifications", &mut notifications);
            ui.row(&Row { gap: 8., ..Default::default() }, |ui| {
                if ui.button_with("Click me", &Button { variant: ButtonVariant::Primary, ..Default::default() }) {
                    clicks += 1;
                }
                ui.text(format!("Clicked {clicks} times"));
            });
        });
    }
}
