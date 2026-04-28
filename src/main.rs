mod app;
mod settings;
mod ui;
mod utils;

use app::App;
use eframe::egui;
use settings::load_settings;
use std::fs;

fn setup_fonts(ctx: &egui::Context) {
    #[cfg(target_os = "macos")]
    let candidates: &[&str] = &[
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc",
        "/System/Library/Fonts/Hiragino Sans GB W3.otf",
    ];
    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &[
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/YuGothR.ttc",
        "C:/Windows/Fonts/msgothic.ttc",
    ];
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let candidates: &[&str] = &[];

    let mut fonts = egui::FontDefinitions::default();
    for path in candidates {
        if let Ok(data) = fs::read(path) {
            fonts
                .font_data
                .insert("jp".to_owned(), egui::FontData::from_owned(data));
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .push("jp".to_owned());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Dataset Tag Editor",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            let native_ppp = cc.egui_ctx.pixels_per_point();
            let s = load_settings();
            cc.egui_ctx.set_pixels_per_point(native_ppp * s.zoom);
            let mut app = App {
                native_ppp,
                zoom: s.zoom,
                list_width: s.list_width,
                tag_width: s.tag_width,
                caption_height: s.caption_height,
                ..Default::default()
            };
            if let Some(dir) = s.last_dir {
                if dir.is_dir() {
                    app.load_dir(&dir, &cc.egui_ctx);
                }
            }
            Ok(Box::new(app))
        }),
    )
}
