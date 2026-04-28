use eframe::egui;
use crate::app::App;
use crate::settings::save_settings;
use crate::utils::{show_overlay, tag_color};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ディレクトリクローズの遅延実行（n 計算前に処理）
        if self.close_dir_pending {
            self.close_dir_pending = false;
            self.close_dir();
            save_settings(&self.current_settings());
        }

        // ズームショートカット
        let (zoom_in, zoom_out, zoom_reset) = ctx.input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && (i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus)),
                cmd && i.key_pressed(egui::Key::Minus),
                cmd && i.key_pressed(egui::Key::Num0),
            )
        });
        if zoom_in || zoom_out || zoom_reset {
            if zoom_in {
                self.zoom = (self.zoom + 0.1).min(3.0);
            }
            if zoom_out {
                self.zoom = (self.zoom - 0.1).max(0.5);
            }
            if zoom_reset {
                self.zoom = 1.0;
            }
            self.zoom = (self.zoom * 10.0).round() / 10.0;
            ctx.set_pixels_per_point(self.native_ppp * self.zoom);
            save_settings(&self.current_settings());
        }

        // アプリ終了
        if ctx.input(|i| i.viewport().close_requested()) {
            save_settings(&self.current_settings());
            if !self.pending.is_empty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.confirm_close = true;
            }
        }

        // アプリ終了確認モーダル
        if self.confirm_close {
            show_overlay(ctx);
            egui::Window::new("未保存の変更")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "{}件の未保存の変更があります。",
                        self.pending.len()
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("全保存して終了").clicked() {
                            self.save_all();
                            self.confirm_close = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("破棄して終了").clicked() {
                            self.pending.clear();
                            self.confirm_close = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("キャンセル").clicked() {
                            self.confirm_close = false;
                        }
                    });
                });
        }

        // ディレクトリを閉じる確認モーダル
        if self.confirm_close_dir {
            show_overlay(ctx);
            egui::Window::new("未保存の変更##dir")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "{}件の未保存の変更があります。",
                        self.pending.len()
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("全保存して閉じる").clicked() {
                            self.save_all();
                            self.close_dir();
                            self.confirm_close_dir = false;
                            save_settings(&self.current_settings());
                        }
                        if ui.button("破棄して閉じる").clicked() {
                            self.close_dir();
                            self.confirm_close_dir = false;
                            save_settings(&self.current_settings());
                        }
                        if ui.button("キャンセル").clicked() {
                            self.confirm_close_dir = false;
                        }
                    });
                });
        }

        // フォルダドロップ
        let dropped_dir = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .find_map(|f| f.path.as_ref().filter(|p| p.is_dir()).cloned())
        });
        if let Some(dir) = dropped_dir {
            self.load_dir(&dir, ctx);
            save_settings(&self.current_settings());
        }

        // キーボードナビゲーション
        if !ctx.wants_keyboard_input() {
            let delta = ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowLeft) {
                    -1i32
                } else if i.key_pressed(egui::Key::ArrowRight) {
                    1
                } else {
                    0
                }
            });
            if delta != 0 {
                self.navigate(delta, ctx);
            }
        }

        let n = self.entries.len();
        let current_dirty = self.pending.contains_key(&self.current);
        let any_dirty = !self.pending.is_empty();
        let mouse_released = ctx.input(|i| i.pointer.primary_released());

        // トップバー
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open Directory").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.load_dir(&dir, ctx);
                        save_settings(&self.current_settings());
                    }
                }
                if self.current_dir.is_some() {
                    if ui.button("Close Directory").clicked() {
                        if self.pending.is_empty() {
                            self.close_dir_pending = true;
                        } else {
                            self.confirm_close_dir = true;
                        }
                    }
                }
                if n > 0 {
                    ui.separator();
                    if ui.button("◀ Prev").clicked() {
                        self.navigate(-1, ctx);
                    }
                    if ui.button("Next ▶").clicked() {
                        self.navigate(1, ctx);
                    }
                    let name = self.entries[self.current]
                        .image_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let label = if current_dirty {
                        format!("* {}/{}: {name}", self.current + 1, n)
                    } else {
                        format!("{}/{}: {name}", self.current + 1, n)
                    };
                    ui.label(label);
                    if current_dirty && ui.button("Save").clicked() {
                        self.save_current();
                    }
                    if any_dirty && ui.button("Save All").clicked() {
                        self.save_all();
                    }
                }
            });
        });

        // 左パネル（ファイルリスト）
        let list_panel = egui::SidePanel::left("list")
            .default_width(self.list_width)
            .min_width(20.0)
            .show(ctx, |ui| {
                ui.set_min_width(ui.available_width());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..n {
                        let (thumb_info, name, is_entry_dirty) = {
                            let e = &self.entries[i];
                            let info = e.thumbnail.as_ref().map(|t| (t.id(), t.size()));
                            let name = e
                                .image_path
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string();
                            let dirty = self.pending.contains_key(&i);
                            (info, name, dirty)
                        };
                        let is_selected = i == self.current;
                        let fill = if is_selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let available_width = ui.available_width();
                        let row = egui::Frame::none().fill(fill).show(ui, |ui| {
                            ui.set_min_width(available_width);
                            ui.horizontal(|ui| {
                                if let Some((id, [w, h])) = thumb_info {
                                    let th = 64.0f32;
                                    let tw = w as f32 * th / h as f32;
                                    ui.add(egui::Image::new((id, egui::vec2(tw, th))));
                                }
                                ui.vertical(|ui| {
                                    if is_entry_dirty {
                                        ui.label(
                                            egui::RichText::new("●")
                                                .color(egui::Color32::from_rgb(245, 158, 11))
                                                .small(),
                                        );
                                    }
                                    ui.add(egui::Label::new(&name).truncate());
                                });
                            });
                        });
                        if ui
                            .interact(
                                row.response.rect,
                                egui::Id::new("row").with(i),
                                egui::Sense::click(),
                            )
                            .clicked()
                            && !is_selected
                        {
                            self.go_to(i, ctx);
                        }
                        ui.separator();
                    }
                });
            });

        // 右パネル（タグ頻度）
        let tag_panel = egui::SidePanel::right("tag_counts")
            .default_width(self.tag_width)
            .min_width(160.0)
            .show(ctx, |ui| {
                ui.label(format!("Tags ({})", self.tag_counts.len()));
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("tag_grid")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for (tag, count) in &self.tag_counts {
                                ui.label(tag.as_str());
                                ui.label(count.to_string());
                                ui.end_row();
                            }
                        });
                });
            });

        // 下パネル（キャプション編集）
        let caption_panel = egui::TopBottomPanel::bottom("caption")
            .resizable(true)
            .min_height(120.0)
            .default_height(self.caption_height)
            .show(ctx, |ui| {
                let tags: Vec<String> = self
                    .caption
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
                let mut remove_idx: Option<usize> = None;
                let mut new_drag_idx = self.drag_idx;
                let mut drop_target: Option<usize> = None;
                let released = ctx.input(|i| i.pointer.primary_released());

                let scroll_height = ui.available_height() - 36.0;
                let tag_area_width = ui.available_width();
                egui::ScrollArea::vertical()
                    .max_height(scroll_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(tag_area_width);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                            let font_id = ui.style().text_styles[&egui::TextStyle::Body].clone();
                            let h = ui.text_style_height(&egui::TextStyle::Body);
                            let v_margin = 4.0_f32;
                            let h_margin = 8.0_f32;
                            let inner_spacing = 4.0_f32;

                            for (i, tag) in tags.iter().enumerate() {
                                let being_dragged = self.drag_idx == Some(i);
                                let base = tag_color(tag);

                                let (handle_w, tag_w, x_w) = ui.fonts(|f| {
                                    let hw = f.layout_no_wrap("≡".to_owned(), font_id.clone(), egui::Color32::WHITE).size().x;
                                    let tw = f.layout_no_wrap(tag.clone(), font_id.clone(), egui::Color32::WHITE).size().x;
                                    let xw = f.layout_no_wrap("⊗".to_owned(), font_id.clone(), egui::Color32::WHITE).size().x;
                                    (hw, tw, xw)
                                });

                                let content_w = handle_w + inner_spacing + tag_w
                                    + if !being_dragged { inner_spacing + x_w } else { 0.0 };
                                let chip_size = egui::vec2(
                                    content_w + h_margin * 2.0,
                                    h + v_margin * 2.0,
                                );

                                let (rect, _) = ui.allocate_exact_size(chip_size, egui::Sense::hover());

                                if ui.is_rect_visible(rect) {
                                    let fill = if being_dragged {
                                        egui::Color32::from_rgba_unmultiplied(
                                            base.r(), base.g(), base.b(), 100,
                                        )
                                    } else {
                                        base
                                    };
                                    ui.painter().rect_filled(rect, egui::Rounding::same(8.0), fill);

                                    let cy = rect.center().y;
                                    let mut cx = rect.min.x + h_margin;
                                    ui.painter().text(egui::pos2(cx, cy), egui::Align2::LEFT_CENTER,
                                        "≡", font_id.clone(), egui::Color32::from_white_alpha(160));
                                    cx += handle_w + inner_spacing;
                                    ui.painter().text(egui::pos2(cx, cy), egui::Align2::LEFT_CENTER,
                                        tag.as_str(), font_id.clone(), egui::Color32::WHITE);
                                    cx += tag_w + inner_spacing;
                                    if !being_dragged {
                                        ui.painter().text(egui::pos2(cx, cy), egui::Align2::LEFT_CENTER,
                                            "⊗", font_id.clone(), egui::Color32::WHITE);
                                    }
                                }

                                // ≡ drag handle
                                let handle_rect = egui::Rect::from_min_size(
                                    rect.min,
                                    egui::vec2(h_margin + handle_w + inner_spacing, rect.height()),
                                );
                                let handle_r = ui.interact(
                                    handle_rect, egui::Id::new("hdl").with(i), egui::Sense::drag(),
                                );
                                if handle_r.drag_started() {
                                    new_drag_idx = Some(i);
                                }

                                // ⊗ delete button
                                if !being_dragged {
                                    let cx = rect.min.x + h_margin + handle_w + inner_spacing + tag_w + inner_spacing;
                                    let x_rect = egui::Rect::from_min_size(
                                        egui::pos2(cx, rect.min.y),
                                        egui::vec2(x_w + h_margin, rect.height()),
                                    );
                                    let x_r = ui.interact(
                                        x_rect, egui::Id::new("del").with(i), egui::Sense::click(),
                                    );
                                    if x_r.clicked() {
                                        remove_idx = Some(i);
                                    }
                                }

                                let hovered = ctx.input(|inp| {
                                    inp.pointer.hover_pos().is_some_and(|p| rect.contains(p))
                                });
                                if being_dragged {
                                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                                } else if self.drag_idx.is_none() && handle_r.hovered() {
                                    ctx.set_cursor_icon(egui::CursorIcon::Grab);
                                }
                                if self.drag_idx.is_some() && !being_dragged && hovered {
                                    drop_target = Some(i);
                                    ui.painter().rect_stroke(
                                        rect.expand(2.0),
                                        egui::Rounding::same(9.0),
                                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                                    );
                                }
                            }
                        });
                    });

                if let Some(i) = remove_idx {
                    self.caption = tags
                        .iter()
                        .enumerate()
                        .filter(|&(j, _)| j != i)
                        .map(|(_, t)| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.mark_dirty();
                }
                if released {
                    if let (Some(src), Some(dst)) = (self.drag_idx, drop_target) {
                        let mut v: Vec<&str> = tags.iter().map(String::as_str).collect();
                        let item = v.remove(src);
                        v.insert(dst, item);
                        self.caption = v.join(", ");
                        self.mark_dirty();
                    }
                    new_drag_idx = None;
                }
                self.drag_idx = new_drag_idx;

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("add tag");
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.add_tag_input).desired_width(300.0),
                    );
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
                            self.mark_dirty();
                        }
                    }
                });
                // content_ui.min_rect() must cover the full panel inner rect so
                // PanelState records the correct (dragged) height rather than
                // the content height, which would cause the panel to shrink
                // back toward min_height on every repaint.
                ui.expand_to_include_rect(ui.max_rect());
            });

        // パネルサイズ変化を検出して保存（マウス離し時）
        if mouse_released {
            let new_lw = list_panel.response.rect.width();
            let new_tw = tag_panel.response.rect.width();
            let new_ch = caption_panel.response.rect.height();
            if (new_lw - self.list_width).abs() > 0.5
                || (new_tw - self.tag_width).abs() > 0.5
                || (new_ch - self.caption_height).abs() > 0.5
            {
                self.list_width = new_lw;
                self.tag_width = new_tw;
                self.caption_height = new_ch;
                save_settings(&self.current_settings());
            }
        }

        // 中央パネル（画像表示）
        let hovering_dir = ctx.input(|i| {
            i.raw
                .hovered_files
                .iter()
                .any(|f| f.path.as_ref().is_some_and(|p| p.is_dir()))
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
            if hovering_dir {
                let rect = ui.ctx().screen_rect();
                ui.painter().rect_filled(
                    rect,
                    egui::Rounding::ZERO,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "ここにドロップ",
                    egui::FontId::proportional(32.0),
                    egui::Color32::WHITE,
                );
            }
        });
    }
}
