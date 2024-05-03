
#[derive(Eq, PartialEq, Clone, Copy, Default)]
pub enum Alignment {
    Start,
    #[default]
    Center,
    End,
}

impl Alignment {
    pub(crate) fn has_start(&self) -> bool {
        match self {
            Alignment::Start | Alignment::Center => true,
            _ => false,
        }
    }

    pub(crate) fn has_end(&self) -> bool {
        match self {
            Alignment::End | Alignment::Center => true,
            _ => false,
        }
    }
}

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
    pub(crate) fn select<T, S>(
        &self,
        s: S,
        first: impl FnOnce(S) -> T,
        second: impl FnOnce(S) -> T,
    ) -> T {
        match self {
            LayoutType::Row => first(s),
            LayoutType::Column => second(s),
        }
    }
}

/// Units which describe spacing and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginUnits {

    /// A percentage of the parent dimension.
    ///
    /// A percentage of the (parent's width - parent's padding - margin) when applied to left, right properties.
    /// A percentage of the (parent's height - parent's padding - margin) when applied to top, bottom properties.
    Percentage(f32),

    /// A percentage of the viewport dimension.
    ///
    /// A percentage of the viewport's width when applied to width properties.
    /// A percentage of the viewport's height when applied to height properties.
    Viewport(f32),
}

impl Default for MarginUnits {
    fn default() -> Self {
        MarginUnits::Percentage(0.0)
    }
}

impl MarginUnits {

    /// Returns the units converted to pixels or a provided default.
    pub fn to_px(&self, viewport_value: f32, parent_value: f32, parent_padding: f32) -> f32 {
        match self {
            MarginUnits::Percentage(percentage) => {
                percentage_calc(*percentage, parent_value, parent_padding)
            }
            MarginUnits::Viewport(percentage) => percentage_calc(*percentage, viewport_value, 0.0),
        }
    }

    pub fn to_px_clamped(
        &self,
        viewport_value: f32,
        parent_value: f32,
        parent_padding: f32,
        min: MarginUnits,
        max: MarginUnits,
    ) -> f32 {
        let min = min.to_px(viewport_value, parent_value, parent_padding);
        let max = max.to_px(viewport_value, parent_value, parent_padding);

        match self {
            MarginUnits::Percentage(percentage) => {
                percentage_calc(*percentage, parent_value, parent_padding)
                    .min(max)
                    .max(min)
            }
            MarginUnits::Viewport(percentage) => percentage_calc(*percentage, viewport_value, 0.0)
                .min(max)
                .max(min),
        }
    }

    pub fn clamp(&self, min: MarginUnits, max: MarginUnits) -> Self {
        match (self, min, max) {
            (
                MarginUnits::Percentage(val),
                MarginUnits::Percentage(min),
                MarginUnits::Percentage(max),
            ) => MarginUnits::Percentage(val.min(max).max(min)),
            (
                MarginUnits::Viewport(val),
                MarginUnits::Viewport(min),
                MarginUnits::Viewport(max),
            ) => MarginUnits::Viewport(val.min(max).max(min)),
            _ => *self,
        }
    }

    /// Returns true if the value is a percentage.
    pub fn is_percentage(&self) -> bool {
        matches!(self, MarginUnits::Percentage(_))
    }

    /// Returns true if the value is a viewport percentage.
    pub fn is_viewport(&self) -> bool {
        matches!(self, MarginUnits::Viewport(_))
    }
}

/// Units which describe spacing and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeUnits {
    /// A percentage of the parent dimension.
    ///
    /// A percentage of the (parent's width - parent's padding - margin - border) when applied to width properties.
    /// A percentage of the (parent's height - parent's padding - margin - border) when applied to height properties.
    Percentage(f32),

    /// A percentage of the viewport dimension.
    ///
    /// A percentage of the viewport's width when applied to width properties.
    /// A percentage of the viewport's height when applied to height properties.
    Viewport(f32),
}

impl SizeUnits {
    /// Returns the units converted to pixels or a provided default.
    pub fn to_px(
        &self,
        viewport_value: f32,
        parent_value: f32,
        parent_padding: f32,
    ) -> f32 {
        match self {
            SizeUnits::Percentage(percentage) => {
                percentage_calc(*percentage, parent_value, parent_padding)
            }
            SizeUnits::Viewport(percentage) => percentage_calc(*percentage, viewport_value, 0.0),
        }
    }

    /// Returns true if the value is a percentage.
    pub fn is_percentage(&self) -> bool {
        matches!(self, SizeUnits::Percentage(_))
    }

    /// Returns true if the value is a viewport percentage.
    pub fn is_viewport(&self) -> bool {
        matches!(self, SizeUnits::Viewport(_))
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
