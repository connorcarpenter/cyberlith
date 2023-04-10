use render_egui::egui::{CollapsingHeader, Ui};

#[derive(Clone, Default)]
pub struct Tree(Vec<Tree>);

impl Tree {
    pub fn new() -> Self {
        Self(vec![
            Tree(vec![Tree::default(); 4]),
            Tree(vec![Tree(vec![Tree::default(); 2]); 3]),
        ])
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        self.ui_impl(ui, 0, "root")
    }
}

impl Tree {
    fn ui_impl(&mut self, ui: &mut Ui, depth: usize, name: &str) {
        CollapsingHeader::new(name)
            .default_open(depth < 1)
            .show(ui, |ui| self.children_ui(ui, depth));
    }

    fn children_ui(&mut self, ui: &mut Ui, depth: usize) {
        for (i, tree) in self.0.iter_mut().enumerate() {
            tree.ui_impl(ui, depth + 1, &format!("child #{}", i));
        }
    }
}
