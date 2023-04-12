use render_egui::{egui, egui::{CollapsingHeader, Ui}};

#[derive(Clone)]
pub struct Tree {
    name: String,
    trees: Vec<Tree>,
    selected: bool,
}

impl Tree {
    pub fn project_test() -> Self {
        Self::new("Projects", vec![
            Self::new("project1", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
            ]),
            Self::new("project2", vec![
                Self::new("file1", vec![]),
            ]),
            Self::new("project3", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
                Self::new("project4", vec![
                    Self::new("file1", vec![]),
                    Self::new("file2", vec![]),
                ]),
            ]),
        ])
    }

    pub fn changes_test() -> Self {
        Self::new("Changes", vec![
            Self::new("change1", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
            ]),
            Self::new("change2", vec![
                Self::new("file1", vec![]),
            ]),
            Self::new("change3", vec![
                Self::new("file1", vec![]),
                Self::new("file2", vec![]),
                Self::new("change4", vec![
                    Self::new("file1", vec![]),
                    Self::new("file2", vec![]),
                ]),
            ]),
        ])
    }
}

impl Tree {
    pub fn new(name: &str, trees: Vec<Tree>) -> Self {
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
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.toggle_value(&mut selected, full_path); // put &tree.name here instead!
            })
            .body(|ui| {
                return show_body(ui);
            });
        return selected;
    }
}