use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use eframe::egui;
use crate::settings::Settings;
use crate::utils::{load_thumbnail, load_texture};

pub struct Entry {
    pub image_path: PathBuf,
    pub caption_path: PathBuf,
    pub thumbnail: Option<egui::TextureHandle>,
}

#[derive(Default)]
pub struct App {
    pub entries: Vec<Entry>,
    pub current: usize,
    pub caption: String,
    pub texture: Option<egui::TextureHandle>,
    pub tag_counts: Vec<(String, usize)>,
    pub add_tag_input: String,
    pub drag_idx: Option<usize>,
    pub pending: HashMap<usize, String>,
    pub current_dir: Option<PathBuf>,
    pub confirm_close: bool,
    pub confirm_close_dir: bool,
    pub close_dir_pending: bool,
    pub native_ppp: f32,
    pub zoom: f32,
    pub list_width: f32,
    pub tag_width: f32,
    pub caption_height: f32,
}

impl App {
    pub fn current_settings(&self) -> Settings {
        Settings {
            zoom: self.zoom,
            list_width: self.list_width,
            tag_width: self.tag_width,
            caption_height: self.caption_height,
            last_dir: self.current_dir.clone(),
        }
    }

    pub fn load_dir(&mut self, dir: &Path, ctx: &egui::Context) {
        self.pending.clear();
        self.current_dir = Some(dir.to_path_buf());
        let mut paths: Vec<PathBuf> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("jpg" | "jpeg" | "png")
                )
            })
            .collect();
        paths.sort();
        self.entries = paths
            .into_iter()
            .map(|img| {
                let stem = img.file_stem().unwrap().to_string_lossy().into_owned();
                let parent = img.parent().unwrap().to_path_buf();
                let caption = [".txt", ".caption"]
                    .iter()
                    .map(|ext| parent.join(format!("{stem}{ext}")))
                    .find(|p| p.exists())
                    .unwrap_or_else(|| parent.join(format!("{stem}.txt")));
                let thumbnail = load_thumbnail(ctx, &img);
                Entry { image_path: img, caption_path: caption, thumbnail }
            })
            .collect();
        self.current = 0;
        self.load_entry(ctx);
        self.rebuild_tag_counts();
    }

    pub fn close_dir(&mut self) {
        self.entries.clear();
        self.current = 0;
        self.caption.clear();
        self.texture = None;
        self.tag_counts.clear();
        self.pending.clear();
        self.drag_idx = None;
        self.current_dir = None;
    }

    pub fn rebuild_tag_counts(&mut self) {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for (i, entry) in self.entries.iter().enumerate() {
            let text = self
                .pending
                .get(&i)
                .cloned()
                .unwrap_or_else(|| fs::read_to_string(&entry.caption_path).unwrap_or_default());
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

    pub fn load_entry(&mut self, ctx: &egui::Context) {
        self.caption.clear();
        self.texture = None;
        self.drag_idx = None;
        if let Some(entry) = self.entries.get(self.current) {
            self.caption = self
                .pending
                .get(&self.current)
                .cloned()
                .unwrap_or_else(|| fs::read_to_string(&entry.caption_path).unwrap_or_default());
            self.texture = load_texture(ctx, &entry.image_path);
        }
    }

    pub fn mark_dirty(&mut self) {
        self.pending.insert(self.current, self.caption.clone());
        self.rebuild_tag_counts();
    }

    pub fn save_current(&mut self) {
        if let Some(entry) = self.entries.get(self.current) {
            let _ = fs::write(&entry.caption_path, &self.caption);
            self.pending.remove(&self.current);
        }
        self.rebuild_tag_counts();
    }

    pub fn save_all(&mut self) {
        for (i, text) in &self.pending {
            if let Some(entry) = self.entries.get(*i) {
                let _ = fs::write(&entry.caption_path, text);
            }
        }
        self.pending.clear();
        self.rebuild_tag_counts();
    }

    pub fn go_to(&mut self, index: usize, ctx: &egui::Context) {
        self.current = index;
        self.load_entry(ctx);
    }

    pub fn navigate(&mut self, delta: i32, ctx: &egui::Context) {
        let n = self.entries.len();
        if n == 0 {
            return;
        }
        let next = ((self.current as i32 + delta).rem_euclid(n as i32)) as usize;
        self.go_to(next, ctx);
    }
}
