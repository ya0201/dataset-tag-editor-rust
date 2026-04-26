use eframe::egui;
use std::{fs, path::{Path, PathBuf}};

struct Entry {
    image_path: PathBuf,
    caption_path: PathBuf,
}

#[derive(Default)]
struct App {
    entries: Vec<Entry>,
    current: usize,
    caption: String,
    texture: Option<egui::TextureHandle>,
    dirty: bool,
}

impl App {
    fn load_dir(&mut self, dir: &Path, ctx: &egui::Context) {
        self.save_if_dirty();
        let mut paths: Vec<PathBuf> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("jpg" | "jpeg" | "png")))
            .collect();
        paths.sort();
        self.entries = paths.into_iter().map(|img| {
            let stem = img.file_stem().unwrap().to_string_lossy().into_owned();
            let parent = img.parent().unwrap().to_path_buf();
            let caption = [".txt", ".caption"]
                .iter()
                .map(|ext| parent.join(format!("{stem}{ext}")))
                .find(|p| p.exists())
                .unwrap_or_else(|| parent.join(format!("{stem}.txt")));
            Entry { image_path: img, caption_path: caption }
        }).collect();
        self.current = 0;
        self.load_entry(ctx);
    }

    fn load_entry(&mut self, ctx: &egui::Context) {
        self.caption.clear();
        self.texture = None;
        if let Some(entry) = self.entries.get(self.current) {
            self.caption = fs::read_to_string(&entry.caption_path).unwrap_or_default();
            self.texture = load_texture(ctx, &entry.image_path);
        }
        self.dirty = false;
    }

    fn save_if_dirty(&mut self) {
        if !self.dirty { return; }
        if let Some(entry) = self.entries.get(self.current) {
            let _ = fs::write(&entry.caption_path, &self.caption);
        }
        self.dirty = false;
    }

    fn go_to(&mut self, index: usize, ctx: &egui::Context) {
        self.save_if_dirty();
        self.current = index;
        self.load_entry(ctx);
    }

    fn navigate(&mut self, delta: i32, ctx: &egui::Context) {
        let n = self.entries.len();
        if n == 0 { return; }
        let next = ((self.current as i32 + delta).rem_euclid(n as i32)) as usize;
        self.go_to(next, ctx);
    }
}

fn load_texture(ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    let img = image::open(path).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
    Some(ctx.load_texture("image", color_image, egui::TextureOptions::LINEAR))
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !ctx.wants_keyboard_input() {
            let delta = ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowLeft) { -1i32 }
                else if i.key_pressed(egui::Key::ArrowRight) { 1 }
                else { 0 }
            });
            if delta != 0 { self.navigate(delta, ctx); }
        }

        let n = self.entries.len();

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open Directory").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.load_dir(&dir, ctx);
                    }
                }
                if n > 0 {
                    ui.separator();
                    if ui.button("◀ Prev").clicked() { self.navigate(-1, ctx); }
                    if ui.button("Next ▶").clicked() { self.navigate(1, ctx); }
                    let name = self.entries[self.current].image_path.file_name()
                        .unwrap().to_string_lossy().to_string();
                    ui.label(format!("{}/{}: {name}", self.current + 1, n));
                    if self.dirty && ui.button("Save").clicked() {
                        self.save_if_dirty();
                    }
                }
            });
        });

        egui::SidePanel::left("list").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in 0..n {
                    let name = self.entries[i].image_path.file_name()
                        .unwrap().to_string_lossy().to_string();
                    if ui.selectable_label(i == self.current, name).clicked() && i != self.current {
                        self.go_to(i, ctx);
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("caption").resizable(true).show(ctx, |ui| {
            ui.label("Caption:");
            let r = ui.add(
                egui::TextEdit::multiline(&mut self.caption)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3),
            );
            if r.changed() { self.dirty = true; }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.texture {
                Some(tex) => {
                    let avail = ui.available_size();
                    let img_size = tex.size_vec2();
                    let scale = (avail.x / img_size.x).min(avail.y / img_size.y).min(1.0);
                    ui.centered_and_justified(|ui| {
                        ui.image((tex.id(), img_size * scale));
                    });
                }
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.label("ディレクトリを開いてください");
                    });
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Dataset Tag Editor",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
