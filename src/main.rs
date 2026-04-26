use eframe::egui;
use std::{collections::HashMap, fs, path::{Path, PathBuf}};

struct Entry {
    image_path: PathBuf,
    caption_path: PathBuf,
    thumbnail: Option<egui::TextureHandle>,
}

#[derive(Default)]
struct App {
    entries: Vec<Entry>,
    current: usize,
    caption: String,
    texture: Option<egui::TextureHandle>,
    dirty: bool,
    tag_counts: Vec<(String, usize)>,
    add_tag_input: String,
    drag_idx: Option<usize>,
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
            let thumbnail = load_thumbnail(ctx, &img);
            Entry { image_path: img, caption_path: caption, thumbnail }
        }).collect();
        self.current = 0;
        self.load_entry(ctx);
        self.rebuild_tag_counts();
    }

    fn rebuild_tag_counts(&mut self) {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for entry in &self.entries {
            let text = fs::read_to_string(&entry.caption_path).unwrap_or_default();
            for tag in text.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    *counts.entry(tag.to_string()).or_insert(0) += 1;
                }
            }
        }
        let mut v: Vec<(String, usize)> = counts.into_iter().collect();
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        self.tag_counts = v;
    }

    fn load_entry(&mut self, ctx: &egui::Context) {
        self.caption.clear();
        self.texture = None;
        self.drag_idx = None;
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
        self.rebuild_tag_counts();
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

fn tag_color(tag: &str) -> egui::Color32 {
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
    let hash = tag.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    COLORS[hash as usize % COLORS.len()]
}

fn load_thumbnail(ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    let img = image::open(path).ok()?.thumbnail(80, 80).to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
    Some(ctx.load_texture(format!("thumb:{}", path.to_string_lossy()), color_image, egui::TextureOptions::LINEAR))
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

        egui::SidePanel::left("list").min_width(100.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in 0..n {
                    let (thumb_info, name) = {
                        let e = &self.entries[i];
                        let info = e.thumbnail.as_ref().map(|t| (t.id(), t.size()));
                        let name = e.image_path.file_name().unwrap().to_string_lossy().to_string();
                        (info, name)
                    };
                    let is_selected = i == self.current;
                    ui.horizontal(|ui| {
                        let thumb_clicked = if let Some((id, [w, h])) = thumb_info {
                            let th = 64.0f32;
                            let tw = w as f32 * th / h as f32;
                            ui.add(egui::Image::new((id, egui::vec2(tw, th)))
                                .sense(egui::Sense::click()))
                                .clicked()
                        } else { false };
                        let label_clicked = ui.selectable_label(is_selected, &name).clicked();
                        if (thumb_clicked || label_clicked) && !is_selected {
                            self.go_to(i, ctx);
                        }
                    });
                    ui.separator();
                }
            });
        });

        egui::SidePanel::right("tag_counts").min_width(160.0).show(ctx, |ui| {
            ui.label(format!("Tags ({})", self.tag_counts.len()));
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("tag_grid").num_columns(2).striped(true).show(ui, |ui| {
                    for (tag, count) in &self.tag_counts {
                        ui.label(tag.as_str());
                        ui.label(count.to_string());
                        ui.end_row();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("caption").resizable(true).min_height(120.0).show(ctx, |ui| {
            let tags: Vec<String> = self.caption.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            let mut remove_idx: Option<usize> = None;
            let mut new_drag_idx = self.drag_idx;
            let mut drop_target: Option<usize> = None;
            let released = ctx.input(|i| i.pointer.primary_released());

            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 36.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        for (i, tag) in tags.iter().enumerate() {
                            let being_dragged = self.drag_idx == Some(i);
                            let base = tag_color(tag);
                            let fill = if being_dragged {
                                egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 100)
                            } else {
                                base
                            };
                            let inner = egui::Frame::none()
                                .fill(fill)
                                .rounding(egui::Rounding::same(8.0))
                                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        ui.label(egui::RichText::new(tag.as_str()).color(egui::Color32::WHITE));
                                        if !being_dragged {
                                            if ui.add(egui::Label::new(
                                                egui::RichText::new("⊗").color(egui::Color32::WHITE)
                                            ).sense(egui::Sense::click())).clicked() {
                                                remove_idx = Some(i);
                                            }
                                        }
                                    });
                                });
                            let chip_resp = ui.interact(
                                inner.response.rect,
                                egui::Id::new("chip").with(i),
                                egui::Sense::drag(),
                            );
                            if chip_resp.drag_started() {
                                new_drag_idx = Some(i);
                            }
                            // ポインタ位置で直接ホバー判定（drag中はhovered()が抑制されるため）
                            let hovered = ctx.input(|inp| {
                                inp.pointer.hover_pos().is_some_and(|p| inner.response.rect.contains(p))
                            });
                            if being_dragged {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                            } else if self.drag_idx.is_none() && hovered {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                            }
                            if self.drag_idx.is_some() && !being_dragged && hovered {
                                drop_target = Some(i);
                                ui.painter().rect_stroke(
                                    inner.response.rect.expand(2.0),
                                    egui::Rounding::same(9.0),
                                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                                );
                            }
                        }
                    });
                });

            if let Some(i) = remove_idx {
                self.caption = tags.iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.dirty = true;
            }
            if released {
                if let (Some(src), Some(dst)) = (self.drag_idx, drop_target) {
                    let mut v: Vec<&str> = tags.iter().map(String::as_str).collect();
                    let item = v.remove(src);
                    v.insert(dst, item);
                    self.caption = v.join(", ");
                    self.dirty = true;
                }
                new_drag_idx = None;
            }
            self.drag_idx = new_drag_idx;

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("add tag");
                let r = ui.add(egui::TextEdit::singleline(&mut self.add_tag_input).desired_width(300.0));
                let enter = r.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("Insert").clicked() || enter {
                    let new_tag = self.add_tag_input.trim().to_string();
                    if !new_tag.is_empty() {
                        if self.caption.trim().is_empty() {
                            self.caption = new_tag;
                        } else {
                            self.caption.push_str(", ");
                            self.caption.push_str(&new_tag);
                        }
                        self.add_tag_input.clear();
                        self.dirty = true;
                    }
                }
            });
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
