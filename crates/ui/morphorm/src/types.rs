/// The layout type determines how the nodes will position its parent-directed children.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    /// Stack child elements horizontally.
    Row,
    /// Stack child elements vertically.
    #[default]
    Column,
}

impl LayoutType {
    // Helper function for selecting between optional values depending on the layout type.
    pub(crate) fn select_unwrap<T: Default, S>(
        &self,
        s: S,
        first: impl FnOnce(S) -> Option<T>,
        second: impl FnOnce(S) -> Option<T>,
    ) -> T {
        match self {
            LayoutType::Row => first(s).unwrap_or_default(),
            LayoutType::Column => second(s).unwrap_or_default(),
        }
    }

    // Helper function for selecting between optional values depending on the layout type with specified default.
    pub(crate) fn select_unwrap_default<T, S>(
        &self,
        s: S,
        first: impl FnOnce(S) -> Option<T>,
        second: impl FnOnce(S) -> Option<T>,
        default: T,
    ) -> T {
        match self {
            LayoutType::Row => first(s).unwrap_or(default),
            LayoutType::Column => second(s).unwrap_or(default),
        }
    }
}

/// The position type determines whether a node will be positioned in-line with its siblings or out-of-line / independently of its siblings.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    /// Node is positioned relative to parent but ignores its siblings.
    Absolute,
    /// Node is positioned relative to parent and in-line with siblings.
    #[default]
    Relative,
}

/// Units which describe spacing and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginUnits {
    /// A number of logical pixels.
    Pixels(f32),
    /// A percentage of the parent dimension.
    ///
    /// A percentage of the (parent's width - parent's padding - margin) when applied to left, right properties.
    /// A percentage of the (parent's height - parent's padding - margin) when applied to top, bottom properties.
    Percentage(f32),
    /// A factor of the remaining free space.
    ///
    /// The remaining free space is the parent space minus the space and size of any fixed-size nodes in that axis.
    /// The remaining free space is then shared between any stretch nodes based on the ratio of their stretch factors.
    ///
    /// For example, given two stretch nodes with factors of 1.0 and 2.0 respectively. The first will occupy 1/3 of the
    /// remaining free space while the second will occupy 2/3 of the remaining free space.
    Stretch(f32),
}

impl Default for MarginUnits {
    fn default() -> Self {
        MarginUnits::Pixels(0.0)
    }
}

impl MarginUnits {

    pub fn add_size_units(&mut self, size_units: SizeUnits) {
        match (self, size_units) {
            (_, SizeUnits::Auto) => {}
            (MarginUnits::Stretch(_), _) => {}
            (MarginUnits::Pixels(val), SizeUnits::Pixels(size)) => *val += size,
            (MarginUnits::Percentage(val), SizeUnits::Percentage(size)) => *val += size,
            (_, _) => {}
        }
    }

    /// Returns the units converted to pixels or a provided default.
    pub fn to_px(&self, parent_value: f32, parent_padding: f32, default: f32) -> f32 {
        match self {
            MarginUnits::Pixels(pixels) => *pixels,
            MarginUnits::Percentage(percentage) => percentage_calc(*percentage, parent_value, parent_padding),
            MarginUnits::Stretch(_) => default,
        }
    }

    pub fn to_px_clamped(&self, parent_value: f32, parent_padding: f32, default: f32, min: MarginUnits, max: MarginUnits) -> f32 {
        let min = min.to_px(parent_value, parent_padding, f32::MIN);
        let max = max.to_px(parent_value, parent_padding, f32::MAX);

        match self {
            MarginUnits::Pixels(pixels) => pixels.min(max).max(min),
            MarginUnits::Percentage(percentage) => percentage_calc(*percentage, parent_value, parent_padding).min(max).max(min),
            MarginUnits::Stretch(_) => default.min(max).max(min),
        }
    }

