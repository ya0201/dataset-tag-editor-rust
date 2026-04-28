use std::{fs, path::PathBuf};

pub struct Settings {
    pub zoom: f32,
    pub list_width: f32,
    pub tag_width: f32,
    pub caption_height: f32,
    pub last_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            list_width: 300.0,
            tag_width: 200.0,
            caption_height: 160.0,
            last_dir: None,
        }
    }
}

fn config_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".cache"))
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("dataset-tag-editor-rust").join("settings.txt")
}

pub fn load_settings() -> Settings {
    let mut s = Settings::default();
    if let Ok(text) = fs::read_to_string(config_path()) {
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                let v = v.trim();
                match k.trim() {
                    "zoom" => {
                        if let Ok(x) = v.parse() {
                            s.zoom = x;
                        }
                    }
                    "list_width" => {
                        if let Ok(x) = v.parse() {
                            s.list_width = x;
                        }
                    }
                    "tag_width" => {
                        if let Ok(x) = v.parse() {
                            s.tag_width = x;
                        }
                    }
                    "caption_height" => {
                        if let Ok(x) = v.parse() {
                            s.caption_height = x;
                        }
                    }
                    "last_dir" => {
                        s.last_dir = Some(PathBuf::from(v));
                    }
                    _ => {}
                }
            }
        }
    }
    s.zoom = s.zoom.clamp(0.5, 3.0);
    s
}

pub fn save_settings(s: &Settings) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut content = format!(
        "zoom={}\nlist_width={}\ntag_width={}\ncaption_height={}\n",
        s.zoom, s.list_width, s.tag_width, s.caption_height,
    );
    if let Some(dir) = &s.last_dir {
        content.push_str(&format!("last_dir={}\n", dir.to_string_lossy()));
    }
    let _ = fs::write(&path, content);
}
