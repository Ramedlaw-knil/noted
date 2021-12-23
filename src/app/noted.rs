use eframe::{epi::App, egui::{CentralPanel, ScrollArea, TextEdit, Ui}};

use crate::syntax_highlighting::CodeTheme;

pub struct Noted {
    text: String
}

impl Noted {
    pub fn new() -> Noted {
        Noted {text: String::new()}
    }

    fn render_editor(&mut self, ui: &mut Ui) {
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let mut layout_job = crate::syntax_highlighting::highlight(ui.ctx(), &CodeTheme::default(), string, "rs");
            layout_job.wrap_width = wrap_width;

            ui.fonts().layout_job(layout_job)
        };

        ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                TextEdit::multiline(&mut self.text)
                .code_editor()
                .desired_rows(10)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
            );
        });
    }
}

impl App for Noted {
    fn name(&self) -> &str {
        "Noted"
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Welcome to Noted");
            
            
            self.render_editor(ui);
        });
    }
}