    pub fn clamp(&self, min: MarginUnits, max: MarginUnits) -> Self {
        match (self, min, max) {
            (MarginUnits::Pixels(val), MarginUnits::Pixels(min), MarginUnits::Pixels(max)) => MarginUnits::Pixels(val.min(max).max(min)),
            (MarginUnits::Percentage(val), MarginUnits::Percentage(min), MarginUnits::Percentage(max)) => {
                MarginUnits::Percentage(val.min(max).max(min))
            }
            (MarginUnits::Stretch(val), MarginUnits::Stretch(min), MarginUnits::Stretch(max)) => MarginUnits::Stretch(val.min(max).max(min)),
            _ => *self,
        }
    }

    /// Returns true if the value is in pixels.
    pub fn is_pixels(&self) -> bool {
        matches!(self, MarginUnits::Pixels(_))
    }

    /// Returns true if the value is a percentage.
    pub fn is_percentage(&self) -> bool {
        matches!(self, MarginUnits::Percentage(_))
    }

    /// Returns true if the value is a stretch factor.
    pub fn is_stretch(&self) -> bool {
        matches!(self, MarginUnits::Stretch(_))
    }
}

/// Units which describe spacing and size.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SizeUnits {
    /// A number of logical pixels.
    Pixels(f32),
    /// A percentage of the parent dimension.
    ///
    /// A percentage of the (parent's width - parent's padding - margin - border) when applied to width properties.
    /// A percentage of the (parent's height - parent's padding - margin - border) when applied to height properties.
    Percentage(f32),
    /// Automatically determine the value.
    ///
    /// When applied to size (width, height) Auto will either size to fit its children, or if there are no children
    /// the node will be sized based on the [`content_size`](crate::Node::content_size) property of the node.
    #[default]
    Auto,
}

impl SizeUnits {
    /// Returns the units converted to pixels or a provided default.
    pub fn to_px(&self, parent_value: f32, parent_padding: f32, default: f32) -> f32 {
        match self {
            SizeUnits::Pixels(pixels) => *pixels,
            SizeUnits::Percentage(percentage) => percentage_calc(*percentage, parent_value, parent_padding),
            SizeUnits::Auto => default,
        }
    }

    pub fn to_px_clamped(&self, parent_value: f32, parent_padding: f32, default: f32, min: SizeUnits, max: SizeUnits) -> f32 {
        let min = min.to_px(parent_value, parent_padding, f32::MIN);
        let max = max.to_px(parent_value, parent_padding, f32::MAX);

        match self {
            SizeUnits::Pixels(pixels) => pixels.min(max).max(min),
            SizeUnits::Percentage(percentage) => percentage_calc(*percentage, parent_value, parent_padding).min(max).max(min),
            SizeUnits::Auto => default.min(max).max(min),
        }
    }

    pub fn clamp(&self, min: SizeUnits, max: SizeUnits) -> Self {
        match (self, min, max) {
            (SizeUnits::Pixels(val), SizeUnits::Pixels(min), SizeUnits::Pixels(max)) => SizeUnits::Pixels(val.min(max).max(min)),
            (SizeUnits::Percentage(val), SizeUnits::Percentage(min), SizeUnits::Percentage(max)) => {
                SizeUnits::Percentage(val.min(max).max(min))
            }
            _ => *self,
        }
    }

    /// Returns true if the value is in pixels.
    pub fn is_pixels(&self) -> bool {
        matches!(self, SizeUnits::Pixels(_))
    }

    /// Returns true if the value is a percentage.
    pub fn is_percentage(&self) -> bool {
        matches!(self, SizeUnits::Percentage(_))
    }

    /// Returns true if the value is auto.
    pub fn is_auto(&self) -> bool {
        self == &SizeUnits::Auto
    }
}

/// A type which represents the computed size of a node after [`layout`](crate::Node::layout).
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Size {
    /// The computed size on the main axis.
    pub main: f32,
    /// The computed size on the cross axis.
    pub cross: f32,
}

pub fn percentage_calc(percentage: f32, parent_value: f32, parent_padding: f32) -> f32 {
    (percentage / 100.0) * (parent_value - parent_padding)
}