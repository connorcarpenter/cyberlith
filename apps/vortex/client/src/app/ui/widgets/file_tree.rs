
use render_egui::egui::{remap_clamp, Ui, Shape, Context, emath, Id, InnerResponse, NumExt, pos2, Rect, remap, Response, Sense, Stroke, vec2};

#[derive(Clone)]
pub struct FileTree {
    name: String,
    trees: Vec<FileTree>,
    selected: bool,
}

impl FileTree {
    pub fn project_test() -> Self {
        Self::new("Projects", vec![
            Self::new("dir1", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
            ]),
            Self::new("dir2", vec![
                Self::new("file1", vec![]),
            ]),
            Self::new("dir3", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
                Self::new("dir4", vec![
                    Self::new("file1", vec![]),
                    Self::new("file2", vec![]),
                ]),
            ]),
        ])
    }

    pub fn changes_test() -> Self {
        Self::new("Changes", vec![
            Self::new("dir1", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
            ]),
            Self::new("dir2", vec![
                Self::new("file1", vec![]),
            ]),
            Self::new("dir3", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
                Self::new("dir4", vec![
                    Self::new("file1", vec![]),
                    Self::new("file2", vec![]),
                ]),
            ]),
        ])
    }
}

impl FileTree {
    pub fn new(name: &str, trees: Vec<FileTree>) -> Self {
        Self {
            name: name.to_string(),
            trees,
            selected: false,
        }
    }

    pub fn render_root(&mut self, ui: &mut Ui) {
        self.render(ui, "")
    }

    fn render(&mut self, ui: &mut Ui, path: &str) {
        let tree_name = self.name.clone();
        let full_path = format!("{}:{}", path, tree_name);
        self.selected = CustomCollapsingHeader::show(&full_path, &tree_name, self.selected, ui, |ui| self.render_children(ui, &full_path));
    }

    fn render_children(&mut self, ui: &mut Ui, path: &str) {
        for tree in self.trees.iter_mut() {
            tree.render(ui, path);
        }
    }
}

struct CustomCollapsingHeader;

