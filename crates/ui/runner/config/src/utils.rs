use ui_layout::TextMeasurer;

pub fn text_get_subimage_indices(text: &str) -> Vec<usize> {
    let mut output = Vec::new();

    for c in text.chars() {
        let c: u8 = if c.is_ascii() {
            c as u8
        } else {
            42 // asterisk
        };
        let subimage_index = (c - 32) as usize;
        output.push(subimage_index);
    }

    output
}

pub fn text_get_raw_rects(
    text_measurer: &dyn TextMeasurer,
    subimage_indices: &[usize],
) -> (Vec<f32>, f32) {
    let mut widths = Vec::new();

    let mut width = 0.0;
    widths.push(width);

    for subimage_index in subimage_indices {
        if width > 0.0 {
            width += 8.0; // between character spacing - TODO: replace with config
        }

        // get character width in order to move cursor appropriately
        let icon_width = text_measurer.get_raw_char_width(*subimage_index);

        width += icon_width;

        widths.push(width);
    }

    (widths, text_measurer.get_raw_char_height(0))
}

pub fn text_measure_raw_size(text_measurer: &dyn TextMeasurer, text: &str) -> (f32, f32) {
    let subimage_indices = text_get_subimage_indices(text);
    let (widths, height) = text_get_raw_rects(text_measurer, &subimage_indices);
    (
        if let Some(width) = widths.last() {
            *width
        } else {
            0.0
        },
        height,
    )
}

pub fn point_is_inside(layout: (f32, f32, f32, f32), mouse_x: f32, mouse_y: f32) -> bool {
    let (width, height, posx, posy) = layout;

    mouse_x >= posx
        && mouse_x <= posx + width + 1.0
        && mouse_y >= posy
        && mouse_y <= posy + height + 1.0
}