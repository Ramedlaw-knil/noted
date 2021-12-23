use eframe::egui::{text::LayoutJob, TextEdit, TextStyle, Ui, Context, util, TextFormat, Style, Id, widgets, color_picker, Vec2, Frame, Button, Color32};

/// View some code with syntax highlighing and selection.
pub fn code_view_ui(ui: &mut Ui, mut code: &str) {
    let language = "rs";
    let theme = CodeTheme::from_memory(ui.ctx());

    let mut layouter = |ui: &Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string, language);
        // layout_job.wrap_width = wrap_width; // no wrapping
        ui.fonts().layout_job(layout_job)
    };

    ui.add(
        TextEdit::multiline(&mut code)
            .text_style(TextStyle::Monospace) // for cursor height
            .code_editor()
            .desired_rows(1)
            .lock_focus(true)
            .layouter(&mut layouter),
    );
}

/// Memoized Code highlighting
pub fn highlight(ctx: &Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    impl util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highligher {
        fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob {
            self.highlight(theme, code, lang)
        }
    }

    type HighlightCache<'a> = util::cache::FrameCache<LayoutJob, Highligher>;

    let mut memory = ctx.memory();
    let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
    highlight_cache.get((theme, code, language))
}

// ----------------------------------------------------------------------------

#[cfg(not(feature = "syntect"))]
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(enum_map::Enum)]
enum TokenType {
    Comment,
    Keyword,
    Literal,
    StringLiteral,
    Punctuation,
    Whitespace,
}

#[derive(Clone, Copy, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeTheme {
    dark_mode: bool,

    #[cfg(feature = "syntect")]
    syntect_theme: SyntectTheme,

    #[cfg(not(feature = "syntect"))]
    formats: enum_map::EnumMap<TokenType, TextFormat>,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn from_style(style: &Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }

    pub fn from_memory(ctx: &Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.memory()
                .data
                .get_persisted(Id::new("dark"))
                .unwrap_or_else(CodeTheme::dark)
        } else {
            ctx.memory()
                .data
                .get_persisted(Id::new("light"))
                .unwrap_or_else(CodeTheme::light)
        }
    }

    pub fn store_in_memory(&self, ctx: &Context) {
        if self.dark_mode {
            ctx.memory()
                .data
                .insert_persisted(Id::new("dark"), *self);
        } else {
            ctx.memory()
                .data
                .insert_persisted(Id::new("light"), *self);
        }
    }
}

#[cfg(not(feature = "syntect"))]
impl CodeTheme {
    pub fn dark() -> Self {
        let text_style = TextStyle::Monospace;
        use {Color32, TextFormat};
        Self {
            dark_mode: true,
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(text_style, Color32::from_gray(120)),
                TokenType::Keyword => TextFormat::simple(text_style, Color32::from_rgb(255, 100, 100)),
                TokenType::Literal => TextFormat::simple(text_style, Color32::from_rgb(87, 165, 171)),
                TokenType::StringLiteral => TextFormat::simple(text_style, Color32::from_rgb(109, 147, 226)),
                TokenType::Punctuation => TextFormat::simple(text_style, Color32::LIGHT_GRAY),
                TokenType::Whitespace => TextFormat::simple(text_style, Color32::TRANSPARENT),
            ],
        }
    }

    pub fn light() -> Self {
        let text_style = TextStyle::Monospace;
        use {Color32, TextFormat};
        Self {
            dark_mode: false,
            #[cfg(not(feature = "syntect"))]
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(text_style, Color32::GRAY),
                TokenType::Keyword => TextFormat::simple(text_style, Color32::from_rgb(235, 0, 0)),
                TokenType::Literal => TextFormat::simple(text_style, Color32::from_rgb(153, 134, 255)),
                TokenType::StringLiteral => TextFormat::simple(text_style, Color32::from_rgb(37, 203, 105)),
                TokenType::Punctuation => TextFormat::simple(text_style, Color32::DARK_GRAY),
                TokenType::Whitespace => TextFormat::simple(text_style, Color32::TRANSPARENT),
            ],
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            let selected_id = Id::null();
            let mut selected_tt: TokenType = *ui
                .memory()
                .data
                .get_persisted_mut_or(selected_id, TokenType::Comment);

            ui.vertical(|ui| {
                ui.set_width(150.0);
                widgets::global_dark_light_mode_buttons(ui);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.scope(|ui| {
                    for (tt, tt_name) in [
                        (TokenType::Comment, "// comment"),
                        (TokenType::Keyword, "keyword"),
                        (TokenType::Literal, "literal"),
                        (TokenType::StringLiteral, "\"string literal\""),
                        (TokenType::Punctuation, "punctuation ;"),
                        // (TokenType::Whitespace, "whitespace"),
                    ] {
                        let format = &mut self.formats[tt];
                        ui.style_mut().override_text_style = Some(format.style);
                        ui.visuals_mut().override_text_color = Some(format.color);
                        ui.radio_value(&mut selected_tt, tt, tt_name);
                    }
                });

                let reset_value = if self.dark_mode {
                    CodeTheme::dark()
                } else {
                    CodeTheme::light()
                };

                if ui
                    .add_enabled(*self != reset_value, Button::new("Reset theme"))
                    .clicked()
                {
                    *self = reset_value;
                }
            });

            ui.add_space(16.0);

            ui.memory().data.insert_persisted(selected_id, selected_tt);

            Frame::group(ui.style())
                .margin(Vec2::splat(2.0))
                .show(ui, |ui| {
                    // ui.group(|ui| {
                    ui.style_mut().override_text_style = Some(TextStyle::Small);
                    ui.spacing_mut().slider_width = 128.0; // Controls color picker size
                    widgets::color_picker::color_picker_color32(
                        ui,
                        &mut self.formats[selected_tt].color,
                        color_picker::Alpha::Opaque,
                    );
                });
        });
    }
}

#[cfg(not(feature = "syntect"))]
#[derive(Default)]
struct Highligher {}

#[cfg(not(feature = "syntect"))]
impl Highligher {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, mut text: &str, _language: &str) -> LayoutJob {
        // Extremely simple syntax highlighter for when we compile without syntect

        let mut job = LayoutJob::default();

        while !text.is_empty() {
            if text.starts_with("//") {
                let end = text.find('\n').unwrap_or_else(|| text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment]);
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..]
                    .find('"')
                    .map(|i| i + 2)
                    .or_else(|| text.find('\n'))
                    .unwrap_or_else(|| text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::StringLiteral]);
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_keyword(word) {
                    TokenType::Keyword
                } else {
                    TokenType::Literal
                };
                job.append(word, 0.0, theme.formats[tt]);
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Whitespace]);
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Punctuation]);
                text = &text[end..];
            }
        }

        job
    }
}

#[cfg(not(feature = "syntect"))]
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}