impl CustomCollapsingHeader {
    pub fn show<ShowRet>(full_path: &str, tree_name: &str, tree_select: bool, ui: &mut Ui, show_body: impl FnOnce(&mut Ui) -> ShowRet) -> bool {
        let mut selected = tree_select;
        let id = ui.make_persistent_id(full_path);
        CollapsingState::load_with_default(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.toggle_value(&mut selected, tree_name);
            })
            .body(|ui| {
                return show_body(ui);
            });
        return selected;
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct InnerState {
    open: bool,

    /// Height of the region when open. Used for animations
    open_height: Option<f32>,
}

/// This is a a building block for building collapsing regions.
///
/// It is used by [`CollapsingHeader`] and [`Window`], but can also be used on its own.
///
/// See [`CollapsingState::show_header`] for how to show a collapsing header with a custom header.
#[derive(Clone, Debug)]
pub struct CollapsingState {
    id: Id,
    state: InnerState,
}

impl CollapsingState {
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| {
            d.get_persisted::<InnerState>(id)
                .map(|state| Self { id, state })
        })
    }

    pub fn store(&self, ctx: &Context) {
        ctx.data_mut(|d| d.insert_persisted(self.id, self.state));
    }

    pub fn load_with_default(ctx: &Context, id: Id) -> Self {
        Self::load(ctx, id).unwrap_or(CollapsingState {
            id,
            state: InnerState {
                open: false,
                open_height: None,
            },
        })
    }

    //pub fn is_open(&self) -> bool {
    //    self.state.open
    //}

    //pub fn set_open(&mut self, open: bool) {
    //    self.state.open = open;
    //}

    pub fn toggle(&mut self, ui: &Ui) {
        self.state.open = !self.state.open;
        ui.ctx().request_repaint();
    }

    /// 0 for closed, 1 for open, with tweening
    pub fn openness(&self, ctx: &Context) -> f32 {
        if ctx.memory(|mem| mem.everything_is_visible()) {
            1.0
        } else {
            ctx.animate_bool(self.id, self.state.open)
        }
    }

    /// Will toggle when clicked, etc.
    fn show_default_button_indented(&mut self, ui: &mut Ui) -> Response {
        self.show_button_indented(ui, paint_default_icon)
    }

    /// Will toggle when clicked, etc.
    fn show_button_indented(
        &mut self,
        ui: &mut Ui,
        icon_fn: impl FnOnce(&mut Ui, f32, &Response) + 'static,
    ) -> Response {
        let size = vec2(ui.spacing().indent, ui.spacing().icon_width);
        let (_id, rect) = ui.allocate_space(size);
        let response = ui.interact(rect, self.id, Sense::click());
        if response.clicked() {
            self.toggle(ui);
        }

        let (mut icon_rect, _) = ui.spacing().icon_rectangles(response.rect);
        icon_rect.set_center(pos2(
            response.rect.left() + ui.spacing().indent / 2.0,
            response.rect.center().y,
        ));
        let openness = self.openness(ui.ctx());
        let small_icon_response = response.clone().with_new_rect(icon_rect);
        icon_fn(ui, openness, &small_icon_response);
        response
    }

    /// Shows header and body (if expanded).
    ///
    /// The header will start with the default button in a horizontal layout, followed by whatever you add.
    ///
    /// Will also store the state.
    ///
    /// Returns the response of the collapsing button, the custom header, and the custom body.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// let id = ui.make_persistent_id("my_collapsing_header");
    /// egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
    ///     .show_header(ui, |ui| {
    ///         ui.label("Header"); // you can put checkboxes or whatever here
    ///     })
    ///     .body(|ui| ui.label("Body"));
    /// # });
    /// ```
    pub fn show_header<HeaderRet>(
        mut self,
        ui: &mut Ui,
        add_header: impl FnOnce(&mut Ui) -> HeaderRet,
    ) -> HeaderResponse<'_, HeaderRet> {
        let header_response = ui.horizontal(|ui| {
            let prev_item_spacing = ui.spacing_mut().item_spacing;
            ui.spacing_mut().item_spacing.x = 0.0; // the toggler button uses the full indent width
            let collapser = self.show_default_button_indented(ui);
            ui.spacing_mut().item_spacing = prev_item_spacing;
            (collapser, add_header(ui))
        });
        HeaderResponse {
            state: self,
            ui,
            toggle_button_response: header_response.inner.0,
            header_response: InnerResponse {
                response: header_response.response,
                inner: header_response.inner.1,
            },
        }
    }

    /// Show body if we are open, with a nice animation between closed and open.
    /// Indent the body to show it belongs to the header.
    ///
    /// Will also store the state.
    pub fn show_body_indented<R>(
        &mut self,
        header_response: &Response,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let id = self.id;
        self.show_body_unindented(ui, |ui| {
            ui.indent(id, |ui| {
                // make as wide as the header:
                ui.expand_to_include_x(header_response.rect.right());
                add_body(ui)
            })
                .inner
        })
    }

    /// Show body if we are open, with a nice animation between closed and open.
    /// Will also store the state.
    pub fn show_body_unindented<R>(
        &mut self,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let openness = self.openness(ui.ctx());
        if openness <= 0.0 {
            self.store(ui.ctx()); // we store any earlier toggling as promised in the docstring
            None
        } else if openness < 1.0 {
            Some(ui.scope(|child_ui| {
                let max_height = if self.state.open && self.state.open_height.is_none() {
                    // First frame of expansion.
                    // We don't know full height yet, but we will next frame.
                    // Just use a placeholder value that shows some movement:
                    10.0
                } else {
                    let full_height = self.state.open_height.unwrap_or_default();
                    remap_clamp(openness, 0.0..=1.0, 0.0..=full_height)
                };

                let mut clip_rect = child_ui.clip_rect();
                clip_rect.max.y = clip_rect.max.y.min(child_ui.max_rect().top() + max_height);
                child_ui.set_clip_rect(clip_rect);

                let ret = add_body(child_ui);

                let mut min_rect = child_ui.min_rect();
                self.state.open_height = Some(min_rect.height());
                self.store(child_ui.ctx()); // remember the height

                // Pretend children took up at most `max_height` space:
                min_rect.max.y = min_rect.max.y.at_most(min_rect.top() + max_height);
                child_ui.set_clip_rect(min_rect);
                ret
            }))
        } else {
            let ret_response = ui.scope(add_body);
            let full_size = ret_response.response.rect.size();
            self.state.open_height = Some(full_size.y);
            self.store(ui.ctx()); // remember the height
            Some(ret_response)
        }
    }
}

/// From [`CollapsingState::show_header`].
#[must_use = "Remember to show the body"]
pub struct HeaderResponse<'ui, HeaderRet> {
    state: CollapsingState,
    ui: &'ui mut Ui,
    toggle_button_response: Response,
    header_response: InnerResponse<HeaderRet>,
}

impl<'ui, HeaderRet> HeaderResponse<'ui, HeaderRet> {
    /// Returns the response of the collapsing button, the custom header, and the custom body.
    pub fn body<BodyRet>(
        mut self,
        add_body: impl FnOnce(&mut Ui) -> BodyRet,
    ) -> (
        Response,
        InnerResponse<HeaderRet>,
        Option<InnerResponse<BodyRet>>,
    ) {
        let body_response =
            self.state
                .show_body_indented(&self.header_response.response, self.ui, add_body);
        (
            self.toggle_button_response,
            self.header_response,
            body_response,
        )
    }
}

// ----------------------------------------------------------------------------

/// Paint the arrow icon that indicated if the region is open or not
pub fn paint_default_icon(ui: &mut Ui, openness: f32, response: &Response) {
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