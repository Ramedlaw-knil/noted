mod app;
mod syntax_highlighting;

use eframe::{run_native, NativeOptions};
use app::noted::Noted;


fn main() {
    let app = Noted::new();
    let native_options = NativeOptions::default();
    run_native(Box::new(app), native_options)
}
