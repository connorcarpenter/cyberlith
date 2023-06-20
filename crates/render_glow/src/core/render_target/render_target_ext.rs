use render_api::components::Viewport;

use crate::core::*;

macro_rules! impl_render_target_core_extensions_body {
    () => {
        ///
        /// Returns the viewport that encloses the entire target.
        ///
        pub fn viewport(&self) -> Viewport {
            Viewport::new_at_origin(self.width(), self.height())
        }
    };
}

macro_rules! impl_render_target_core_extensions {
    // 2 generic arguments with bounds
    ($name:ident < $a:ident : $ta:tt , $b:ident : $tb:tt >) => {
        impl<$a: $ta, $b: $tb> $name<$a, $b> {
            impl_render_target_core_extensions_body!();
        }
    };
    // 1 generic argument with bound
    ($name:ident < $a:ident : $ta:tt >) => {
        impl<$a: $ta> $name<$a> {
            impl_render_target_core_extensions_body!();
        }
    };
    // 1 liftetime argument
    ($name:ident < $lt:lifetime >) => {
        impl<$lt> $name<$lt> {
            impl_render_target_core_extensions_body!();
        }
    };
    // without any arguments
    ($name:ty) => {
        impl $name {
            impl_render_target_core_extensions_body!();
        }
    };
}

impl_render_target_core_extensions!(RenderTarget<'a>);
impl_render_target_core_extensions!(ColorTarget<'a>);
impl_render_target_core_extensions!(DepthTarget<'a>);
