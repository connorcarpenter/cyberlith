use render_egui::egui::{
    emath, remap, vec2, Align, Color32, Id, Layout, NumExt, Rect, Response, Rounding, Sense, Shape,
    Stroke, TextStyle, Ui, WidgetText,
};

#[derive(Clone)]
pub struct FileTree {
    name: String,
    trees: Vec<FileTree>,
    selected: bool,
    opened: bool,
}

impl FileTree {
    pub fn project_test() -> Self {
        Self::new(
            "Projects",
            vec![
                Self::new(
                    "dir1",
                    vec![Self::new("file1", vec![]), Self::new("file2", vec![])],
                ),
                Self::new("dir2", vec![Self::new("file1", vec![])]),
                Self::new(
                    "dir3",
                    vec![
                        Self::new("file1", vec![]),
                        Self::new("file2", vec![]),
                        Self::new(
                            "dir4",
                            vec![Self::new("file1", vec![]), Self::new("file2", vec![])],
                        ),
                    ],
                ),
            ],
        )
    }

    pub fn changes_test() -> Self {
        Self::new(
            "Changes",
            vec![
                Self::new(
                    "dir1",
                    vec![Self::new("file1", vec![]), Self::new("file2", vec![])],
                ),
                Self::new("dir2", vec![Self::new("file1", vec![])]),
                Self::new(
                    "dir3",
                    vec![
                        Self::new("file1", vec![]),
                        Self::new("file2", vec![]),
                        Self::new(
                            "dir4",
                            vec![Self::new("file1", vec![]), Self::new("file2", vec![])],
                        ),
                    ],
                ),
            ],
        )
    }
}

impl FileTree {
    pub fn new(name: &str, trees: Vec<FileTree>) -> Self {
        Self {
            name: name.to_string(),
            trees,
            selected: false,
            opened: false,
        }
    }

    pub fn render_root(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            self.render(ui, "", 0);
        });
    }

    fn render(&mut self, ui: &mut Ui, path: &str, depth: usize) {
        let separator = if path.len() > 0 { ":" } else { "" };
        let full_path = format!("{}{}{}", path, separator, self.name);

        if self.trees.len() > 0 {
            self.render_row(ui, &full_path, depth, true, paint_default_icon);
            if self.opened {
                self.render_children(ui, &full_path, depth);
            }
        } else {
            self.render_row(ui, &full_path, depth, false, paint_no_icon);
        }
    }

    fn render_children(&mut self, ui: &mut Ui, path: &str, depth: usize) {
        for tree in self.trees.iter_mut() {
            tree.render(ui, path, depth + 1);
        }
    }

    pub fn render_row(
        &mut self,
        ui: &mut Ui,
        path: &str,
        depth: usize,
        is_dir: bool,
        icon_fn: impl FnOnce(&mut Ui, bool, &Response) + 'static,
    ) {
        let wrap_width = ui.available_width();
        let unicode_icon = if is_dir { "📁" } else { "📃" };
        let text_str: &str = &format!("{} {}", unicode_icon, &self.name);
        let widget_text: WidgetText = text_str.into();
        let text = widget_text.into_galley(ui, None, wrap_width, TextStyle::Button);

        let mut desired_size = text.size();
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);

        let (mut row_rect, row_response) = ui.allocate_at_least(desired_size, Sense::click());

        if ui.is_rect_visible(row_response.rect) {
            let item_spacing = 4.0;
            let indent_spacing = 14.0;

            let text_size = text.size();

            let mut inner_pos = ui.layout().align_size_within_rect(text_size, row_rect).min;

            // Add Margin
            inner_pos.x += (depth as f32 * indent_spacing) + 4.0;

            let icon_response = {
                let icon_size = vec2(ui.spacing().icon_width, ui.spacing().icon_width);
                let icon_rect = Rect::from_min_size(inner_pos, icon_size);

                let big_icon_response = ui.interact(icon_rect, Id::new(path), Sense::click());

                if is_dir {
                    if big_icon_response.clicked() {
                        self.opened = !self.opened;
                    }
                }

                big_icon_response
            };

            // Draw Row
            {
                let row_fill = if self.selected {
                    Some(Color32::from_rgb(0, 92, 128))
                } else {
                    if row_response.hovered() || icon_response.hovered() {
                        Some(Color32::from_gray(70))
                    } else {
                        None
                    }
                };

                if let Some(fill_color) = row_fill {
                    row_rect.min.y -= 1.0;
                    row_rect.max.y += 2.0;
                    row_rect.max.x -= 2.0;

                    ui.painter()
                        .rect(row_rect, Rounding::none(), fill_color, Stroke::NONE);
                }
            }

            // Draw Icon
            if is_dir {
                let (small_icon_rect, _) = ui.spacing().icon_rectangles(icon_response.rect);
                let small_icon_response = icon_response.clone().with_new_rect(small_icon_rect);

                icon_fn(ui, self.opened, &small_icon_response);
                inner_pos.x += small_icon_response.rect.width() + item_spacing;
            } else {
                inner_pos.x += 14.0;
            }

            // Draw Text
            {
                text.paint_with_visuals(ui.painter(), inner_pos, ui.style().noninteractive());
                inner_pos.x += text_size.x + item_spacing;
            }
        }

        if row_response.clicked() {
            self.selected = !self.selected;
        }
    }
}

// ----------------------------------------------------------------------------

/// Paint the arrow icon that indicated if the region is open or not
pub fn paint_default_icon(ui: &mut Ui, openned: bool, response: &Response) {
    let openness = if openned { 1.0 } else { 0.0 };

    let visuals = ui.style().interact(response);

    let rect = response.rect;

    // Draw a pointy triangle arrow:
    let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height()) * 0.75);
    let rect = rect.expand(visuals.expansion);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    use std::f32::consts::TAU;
    let rotation = emath::Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }

    ui.painter().add(Shape::convex_polygon(
        points,
        visuals.fg_stroke.color,
        Stroke::NONE,
    ));
}

pub fn paint_no_icon(_ui: &mut Ui, _openness: bool, _response: &Response) {}
