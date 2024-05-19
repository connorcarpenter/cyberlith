use ui_builder_config::{
    BaseNodeStyle, ButtonStyle, NodeStyle, PanelStyle, SpinnerStyle, StyleId, TextStyle,
    TextboxStyle, WidgetKind, WidgetStyle,
};

pub(crate) fn compute_styles(styles: Vec<NodeStyle>) -> Vec<BaseNodeStyle> {
    let mut output = Vec::new();

    for style in styles.iter() {
        let widget_kind = style.base.widget_style.kind();
        let widget_style = match widget_kind {
            WidgetKind::Panel => WidgetStyle::Panel(PanelStyle::empty()),
            WidgetKind::Button => WidgetStyle::Button(ButtonStyle::empty()),
            WidgetKind::Text => WidgetStyle::Text(TextStyle::empty()),
            WidgetKind::Textbox => WidgetStyle::Textbox(TextboxStyle::empty()),
            WidgetKind::Spinner => WidgetStyle::Spinner(SpinnerStyle::empty()),
            WidgetKind::UiContainer => WidgetStyle::UiContainer,
        };
        let mut output_style = BaseNodeStyle::empty(widget_style);

        apply_parent_styles(&styles, style.parent_style, &mut output_style);
        output_style.merge(&style.base, false);

        output.push(output_style);
    }

    output
}

fn apply_parent_styles(
    styles: &Vec<NodeStyle>,
    parent_style_id_opt: Option<StyleId>,
    output_style: &mut BaseNodeStyle,
) {
    if let Some(parent_style_id) = parent_style_id_opt {
        let parent_style_id: usize = parent_style_id.as_usize();
        let parent_style = styles.get(parent_style_id).unwrap();

        // recurse
        apply_parent_styles(styles, parent_style.parent_style, output_style);

        output_style.merge(&parent_style.base, true);
    }
}
