use eframe::egui;
use std::path::Path;

pub fn tag_color(tag: &str) -> egui::Color32 {
    const COLORS: &[egui::Color32] = &[
        egui::Color32::from_rgb(99, 102, 241),
        egui::Color32::from_rgb(139, 92, 246),
        egui::Color32::from_rgb(59, 130, 246),
        egui::Color32::from_rgb(16, 185, 129),
        egui::Color32::from_rgb(245, 158, 11),
        egui::Color32::from_rgb(236, 72, 153),
        egui::Color32::from_rgb(20, 184, 166),
        egui::Color32::from_rgb(249, 115, 22),
    ];
    let hash = tag
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    COLORS[hash as usize % COLORS.len()]
}

pub fn load_thumbnail(ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    let img = image::open(path).ok()?.thumbnail(80, 80).to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
    Some(ctx.load_texture(
        format!("thumb:{}", path.to_string_lossy()),
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}

pub fn load_texture(ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    let img = image::open(path).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
    Some(ctx.load_texture("image", color_image, egui::TextureOptions::LINEAR))
}

pub fn show_overlay(ctx: &egui::Context) {
    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new("modal_overlay"))
        .fixed_pos(screen.min)
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                screen,
                egui::Rounding::ZERO,
                egui::Color32::from_black_alpha(160),
            );
            ui.allocate_rect(screen, egui::Sense::click());
        });
}
