use std::cell::RefCell;

use js_sys::{self, Array};
use slotmap::{new_key_type, SlotMap};

use web_sys::{
    self, HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, VideoFrame,
    WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlQuery,
    WebGlRenderbuffer, WebGlSampler, WebGlShader, WebGlSync, WebGlTexture, WebGlTransformFeedback,
    WebGlUniformLocation, WebGlVertexArrayObject,
};

use super::*;

#[allow(dead_code)]
#[derive(Debug)]
struct Extensions {
    pub angle_instanced_arrays: Option<web_sys::AngleInstancedArrays>,
    pub ext_blend_minmax: Option<web_sys::ExtBlendMinmax>,
    pub ext_color_buffer_float: Option<web_sys::ExtColorBufferFloat>,
    pub ext_color_buffer_half_float: Option<web_sys::ExtColorBufferHalfFloat>,
    pub ext_disjoint_timer_query: Option<web_sys::ExtDisjointTimerQuery>,
    pub ext_disjoint_timer_query_webgl2: Option<()>,
    pub ext_float_blend: Option<()>,
    pub ext_frag_depth: Option<web_sys::ExtFragDepth>,
    pub ext_shader_texture_lod: Option<web_sys::ExtShaderTextureLod>,
    pub ext_srgb: Option<web_sys::ExtSRgb>,
    pub ext_texture_compression_bptc: Option<()>,
    pub ext_texture_compression_rgtc: Option<()>,
    pub ext_texture_filter_anisotropic: Option<web_sys::ExtTextureFilterAnisotropic>,
    pub khr_parallel_shader_compile: Option<()>,
    pub oes_element_index_uint: Option<web_sys::OesElementIndexUint>,
    pub oes_fbo_render_mipmap: Option<()>,
    pub oes_standard_derivatives: Option<web_sys::OesStandardDerivatives>,
    pub oes_texture_float: Option<web_sys::OesTextureFloat>,
    pub oes_texture_float_linear: Option<web_sys::OesTextureFloatLinear>,
    pub oes_texture_half_float: Option<web_sys::OesTextureHalfFloat>,
    pub oes_texture_half_float_linear: Option<web_sys::OesTextureHalfFloatLinear>,
    pub oes_vertex_array_object: Option<web_sys::OesVertexArrayObject>,
    pub ovr_multiview2: Option<web_sys::OvrMultiview2>,
    pub webgl_color_buffer_float: Option<web_sys::WebglColorBufferFloat>,
    pub webgl_compressed_texture_astc: Option<web_sys::WebglCompressedTextureAstc>,
    pub webgl_compressed_texture_etc: Option<web_sys::WebglCompressedTextureEtc>,
    pub webgl_compressed_texture_etc1: Option<web_sys::WebglCompressedTextureEtc1>,
    pub webgl_compressed_texture_pvrtc: Option<web_sys::WebglCompressedTexturePvrtc>,
    pub webgl_compressed_texture_s3tc: Option<web_sys::WebglCompressedTextureS3tc>,
    pub webgl_compressed_texture_s3tc_srgb: Option<web_sys::WebglCompressedTextureS3tcSrgb>,
    pub webgl_debug_renderer_info: Option<web_sys::WebglDebugRendererInfo>,
    pub webgl_debug_shaders: Option<web_sys::WebglDebugShaders>,
    pub webgl_depth_texture: Option<web_sys::WebglDepthTexture>,
    pub webgl_draw_buffers: Option<web_sys::WebglDrawBuffers>,
    pub webgl_lose_context: Option<web_sys::WebglLoseContext>,
    pub webgl_multi_draw: Option<web_sys::WebglMultiDraw>,
}

type TrackedResource<K, V> = RefCell<SlotMap<K, V>>;

fn tracked_resource<K: slotmap::Key, V>() -> TrackedResource<K, V> {
    RefCell::new(SlotMap::with_key())
}

#[derive(Debug)]
pub struct Context {
    raw: WebGl2RenderingContext,
    extensions: Extensions,
    version: Version,
    supported_extensions: HashSet<String>,
    shaders: TrackedResource<WebShaderKey, WebGlShader>,
    programs: TrackedResource<WebProgramKey, WebGlProgram>,
    buffers: TrackedResource<WebBufferKey, WebGlBuffer>,
    vertex_arrays: TrackedResource<WebVertexArrayKey, WebGlVertexArrayObject>,
    textures: TrackedResource<WebTextureKey, WebGlTexture>,
    samplers: TrackedResource<WebSamplerKey, WebGlSampler>,
    fences: TrackedResource<WebFenceKey, WebGlSync>,
    framebuffers: TrackedResource<WebFramebufferKey, WebGlFramebuffer>,
    renderbuffers: TrackedResource<WebRenderbufferKey, WebGlRenderbuffer>,
    queries: TrackedResource<WebQueryKey, WebGlQuery>,
    transform_feedbacks: TrackedResource<WebTransformFeedbackKey, WebGlTransformFeedback>,
}

// bindgen's gl context don't share an interface so a macro is used to deduplicate a bunch of code here
macro_rules! build_extensions {
    ($context:ident, $context_type:ty) => {{
        fn get_extension<T>(context: &$context_type, name: &str) -> Option<T>
        where
            T: wasm_bindgen::JsCast,
        {
            use wasm_bindgen::JsCast;
            // `unchecked_into` is used here because WebGL extensions aren't actually JS classes
            // these objects are duck-type representations of the actual Rust classes
            // https://github.com/rustwasm/wasm-bindgen/pull/1449
            context
                .get_extension(name)
                .ok()
                .and_then(|maybe_ext| maybe_ext.map(|ext| ext.unchecked_into::<T>()))
        }

        fn get_extension_no_object(context: &$context_type, name: &str) -> Option<()> {
            context
                .get_extension(name)
                .ok()
                .and_then(|maybe_ext| maybe_ext.map(|_| ()))
        }

        let extensions = Extensions {
            angle_instanced_arrays: get_extension::<web_sys::AngleInstancedArrays>(
                &$context,
                "ANGLE_instanced_arrays",
            ),
            ext_blend_minmax: get_extension::<web_sys::ExtBlendMinmax>(
                &$context,
                "EXT_blend_minmax",
            ),
            ext_color_buffer_float: get_extension::<web_sys::ExtColorBufferFloat>(
                &$context,
                "EXT_color_buffer_float",
            ),
            ext_color_buffer_half_float: get_extension::<web_sys::ExtColorBufferHalfFloat>(
                &$context,
                "EXT_color_buffer_half_float",
            ),
            ext_disjoint_timer_query: get_extension::<web_sys::ExtDisjointTimerQuery>(
                &$context,
                "EXT_disjoint_timer_query",
            ),
            ext_disjoint_timer_query_webgl2: get_extension_no_object(
                &$context,
                "EXT_disjoint_timer_query_webgl2",
            ),
            ext_float_blend: get_extension_no_object(&$context, "EXT_float_blend"),
            ext_frag_depth: get_extension::<web_sys::ExtFragDepth>(&$context, "EXT_frag_depth"),
            ext_shader_texture_lod: get_extension::<web_sys::ExtShaderTextureLod>(
                &$context,
                "EXT_shader_texture_lod",
            ),
            ext_srgb: get_extension::<web_sys::ExtSRgb>(&$context, "EXT_sRGB"),
            ext_texture_compression_bptc: get_extension_no_object(
                &$context,
                "EXT_texture_compression_bptc",
            ),
            ext_texture_compression_rgtc: get_extension_no_object(
                &$context,
                "EXT_texture_compression_rgtc",
            ),
            ext_texture_filter_anisotropic: get_extension::<web_sys::ExtTextureFilterAnisotropic>(
                &$context,
                "EXT_texture_filter_anisotropic",
            ),
            khr_parallel_shader_compile: get_extension_no_object(
                &$context,
                "KHR_parallel_shader_compile",
            ),
            oes_element_index_uint: get_extension::<web_sys::OesElementIndexUint>(
                &$context,
                "OES_element_index_uint",
            ),
            oes_fbo_render_mipmap: get_extension_no_object(&$context, "OES_fbo_render_mipmap"),
            oes_standard_derivatives: get_extension::<web_sys::OesStandardDerivatives>(
                &$context,
                "OES_standard_derivatives",
            ),
            oes_texture_float: get_extension::<web_sys::OesTextureFloat>(
                &$context,
                "OES_texture_float",
            ),
            oes_texture_float_linear: get_extension::<web_sys::OesTextureFloatLinear>(
                &$context,
                "OES_texture_float_linear",
            ),
            oes_texture_half_float: get_extension::<web_sys::OesTextureHalfFloat>(
                &$context,
                "OES_texture_half_float",
            ),
            oes_texture_half_float_linear: get_extension::<web_sys::OesTextureHalfFloatLinear>(
                &$context,
                "OES_texture_half_float_linear",
            ),
            oes_vertex_array_object: get_extension::<web_sys::OesVertexArrayObject>(
                &$context,
                "OES_vertex_array_object",
            ),
            ovr_multiview2: get_extension::<web_sys::OvrMultiview2>(&$context, "OVR_multiview2"),
            webgl_color_buffer_float: get_extension::<web_sys::WebglColorBufferFloat>(
                &$context,
                "WEBGL_color_buffer_float",
            ),
            webgl_compressed_texture_astc: get_extension::<web_sys::WebglCompressedTextureAstc>(
                &$context,
                "WEBGL_compressed_texture_astc",
            ),
            webgl_compressed_texture_etc: get_extension::<web_sys::WebglCompressedTextureEtc>(
                &$context,
                "WEBGL_compressed_texture_etc",
            ),
            webgl_compressed_texture_etc1: get_extension::<web_sys::WebglCompressedTextureEtc1>(
                &$context,
                "WEBGL_compressed_texture_etc1",
            ),
            webgl_compressed_texture_pvrtc: get_extension::<web_sys::WebglCompressedTexturePvrtc>(
                &$context,
                "WEBGL_compressed_texture_pvrtc",
            ),
            webgl_compressed_texture_s3tc: get_extension::<web_sys::WebglCompressedTextureS3tc>(
                &$context,
                "WEBGL_compressed_texture_s3tc",
            ),
            webgl_compressed_texture_s3tc_srgb: get_extension::<
                web_sys::WebglCompressedTextureS3tcSrgb,
            >(
                &$context, "WEBGL_compressed_texture_s3tc_srgb"
            ),
            webgl_debug_renderer_info: get_extension::<web_sys::WebglDebugRendererInfo>(
                &$context,
                "WEBGL_debug_renderer_info",
            ),
            webgl_debug_shaders: get_extension::<web_sys::WebglDebugShaders>(
                &$context,
                "WEBGL_debug_shaders",
            ),
            webgl_depth_texture: get_extension::<web_sys::WebglDepthTexture>(
                &$context,
                "WEBGL_depth_texture",
            ),
            webgl_draw_buffers: get_extension::<web_sys::WebglDrawBuffers>(
                &$context,
                "WEBGL_draw_buffers",
            ),
            webgl_lose_context: get_extension::<web_sys::WebglLoseContext>(
                &$context,
                "WEBGL_lose_context",
            ),
            webgl_multi_draw: get_extension::<web_sys::WebglMultiDraw>(
                &$context,
                "WEBGL_multi_draw",
            ),
        };

        let supported_extensions = $context
            .get_supported_extensions()
            .unwrap()
            .iter()
            .map(|val| val.as_string().unwrap())
            .collect::<HashSet<String>>();

        (extensions, supported_extensions)
    }};
}

impl Context {
    pub fn from_webgl2_context(context: WebGl2RenderingContext) -> Self {
        let (extensions, supported_extensions) = build_extensions!(context, WebGl2RenderingContext);

        // Retrieve and parse `GL_VERSION`
        let raw_string = context.get_parameter(VERSION).unwrap().as_string().unwrap();
        let version = Version::parse(&raw_string).unwrap();

        Self {
            raw: context,
            extensions,
            supported_extensions,
            version,
            shaders: tracked_resource(),
            programs: tracked_resource(),
            buffers: tracked_resource(),
            vertex_arrays: tracked_resource(),
            textures: tracked_resource(),
            samplers: tracked_resource(),
            fences: tracked_resource(),
            framebuffers: tracked_resource(),
            renderbuffers: tracked_resource(),
            queries: tracked_resource(),
            transform_feedbacks: tracked_resource(),
        }
    }

    // These functions are defined in this order:
    //
    // - image_bitmap
    // - html_canvas
    // - html_image
    // - html_video
    // - video_frame

    pub unsafe fn tex_image_2d_with_image_bitmap(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        format: u32,
        ty: u32,
        pixels: &ImageBitmap,
    ) {
        // TODO: Handle return value?
        self.raw
            .tex_image_2d_with_u32_and_u32_and_image_bitmap(
                target,
                level,
                internal_format,
                format,
                ty,
                pixels,
            )
            .unwrap();
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_2d_with_image_bitmap_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        pixels: &ImageBitmap,
    ) {
        self.raw
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_image_bitmap(
                target,
                level,
                internal_format,
                width,
                height,
                0, // required to be zero
                format,
                ty,
                pixels,
            )
            .unwrap();
    }

    pub unsafe fn tex_image_2d_with_html_canvas(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        format: u32,
        ty: u32,
        canvas: &HtmlCanvasElement,
    ) {
        self.raw
            .tex_image_2d_with_u32_and_u32_and_html_canvas_element(
                target,
                level,
                internal_format,
                format,
                ty,
                canvas,
            )
            .unwrap();
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_2d_with_html_canvas_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        canvas: &HtmlCanvasElement,
    ) {
        self.raw
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_canvas_element(
                target,
                level,
                internal_format,
                width,
                height,
                0, // required to be zero
                format,
                ty,
                canvas,
            )
            .unwrap();
    }

    pub unsafe fn tex_image_2d_with_html_image(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        format: u32,
        ty: u32,
        image: &HtmlImageElement,
    ) {
        self.raw
            .tex_image_2d_with_u32_and_u32_and_html_image_element(
                target,
                level,
                internal_format,
                format,
                ty,
                image,
            )
            .unwrap();
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_2d_with_html_image_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        image: &HtmlImageElement,
    ) {
        self.raw
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_image_element(
                target,
                level,
                internal_format,
                width,
                height,
                0, // required to be zero
                format,
                ty,
                image,
            )
            .unwrap();
    }

    pub unsafe fn tex_image_2d_with_html_video(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        format: u32,
        ty: u32,
        video: &HtmlVideoElement,
    ) {
        self.raw
            .tex_image_2d_with_u32_and_u32_and_html_video_element(
                target,
                level,
                internal_format,
                format,
                ty,
                video,
            )
            .unwrap();
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_2d_with_html_video_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        video: &HtmlVideoElement,
    ) {
        self.raw
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_video_element(
                target,
                level,
                internal_format,
                width,
                height,
                0, // required to be zero
                format,
                ty,
                video,
            )
            .unwrap();
    }

    pub unsafe fn tex_image_2d_with_video_frame(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        format: u32,
        ty: u32,
        video_frame: &VideoFrame,
    ) {
        self.raw
            .tex_image_2d_with_u32_and_u32_and_video_frame(
                target,
                level,
                internal_format,
                format,
                ty,
                video_frame,
            )
            .unwrap();
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_2d_with_video_frame_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        video_frame: &VideoFrame,
    ) {
        self.raw
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_video_frame(
                target,
                level,
                internal_format,
                width,
                height,
                0,
                format,
                ty,
                video_frame,
            )
            .unwrap();
    }

    pub unsafe fn tex_sub_image_2d_with_image_bitmap(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        format: u32,
        ty: u32,
        image: &ImageBitmap,
    ) {
        self.raw
            .tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(
                target, level, x_offset, y_offset, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_2d_with_image_bitmap_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        image: &ImageBitmap,
    ) {
        self.raw
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                target, level, x_offset, y_offset, width, height, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    pub unsafe fn tex_sub_image_2d_with_html_canvas(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        format: u32,
        ty: u32,
        image: &HtmlCanvasElement,
    ) {
        self.raw
            .tex_sub_image_2d_with_u32_and_u32_and_html_canvas_element(
                target, level, x_offset, y_offset, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_2d_with_html_canvas_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        image: &HtmlCanvasElement,
    ) {
        self.raw
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                target, level, x_offset, y_offset, width, height, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    pub unsafe fn tex_sub_image_2d_with_html_image(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        format: u32,
        ty: u32,
        image: &HtmlImageElement,
    ) {
        self.raw
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                target, level, x_offset, y_offset, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_2d_with_html_image_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        image: &HtmlImageElement,
    ) {
        self.raw
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                target, level, x_offset, y_offset, width, height, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    pub unsafe fn tex_sub_image_2d_with_html_video(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        format: u32,
        ty: u32,
        image: &HtmlVideoElement,
    ) {
        self.raw
            .tex_sub_image_2d_with_u32_and_u32_and_html_video_element(
                target, level, x_offset, y_offset, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_2d_with_html_video_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        image: &HtmlVideoElement,
    ) {
        self.raw
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                target, level, x_offset, y_offset, width, height, format, ty, image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    pub unsafe fn tex_sub_image_2d_with_video_frame(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        _width: i32,
        _height: i32,
        format: u32,
        ty: u32,
        video_frame: &VideoFrame,
    ) {
        self.raw
            .tex_sub_image_2d_with_u32_and_u32_and_video_frame(
                target,
                level,
                x_offset,
                y_offset,
                format,
                ty,
                video_frame,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_2d_with_video_frame_and_width_and_height(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        video_frame: &VideoFrame,
    ) {
        self.raw
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_video_frame(
                target,
                level,
                x_offset,
                y_offset,
                width,
                height,
                format,
                ty,
                video_frame,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_3d_with_image_bitmap(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        image: &ImageBitmap,
    ) {
        self.raw
            .tex_image_3d_with_image_bitmap(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_3d_with_html_canvas_element(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        image: &HtmlCanvasElement,
    ) {
        self.raw
            .tex_image_3d_with_html_canvas_element(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_3d_with_html_image_element(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        image: &HtmlImageElement,
    ) {
        self.raw
            .tex_image_3d_with_html_image_element(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_3d_with_html_video_element(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        image: &HtmlVideoElement,
    ) {
        self.raw
            .tex_image_3d_with_html_video_element(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_image_3d_with_video_frame(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        video_frame: &VideoFrame,
    ) {
        self.raw
            .tex_image_3d_with_video_frame(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                ty,
                video_frame,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_3d_with_image_bitmap(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        image: &ImageBitmap,
    ) {
        self.raw
            .tex_sub_image_3d_with_image_bitmap(
                target, level, x_offset, y_offset, z_offset, width, height, depth, format, ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_3d_with_html_canvas_element(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        image: &HtmlCanvasElement,
    ) {
        self.raw
            .tex_sub_image_3d_with_html_canvas_element(
                target, level, x_offset, y_offset, z_offset, width, height, depth, format, ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_3d_with_html_image_element(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        image: &HtmlImageElement,
    ) {
        self.raw
            .tex_sub_image_3d_with_html_image_element(
                target, level, x_offset, y_offset, z_offset, width, height, depth, format, ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_3d_with_html_video_element(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        image: &HtmlVideoElement,
    ) {
        self.raw
            .tex_sub_image_3d_with_html_video_element(
                target, level, x_offset, y_offset, z_offset, width, height, depth, format, ty,
                image,
            )
            .unwrap(); // TODO: Handle return value?
    }

    /// WebGL2 Only
    pub unsafe fn tex_sub_image_3d_with_video_frame(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        video_frame: &VideoFrame,
    ) {
        self.raw
            .tex_sub_image_3d_with_video_frame(
                target,
                level,
                x_offset,
                y_offset,
                z_offset,
                width,
                height,
                depth,
                format,
                ty,
                video_frame,
            )
            .unwrap(); // TODO: Handle return value?
    }

    pub unsafe fn framebuffer_texture_multiview_ovr(
        &self,
        target: u32,
        attachment: u32,
        texture: Option<<Self as HasContext>::Texture>,
        level: i32,
        base_view_index: i32,
        num_views: i32,
    ) {
        let textures = self.textures.borrow();
        let raw_texture = texture.map(|t| textures.get_unchecked(t));

        if let Some(ext) = &self.extensions.ovr_multiview2 {
            ext.framebuffer_texture_multiview_ovr(
                target,
                attachment,
                raw_texture,
                level,
                base_view_index,
                num_views,
            );
        }
    }

    pub unsafe fn bind_external_framebuffer(&self, target: u32, framebuffer: &WebGlFramebuffer) {
        self.raw.bind_framebuffer(target, Some(framebuffer));
    }

    // Returns true if the `WEBGL_compressed_texture_astc` extension is enabled and the "ldr"
    // profile is supported.
    pub fn compressed_texture_astc_supports_ldr_profile(&self) -> bool {
        self.extensions
            .webgl_compressed_texture_astc
            .as_ref()
            .map(|ext| {
                let profiles = ext.get_supported_profiles().unwrap_or_default();
                profiles.includes(&"ldr".into(), 0)
            })
            .unwrap_or(false)
    }

    // Returns true if the `WEBGL_compressed_texture_astc` extension is enabled and the "hdr"
    // profile is supported.
    pub fn compressed_texture_astc_supports_hdr_profile(&self) -> bool {
        self.extensions
            .webgl_compressed_texture_astc
            .as_ref()
            .map(|ext| {
                let profiles = ext.get_supported_profiles().unwrap_or_default();
                profiles.includes(&"hdr".into(), 0)
            })
            .unwrap_or(false)
    }

    /// Simulate losing WebGL rendering context.
    /// Only works when "WEBGL_lose_context" extension is available.
    pub fn lose_context(&self) {
        if let Some(ext) = &self.extensions.webgl_lose_context {
            ext.lose_context()
        }
    }

    /// Simulate restoring WebGL rendering context.
    /// Only works when "WEBGL_lose_context" extension is available.
    /// This will panic when the context is not lost.
    pub fn restore_context(&self) {
        if let Some(ext) = &self.extensions.webgl_lose_context {
            ext.restore_context()
        }
    }
}

new_key_type! { pub struct WebShaderKey; }
new_key_type! { pub struct WebProgramKey; }
new_key_type! { pub struct WebBufferKey; }
new_key_type! { pub struct WebVertexArrayKey; }
new_key_type! { pub struct WebTextureKey; }
new_key_type! { pub struct WebSamplerKey; }
new_key_type! { pub struct WebFenceKey; }
new_key_type! { pub struct WebFramebufferKey; }
new_key_type! { pub struct WebRenderbufferKey; }
new_key_type! { pub struct WebQueryKey; }
new_key_type! { pub struct WebTransformFeedbackKey; }

impl HasContext for Context {
    type Shader = WebShaderKey;
    type Program = WebProgramKey;
    type Buffer = WebBufferKey;
    type VertexArray = WebVertexArrayKey;
    type Texture = WebTextureKey;
    type Sampler = WebSamplerKey;
    type Fence = WebFenceKey;
    type Framebuffer = WebFramebufferKey;
    type Renderbuffer = WebRenderbufferKey;
    type Query = WebQueryKey;
    type UniformLocation = WebGlUniformLocation;
    type TransformFeedback = WebTransformFeedbackKey;

    fn supported_extensions(&self) -> &HashSet<String> {
        &self.supported_extensions
    }

    fn supports_debug(&self) -> bool {
        false
    }

    fn version(&self) -> &Version {
        &self.version
    }

    unsafe fn create_framebuffer(&self) -> Result<Self::Framebuffer, String> {
        let raw_framebuffer = self.raw.create_framebuffer();

        match raw_framebuffer {
            Some(s) => {
                let key = self.framebuffers.borrow_mut().insert(s);
                Ok(key)
            }
            None => Err(String::from("Unable to create framebuffer object")),
        }
    }

    unsafe fn is_framebuffer(&self, framebuffer: Self::Framebuffer) -> bool {
        let framebuffers = self.framebuffers.borrow_mut();
        if let Some(ref f) = framebuffers.get(framebuffer) {
            self.raw.is_framebuffer(Some(f))
        } else {
            false
        }
    }

    unsafe fn create_query(&self) -> Result<Self::Query, String> {
        let raw_query = self.raw.create_query();

        match raw_query {
            Some(s) => {
                let key = self.queries.borrow_mut().insert(s);
                Ok(key)
            }
            None => Err(String::from("Unable to create query object")),
        }
    }

    unsafe fn create_renderbuffer(&self) -> Result<Self::Renderbuffer, String> {
        let raw_renderbuffer = self.raw.create_renderbuffer();

        match raw_renderbuffer {
            Some(s) => {
                let key = self.renderbuffers.borrow_mut().insert(s);
                Ok(key)
            }
            None => Err(String::from("Unable to create renderbuffer object")),
        }
    }

    unsafe fn is_renderbuffer(&self, renderbuffer: Self::Renderbuffer) -> bool {
        let renderbuffers = self.renderbuffers.borrow_mut();
        if let Some(ref r) = renderbuffers.get(renderbuffer) {
            self.raw.is_renderbuffer(Some(r))
        } else {
            false
        }
    }

    unsafe fn create_sampler(&self) -> Result<Self::Sampler, String> {
        let raw_sampler = self.raw.create_sampler();

        match raw_sampler {
            Some(s) => {
                let key = self.samplers.borrow_mut().insert(s);
                Ok(key)
            }
            None => Err(String::from("Unable to create sampler object")),
        }
    }

    unsafe fn create_shader(&self, shader_type: u32) -> Result<Self::Shader, String> {
        let raw_shader = self.raw.create_shader(shader_type as u32);

        match raw_shader {
            Some(s) => {
                let key = self.shaders.borrow_mut().insert(s);
                Ok(key)
            }
            None => Err(String::from("Unable to create shader object")),
        }
    }

    unsafe fn is_shader(&self, shader: Self::Shader) -> bool {
        let shaders = self.shaders.borrow_mut();
        if let Some(ref s) = shaders.get(shader) {
            self.raw.is_shader(Some(s))
        } else {
            false
        }
    }

    unsafe fn create_texture(&self) -> Result<Self::Texture, String> {
        let raw_texture = self.raw.create_texture();

        match raw_texture {
            Some(t) => {
                let key = self.textures.borrow_mut().insert(t);
                Ok(key)
            }
            None => Err(String::from("Unable to create texture object")),
        }
    }

    unsafe fn create_named_texture(&self, _target: u32) -> Result<Self::Texture, String> {
        unimplemented!()
    }

    unsafe fn is_texture(&self, texture: Self::Texture) -> bool {
        let textures = self.textures.borrow_mut();
        if let Some(ref t) = textures.get(texture) {
            self.raw.is_texture(Some(t))
        } else {
            false
        }
    }

    unsafe fn delete_shader(&self, shader: Self::Shader) {
        let mut shaders = self.shaders.borrow_mut();
        if let Some(ref s) = shaders.remove(shader) {
            self.raw.delete_shader(Some(s));
        }
    }

    unsafe fn shader_source(&self, shader: Self::Shader, source: &str) {
        let shaders = self.shaders.borrow();
        let raw_shader = shaders.get_unchecked(shader);
        self.raw.shader_source(raw_shader, source);
    }

    unsafe fn compile_shader(&self, shader: Self::Shader) {
        let shaders = self.shaders.borrow();
        let raw_shader = shaders.get_unchecked(shader);
        self.raw.compile_shader(raw_shader);
    }

    unsafe fn get_shader_completion_status(&self, shader: Self::Shader) -> bool {
        let shaders = self.shaders.borrow();
        let raw_shader = shaders.get_unchecked(shader);
        if self.extensions.khr_parallel_shader_compile.is_none() {
            panic!("Parallel shader compile is not supported")
        }

        self.raw
            .get_shader_parameter(raw_shader, COMPLETION_STATUS)
            .as_bool()
            .unwrap_or(false)
    }

    unsafe fn get_shader_compile_status(&self, shader: Self::Shader) -> bool {
        let shaders = self.shaders.borrow();
        let raw_shader = shaders.get_unchecked(shader);

        self.raw
            .get_shader_parameter(raw_shader, COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
    }

    unsafe fn get_shader_info_log(&self, shader: Self::Shader) -> String {
        let shaders = self.shaders.borrow();
        let raw_shader = shaders.get_unchecked(shader);
        self.raw
            .get_shader_info_log(raw_shader)
            .unwrap_or_else(|| String::from(""))
    }

    unsafe fn create_program(&self) -> Result<Self::Program, String> {
        let raw_program = self.raw.create_program();

        match raw_program {
            Some(p) => {
                let key = self.programs.borrow_mut().insert(p);
                Ok(key)
            }
            None => Err(String::from("Unable to create program object")),
        }
    }

    unsafe fn is_program(&self, program: Self::Program) -> bool {
        let programs = self.programs.borrow_mut();
        if let Some(ref p) = programs.get(program) {
            self.raw.is_program(Some(p))
        } else {
            false
        }
    }

    unsafe fn delete_program(&self, program: Self::Program) {
        let mut programs = self.programs.borrow_mut();
        if let Some(ref p) = programs.remove(program) {
            self.raw.delete_program(Some(p))
        }
    }

    unsafe fn attach_shader(&self, program: Self::Program, shader: Self::Shader) {
        let programs = self.programs.borrow();
        let shaders = self.shaders.borrow();
        let raw_program = programs.get_unchecked(program);
        let raw_shader = shaders.get_unchecked(shader);
        self.raw.attach_shader(raw_program, raw_shader);
    }

    unsafe fn detach_shader(&self, program: Self::Program, shader: Self::Shader) {
        let programs = self.programs.borrow();
        let shaders = self.shaders.borrow();
        let raw_program = programs.get_unchecked(program);
        let raw_shader = shaders.get_unchecked(shader);
        self.raw.detach_shader(raw_program, raw_shader);
    }

    unsafe fn link_program(&self, program: Self::Program) {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        self.raw.link_program(raw_program);
    }

    unsafe fn get_program_completion_status(&self, program: Self::Program) -> bool {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        if self.extensions.khr_parallel_shader_compile.is_none() {
            panic!("Parallel shader compile is not supported")
        }

        self.raw
            .get_program_parameter(raw_program, COMPLETION_STATUS)
            .as_bool()
            .unwrap_or(false)
    }

    unsafe fn get_program_link_status(&self, program: Self::Program) -> bool {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        self.raw
            .get_program_parameter(raw_program, LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
    }

    unsafe fn get_program_info_log(&self, program: Self::Program) -> String {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        self.raw
            .get_program_info_log(raw_program)
            .unwrap_or_else(|| String::from(""))
    }

    unsafe fn get_active_uniforms(&self, program: Self::Program) -> u32 {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        self.raw
            .get_program_parameter(raw_program, WebGl2RenderingContext::ACTIVE_UNIFORMS)
            .as_f64()
            .map(|v| v as u32)
            .unwrap_or(0)
    }

    unsafe fn get_active_uniform(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveUniform> {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        self.raw
            .get_active_uniform(raw_program, index)
            .map(|au| ActiveUniform {
                size: au.size(),
                utype: au.type_(),
                name: au.name(),
            })
    }

    unsafe fn use_program(&self, program: Option<Self::Program>) {
        let programs = self.programs.borrow();
        let raw_program = program.map(|p| programs.get_unchecked(p));
        self.raw.use_program(raw_program);
    }

    unsafe fn create_buffer(&self) -> Result<Self::Buffer, String> {
        let raw_buffer = self.raw.create_buffer();

        match raw_buffer {
            Some(p) => {
                let key = self.buffers.borrow_mut().insert(p);
                Ok(key)
            }
            None => Err(String::from("Unable to create buffer object")),
        }
    }

    unsafe fn create_named_buffer(&self) -> Result<Self::Buffer, String> {
        unimplemented!()
    }

    unsafe fn is_buffer(&self, buffer: Self::Buffer) -> bool {
        let buffers = self.buffers.borrow_mut();
        if let Some(ref b) = buffers.get(buffer) {
            self.raw.is_buffer(Some(b))
        } else {
            false
        }
    }

    unsafe fn bind_buffer(&self, target: u32, buffer: Option<Self::Buffer>) {
        let buffers = self.buffers.borrow();
        let raw_buffer = buffer.map(|b| buffers.get_unchecked(b));
        self.raw.bind_buffer(target, raw_buffer);
    }

    unsafe fn bind_buffer_base(&self, target: u32, index: u32, buffer: Option<Self::Buffer>) {
        let buffers = self.buffers.borrow();
        let raw_buffer = buffer.map(|b| buffers.get_unchecked(b));
        self.raw.bind_buffer_base(target, index, raw_buffer);
    }

    unsafe fn bind_buffer_range(
        &self,
        target: u32,
        index: u32,
        buffer: Option<Self::Buffer>,
        offset: i32,
        size: i32,
    ) {
        let buffers = self.buffers.borrow();
        let raw_buffer = buffer.map(|b| buffers.get_unchecked(b));

        self.raw
            .bind_buffer_range_with_i32_and_i32(target, index, raw_buffer, offset, size);
    }

    unsafe fn bind_framebuffer(&self, target: u32, framebuffer: Option<Self::Framebuffer>) {
        let framebuffers = self.framebuffers.borrow();
        let raw_framebuffer = framebuffer.map(|f| framebuffers.get_unchecked(f));
        self.raw.bind_framebuffer(target, raw_framebuffer);
    }

    unsafe fn bind_renderbuffer(&self, target: u32, renderbuffer: Option<Self::Renderbuffer>) {
        let renderbuffers = self.renderbuffers.borrow();
        let raw_renderbuffer = renderbuffer.map(|r| renderbuffers.get_unchecked(r));
        self.raw.bind_renderbuffer(target, raw_renderbuffer);
    }

    unsafe fn blit_framebuffer(
        &self,
        src_x0: i32,
        src_y0: i32,
        src_x1: i32,
        src_y1: i32,
        dst_x0: i32,
        dst_y0: i32,
        dst_x1: i32,
        dst_y1: i32,
        mask: u32,
        filter: u32,
    ) {
        self.raw.blit_framebuffer(
            src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter,
        );
    }

    unsafe fn create_vertex_array(&self) -> Result<Self::VertexArray, String> {
        let raw_vertex_array = self.raw.create_vertex_array();

        match raw_vertex_array {
            Some(va) => {
                let key = self.vertex_arrays.borrow_mut().insert(va);
                Ok(key)
            }
            None => Err(String::from("Unable to create vertex array object")),
        }
    }

    unsafe fn delete_vertex_array(&self, vertex_array: Self::VertexArray) {
        let mut vertex_arrays = self.vertex_arrays.borrow_mut();
        if let Some(ref va) = vertex_arrays.remove(vertex_array) {
            self.raw.delete_vertex_array(Some(va));
        }
    }

    unsafe fn bind_vertex_array(&self, vertex_array: Option<Self::VertexArray>) {
        let vertex_arrays = self.vertex_arrays.borrow();
        let raw_vertex_array = vertex_array.map(|va| vertex_arrays.get_unchecked(va));
        self.raw.bind_vertex_array(raw_vertex_array);
    }

    unsafe fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.raw.clear_color(red, green, blue, alpha);
    }

    unsafe fn supports_f64_precision() -> bool {
        false
    }

    unsafe fn clear_depth_f32(&self, depth: f32) {
        self.raw.clear_depth(depth);
    }

    unsafe fn clear_stencil(&self, stencil: i32) {
        self.raw.clear_stencil(stencil);
    }

    unsafe fn clear(&self, mask: u32) {
        self.raw.clear(mask);
    }

    unsafe fn pixel_store_i32(&self, parameter: u32, value: i32) {
        self.raw.pixel_storei(parameter, value);
    }

    unsafe fn pixel_store_bool(&self, parameter: u32, value: bool) {
        self.raw.pixel_storei(parameter, value as i32);
    }

    unsafe fn buffer_data_size(&self, target: u32, size: i32, usage: u32) {
        self.raw.buffer_data_with_i32(target, size, usage);
    }

    unsafe fn buffer_data_u8_slice(&self, target: u32, data: &[u8], usage: u32) {
        let array = js_sys::Uint8Array::view(data);
        self.raw
            .buffer_data_with_array_buffer_view(target, &array, usage);
    }

    unsafe fn named_buffer_data_u8_slice(&self, _buffer: Self::Buffer, _data: &[u8], _usage: u32) {
        unimplemented!()
    }

    unsafe fn buffer_sub_data_u8_slice(&self, target: u32, offset: i32, src_data: &[u8]) {
        let array = js_sys::Uint8Array::view(src_data);
        self.raw
            .buffer_sub_data_with_i32_and_array_buffer_view(target, offset, &array);
    }

    unsafe fn get_buffer_sub_data(&self, target: u32, offset: i32, dst_data: &mut [u8]) {
        let array = js_sys::Uint8Array::view(dst_data);
        self.raw
            .get_buffer_sub_data_with_i32_and_array_buffer_view(target, offset, &array);
    }

    unsafe fn check_framebuffer_status(&self, target: u32) -> u32 {
        self.raw.check_framebuffer_status(target)
    }

    unsafe fn clear_buffer_i32_slice(&self, target: u32, draw_buffer: u32, values: &[i32]) {
        self.raw
            .clear_bufferiv_with_i32_array(target, draw_buffer as i32, values);
    }

    unsafe fn clear_buffer_u32_slice(&self, target: u32, draw_buffer: u32, values: &[u32]) {
        self.raw
            .clear_bufferuiv_with_u32_array(target, draw_buffer as i32, values)
    }

    unsafe fn clear_buffer_f32_slice(&self, target: u32, draw_buffer: u32, values: &[f32]) {
        self.raw
            .clear_bufferfv_with_f32_array(target, draw_buffer as i32, values)
    }

    unsafe fn clear_buffer_depth_stencil(
        &self,
        target: u32,
        draw_buffer: u32,
        depth: f32,
        stencil: i32,
    ) {
        self.raw
            .clear_bufferfi(target, draw_buffer as i32, depth, stencil)
    }

    unsafe fn client_wait_sync(&self, fence: Self::Fence, flags: u32, timeout: i32) -> u32 {
        let fences = self.fences.borrow();
        let raw_fence = fences.get_unchecked(fence);

        self.raw
            .client_wait_sync_with_u32(raw_fence, flags, timeout as u32)
    }

    unsafe fn wait_sync(&self, fence: Self::Fence, flags: u32, timeout: u64) {
        let fences = self.fences.borrow();
        let raw_fence = fences.get_unchecked(fence);

        self.raw
            .wait_sync_with_i32(raw_fence, flags, timeout as i32)
    }

    unsafe fn copy_buffer_sub_data(
        &self,
        src_target: u32,
        dst_target: u32,
        src_offset: i32,
        dst_offset: i32,
        size: i32,
    ) {
        self.raw.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            src_target, dst_target, src_offset, dst_offset, size,
        );
    }

    unsafe fn copy_image_sub_data(
        &self,
        _src_name: Self::Texture,
        _src_target: u32,
        _src_level: i32,
        _src_x: i32,
        _src_y: i32,
        _src_z: i32,
        _dst_name: Self::Texture,
        _dst_target: u32,
        _dst_level: i32,
        _dst_x: i32,
        _dst_y: i32,
        _dst_z: i32,
        _src_width: i32,
        _src_height: i32,
        _src_depth: i32,
    ) {
        unimplemented!()
    }

    unsafe fn copy_tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        border: i32,
    ) {
        self.raw
            .copy_tex_image_2d(target, level, internal_format, x, y, width, height, border);
    }

    unsafe fn copy_tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        self.raw
            .copy_tex_sub_image_2d(target, level, x_offset, y_offset, x, y, width, height);
    }

    unsafe fn copy_tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        self.raw.copy_tex_sub_image_3d(
            target, level, x_offset, y_offset, z_offset, x, y, width, height,
        );
    }

    unsafe fn delete_buffer(&self, buffer: Self::Buffer) {
        let mut buffers = self.buffers.borrow_mut();
        if let Some(ref b) = buffers.remove(buffer) {
            self.raw.delete_buffer(Some(b));
        }
    }

    unsafe fn delete_framebuffer(&self, framebuffer: Self::Framebuffer) {
        let mut framebuffers = self.framebuffers.borrow_mut();
        if let Some(ref f) = framebuffers.remove(framebuffer) {
            self.raw.delete_framebuffer(Some(f));
        }
    }

    unsafe fn delete_query(&self, query: Self::Query) {
        let mut queries = self.queries.borrow_mut();
        if let Some(ref r) = queries.remove(query) {
            self.raw.delete_query(Some(r));
        }
    }

    unsafe fn delete_renderbuffer(&self, renderbuffer: Self::Renderbuffer) {
        let mut renderbuffers = self.renderbuffers.borrow_mut();
        if let Some(ref r) = renderbuffers.remove(renderbuffer) {
            self.raw.delete_renderbuffer(Some(r));
        }
    }

    unsafe fn delete_sampler(&self, sampler: Self::Sampler) {
        let mut samplers = self.samplers.borrow_mut();
        if let Some(ref s) = samplers.remove(sampler) {
            self.raw.delete_sampler(Some(s));
        }
    }

    unsafe fn delete_sync(&self, fence: Self::Fence) {
        let mut fences = self.fences.borrow_mut();
        if let Some(ref f) = fences.remove(fence) {
            self.raw.delete_sync(Some(f));
        }
    }

    unsafe fn delete_texture(&self, texture: Self::Texture) {
        let mut textures = self.textures.borrow_mut();
        if let Some(ref t) = textures.remove(texture) {
            self.raw.delete_texture(Some(t));
        }
    }

    unsafe fn disable(&self, parameter: u32) {
        self.raw.disable(parameter)
    }

    unsafe fn disable_vertex_attrib_array(&self, index: u32) {
        self.raw.disable_vertex_attrib_array(index)
    }

    unsafe fn draw_arrays(&self, mode: u32, first: i32, count: i32) {
        self.raw.draw_arrays(mode as u32, first, count)
    }

    unsafe fn draw_arrays_instanced(&self, mode: u32, first: i32, count: i32, instance_count: i32) {
        self.raw
            .draw_arrays_instanced(mode as u32, first, count, instance_count)
    }

    unsafe fn multi_draw_arrays_instanced(
        &self,
        mode: u32,
        draw_commands: Vec<DrawArraysIndirectCommand>,
    ) {
        let Some(ext) = &self.extensions.webgl_multi_draw else {
            panic!("Multi draw is not supported");
        };

        let draw_count = draw_commands.len() as i32;

        let mut firsts_list = Vec::new();
        let mut counts_list = Vec::new();
        let mut instance_counts_list = Vec::new();

        for command in draw_commands {
            firsts_list.push(command.first as i32);
            counts_list.push(command.count as i32);
            instance_counts_list.push(command.instance_count as i32);
        }

        // info!("calling multi_draw_arrays({:?}, {:?}, {:?}, {:?})", firsts_list, counts_list, instance_counts_list, draw_count);

        ext.multi_draw_arrays_instanced_webgl_with_i32_array_and_i32_array_and_i32_array(
            mode,
            &mut firsts_list,
            0,
            &mut counts_list,
            0,
            &mut instance_counts_list,
            0,
            draw_count,
        );
    }

    unsafe fn draw_buffer(&self, draw_buffer: u32) {
        self.raw.draw_buffers(&Array::of1(&draw_buffer.into()));
    }

    unsafe fn draw_buffers(&self, buffers: &[u32]) {
        let js_buffers = Array::new();
        for &b in buffers {
            js_buffers.push(&b.into());
        }
        self.raw.draw_buffers(&js_buffers);
    }

    unsafe fn draw_elements(&self, mode: u32, count: i32, element_type: u32, offset: i32) {
        self.raw
            .draw_elements_with_i32(mode as u32, count, element_type as u32, offset);
    }

    unsafe fn draw_elements_instanced(
        &self,
        mode: u32,
        count: i32,
        element_type: u32,
        offset: i32,
        instance_count: i32,
    ) {
        self.raw.draw_elements_instanced_with_i32(
            mode as u32,
            count,
            element_type as u32,
            offset,
            instance_count,
        );
    }

    unsafe fn enable(&self, parameter: u32) {
        self.raw.enable(parameter);
    }

    unsafe fn is_enabled(&self, parameter: u32) -> bool {
        self.raw.is_enabled(parameter)
    }

    unsafe fn enable_vertex_array_attrib(&self, _vao: Self::VertexArray, _index: u32) {
        unimplemented!()
    }

    unsafe fn enable_vertex_attrib_array(&self, index: u32) {
        self.raw.enable_vertex_attrib_array(index);
    }

    unsafe fn flush(&self) {
        self.raw.flush();
    }

    unsafe fn framebuffer_renderbuffer(
        &self,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
        renderbuffer: Option<Self::Renderbuffer>,
    ) {
        let renderbuffers = self.renderbuffers.borrow();
        let raw_renderbuffer = renderbuffer.map(|r| renderbuffers.get_unchecked(r));
        self.raw.framebuffer_renderbuffer(
            target,
            attachment,
            renderbuffer_target,
            raw_renderbuffer,
        );
    }

    unsafe fn framebuffer_texture_2d(
        &self,
        target: u32,
        attachment: u32,
        texture_target: u32,
        texture: Option<Self::Texture>,
        level: i32,
    ) {
        let textures = self.textures.borrow();
        let raw_texture = texture.map(|t| textures.get_unchecked(t));

        self.raw
            .framebuffer_texture_2d(target, attachment, texture_target, raw_texture, level);
    }

    unsafe fn framebuffer_texture_layer(
        &self,
        target: u32,
        attachment: u32,
        texture: Option<Self::Texture>,
        level: i32,
        layer: i32,
    ) {
        let textures = self.textures.borrow();
        let raw_texture = texture.map(|t| textures.get_unchecked(t));

        self.raw
            .framebuffer_texture_layer(target, attachment, raw_texture, level, layer);
    }

    unsafe fn front_face(&self, value: u32) {
        self.raw.front_face(value as u32);
    }

    unsafe fn get_error(&self) -> u32 {
        self.raw.get_error()
    }

    unsafe fn get_tex_parameter_i32(&self, target: u32, parameter: u32) -> i32 {
        self.raw
            .get_tex_parameter(target, parameter)
            .as_f64()
            .map(|v| v as i32)
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or(0)
    }

    unsafe fn get_buffer_parameter_i32(&self, target: u32, parameter: u32) -> i32 {
        self.raw
            .get_buffer_parameter(target, parameter)
            .as_f64()
            .map(|v| v as i32)
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or(0)
    }

    unsafe fn get_parameter_i32(&self, parameter: u32) -> i32 {
        self.raw
            .get_parameter(parameter)
            .unwrap()
            .as_f64()
            .map(|v| v as i32)
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or(0)
    }

    unsafe fn get_parameter_i32_slice(&self, parameter: u32, v: &mut [i32]) {
        let value = self.raw.get_parameter(parameter).unwrap();
        use wasm_bindgen::JsCast;
        if let Some(value) = value.as_f64() {
            v[0] = value as i32;
        } else if let Some(values) = value.dyn_ref::<js_sys::Int32Array>() {
            values.copy_to(v)
        }
    }

    unsafe fn get_parameter_f32(&self, parameter: u32) -> f32 {
        self.raw
            .get_parameter(parameter)
            .unwrap()
            .as_f64()
            .map(|v| v as f32)
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or(0.0)
    }

    unsafe fn get_parameter_f32_slice(&self, parameter: u32, v: &mut [f32]) {
        let value = self.raw.get_parameter(parameter).unwrap();
        use wasm_bindgen::JsCast;
        if let Some(value) = value.as_f64() {
            v[0] = value as f32;
        } else if let Some(values) = value.dyn_ref::<js_sys::Float32Array>() {
            values.copy_to(v)
        }
    }

    unsafe fn get_parameter_indexed_i32(&self, parameter: u32, index: u32) -> i32 {
        self.raw
            .get_indexed_parameter(parameter, index)
            .unwrap()
            .as_f64()
            .map(|v| v as i32)
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or(0)
    }

    unsafe fn get_parameter_indexed_string(&self, parameter: u32, index: u32) -> String {
        self.raw
            .get_indexed_parameter(parameter, index)
            .unwrap()
            .as_string()
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or_else(|| String::from(""))
    }

    unsafe fn get_parameter_string(&self, parameter: u32) -> String {
        self.raw
            .get_parameter(parameter)
            .unwrap()
            .as_string()
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or_else(|| String::from(""))
    }

    unsafe fn get_uniform_location(
        &self,
        program: Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation> {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        self.raw.get_uniform_location(raw_program, name)
    }

    unsafe fn get_attrib_location(&self, program: Self::Program, name: &str) -> Option<u32> {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        let attrib_location = self.raw.get_attrib_location(raw_program, name);
        if attrib_location < 0 {
            None
        } else {
            Some(attrib_location as u32)
        }
    }

    unsafe fn bind_attrib_location(&self, program: Self::Program, index: u32, name: &str) {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        self.raw.bind_attrib_location(raw_program, index, name)
    }

    unsafe fn get_active_attributes(&self, program: Self::Program) -> u32 {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        self.raw
            .get_program_parameter(raw_program, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
            .as_f64()
            .map(|v| v as u32)
            .unwrap_or(0)
    }

    unsafe fn get_active_attribute(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveAttribute> {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        self.raw
            .get_active_attrib(raw_program, index)
            .map(|au| ActiveAttribute {
                size: au.size(),
                atype: au.type_(),
                name: au.name(),
            })
    }

    unsafe fn get_sync_status(&self, fence: Self::Fence) -> u32 {
        let fences = self.fences.borrow();
        let raw_fence = fences.get_unchecked(fence);
        self.raw
            .get_sync_parameter(raw_fence, SYNC_STATUS)
            .as_f64()
            .map(|v| v as u32)
            .unwrap_or(UNSIGNALED)
    }

    unsafe fn is_sync(&self, fence: Self::Fence) -> bool {
        let fences = self.fences.borrow();
        let raw_fence = fences.get_unchecked(fence);
        self.raw.is_sync(Some(raw_fence))
    }

    unsafe fn renderbuffer_storage(
        &self,
        target: u32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        self.raw
            .renderbuffer_storage(target, internal_format, width, height);
    }

    unsafe fn renderbuffer_storage_multisample(
        &self,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        self.raw
            .renderbuffer_storage_multisample(target, samples, internal_format, width, height);
    }

    unsafe fn sampler_parameter_f32(&self, sampler: Self::Sampler, name: u32, value: f32) {
        let samplers = self.samplers.borrow();
        let raw_sampler = samplers.get_unchecked(sampler);

        self.raw.sampler_parameterf(raw_sampler, name, value);
    }

    unsafe fn sampler_parameter_i32(&self, sampler: Self::Sampler, name: u32, value: i32) {
        let samplers = self.samplers.borrow();
        let raw_sampler = samplers.get_unchecked(sampler);

        self.raw.sampler_parameteri(raw_sampler, name, value);
    }

    unsafe fn generate_mipmap(&self, target: u32) {
        self.raw.generate_mipmap(target);
    }

    unsafe fn generate_texture_mipmap(&self, _texture: Self::Texture) {
        unimplemented!()
    }

    unsafe fn tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        ty: u32,
        pixels: Option<&[u8]>,
    ) {
        let pixels = pixels.map(|bytes| texture_data_view(ty, bytes));

        // TODO: Handle return value?
        self.raw
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                ty,
                pixels.as_ref(),
            )
            .unwrap();
    }

    unsafe fn compressed_tex_image_2d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        _image_size: i32,
        pixels: &[u8],
    ) {
        let src_data = js_sys::Uint8Array::view(pixels);
        self.raw.compressed_tex_image_2d_with_array_buffer_view(
            target,
            level,
            internal_format as u32,
            width,
            height,
            border,
            &src_data,
        );
    }

    unsafe fn tex_image_3d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        ty: u32,
        pixels: Option<&[u8]>,
    ) {
        let pixels = pixels.map(|bytes| texture_data_view(ty, bytes));
        // TODO: Handle return value?
        self.raw
            .tex_image_3d_with_opt_array_buffer_view(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                ty,
                pixels.as_ref(),
            )
            .unwrap();
    }

    unsafe fn compressed_tex_image_3d(
        &self,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        _image_size: i32,
        pixels: &[u8],
    ) {
        let src_data = js_sys::Uint8Array::view(pixels);
        self.raw.compressed_tex_image_3d_with_array_buffer_view(
            target,
            level,
            internal_format as u32,
            width,
            height,
            depth,
            border,
            &src_data,
        );
    }

    unsafe fn tex_storage_2d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) {
        self.raw
            .tex_storage_2d(target, levels, internal_format, width, height);
    }

    unsafe fn tex_storage_3d(
        &self,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    ) {
        self.raw
            .tex_storage_3d(target, levels, internal_format, width, height, depth);
    }

    unsafe fn uniform_1_i32(&self, uniform_location: Option<&Self::UniformLocation>, x: i32) {
        self.raw.uniform1i(uniform_location, x);
    }

    unsafe fn uniform_2_i32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
    ) {
        self.raw.uniform2i(uniform_location, x, y);
    }

    unsafe fn uniform_3_i32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
    ) {
        self.raw.uniform3i(uniform_location, x, y, z);
    }

    unsafe fn uniform_4_i32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    ) {
        self.raw.uniform4i(uniform_location, x, y, z, w);
    }

    unsafe fn uniform_1_i32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[i32],
    ) {
        self.raw.uniform1iv_with_i32_array(uniform_location, v)
    }

    unsafe fn uniform_2_i32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[i32],
    ) {
        self.raw.uniform2iv_with_i32_array(uniform_location, v)
    }

    unsafe fn uniform_3_i32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[i32],
    ) {
        self.raw.uniform3iv_with_i32_array(uniform_location, v)
    }

    unsafe fn uniform_4_i32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[i32],
    ) {
        self.raw.uniform4iv_with_i32_array(uniform_location, v)
    }

    unsafe fn uniform_1_u32(&self, uniform_location: Option<&Self::UniformLocation>, x: u32) {
        self.raw.uniform1ui(uniform_location, x);
    }

    unsafe fn uniform_2_u32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
    ) {
        self.raw.uniform2ui(uniform_location, x, y);
    }

    unsafe fn uniform_3_u32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
    ) {
        self.raw.uniform3ui(uniform_location, x, y, z);
    }

    unsafe fn uniform_4_u32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: u32,
        y: u32,
        z: u32,
        w: u32,
    ) {
        self.raw.uniform4ui(uniform_location, x, y, z, w);
    }

    unsafe fn uniform_1_u32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[u32],
    ) {
        self.raw.uniform1uiv_with_u32_array(uniform_location, v)
    }

    unsafe fn uniform_2_u32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[u32],
    ) {
        self.raw.uniform2uiv_with_u32_array(uniform_location, v)
    }

    unsafe fn uniform_3_u32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[u32],
    ) {
        self.raw.uniform3uiv_with_u32_array(uniform_location, v)
    }

    unsafe fn uniform_4_u32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[u32],
    ) {
        self.raw.uniform4uiv_with_u32_array(uniform_location, v)
    }

    unsafe fn uniform_1_f32(&self, uniform_location: Option<&Self::UniformLocation>, x: f32) {
        self.raw.uniform1f(uniform_location, x);
    }

    unsafe fn uniform_2_f32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
    ) {
        self.raw.uniform2f(uniform_location, x, y);
    }

    unsafe fn uniform_3_f32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
    ) {
        self.raw.uniform3f(uniform_location, x, y, z);
    }

    unsafe fn uniform_4_f32(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) {
        self.raw.uniform4f(uniform_location, x, y, z, w);
    }

    unsafe fn uniform_1_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[f32],
    ) {
        self.raw.uniform1fv_with_f32_array(uniform_location, v)
    }

    unsafe fn uniform_2_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[f32],
    ) {
        self.raw.uniform2fv_with_f32_array(uniform_location, v)
    }

    unsafe fn uniform_3_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[f32],
    ) {
        self.raw.uniform3fv_with_f32_array(uniform_location, v)
    }

    unsafe fn uniform_4_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        v: &[f32],
    ) {
        self.raw.uniform4fv_with_f32_array(uniform_location, v)
    }

    unsafe fn uniform_matrix_2_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix2fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_2x3_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix2x3fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_2x4_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix2x4fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_3x2_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix3x2fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_3_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix3fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_3x4_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix3x4fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_4x2_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix4x2fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_4x3_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix4x3fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn uniform_matrix_4_f32_slice(
        &self,
        uniform_location: Option<&Self::UniformLocation>,
        transpose: bool,
        v: &[f32],
    ) {
        self.raw
            .uniform_matrix4fv_with_f32_array(uniform_location, transpose, v)
    }

    unsafe fn cull_face(&self, value: u32) {
        self.raw.cull_face(value as u32);
    }

    unsafe fn color_mask(&self, red: bool, green: bool, blue: bool, alpha: bool) {
        self.raw.color_mask(red, green, blue, alpha);
    }

    unsafe fn depth_mask(&self, value: bool) {
        self.raw.depth_mask(value);
    }

    unsafe fn blend_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.raw.blend_color(red, green, blue, alpha);
    }

    unsafe fn line_width(&self, width: f32) {
        self.raw.line_width(width);
    }

    unsafe fn invalidate_framebuffer(&self, target: u32, attachments: &[u32]) {
        let js_attachments = Array::new();
        for &a in attachments {
            js_attachments.push(&a.into());
        }
        self.raw
            .invalidate_framebuffer(target, &js_attachments)
            .unwrap();
    }

    unsafe fn polygon_offset(&self, factor: f32, units: f32) {
        self.raw.polygon_offset(factor, units);
    }

    unsafe fn polygon_mode(&self, _face: u32, mode: u32) {
        if mode != FILL {
            panic!("Polygon mode other than FILL is not supported");
        }
    }

    unsafe fn finish(&self) {
        self.raw.finish();
    }

    unsafe fn bind_texture(&self, target: u32, texture: Option<Self::Texture>) {
        let textures = self.textures.borrow();
        let raw_texture = texture.map(|t| textures.get_unchecked(t));
        self.raw.bind_texture(target, raw_texture);
    }

    unsafe fn bind_sampler(&self, unit: u32, sampler: Option<Self::Sampler>) {
        let samplers = self.samplers.borrow();
        let raw_sampler = sampler.map(|s| samplers.get_unchecked(s));
        self.raw.bind_sampler(unit, raw_sampler);
    }

    unsafe fn active_texture(&self, unit: u32) {
        self.raw.active_texture(unit);
    }

    unsafe fn fence_sync(&self, condition: u32, flags: u32) -> Result<Self::Fence, String> {
        let raw_fence = self.raw.fence_sync(condition as u32, flags);
        match raw_fence {
            Some(f) => {
                let key = self.fences.borrow_mut().insert(f);
                Ok(key)
            }
            None => Err(String::from("Unable to create fence object")),
        }
    }

    unsafe fn tex_parameter_f32(&self, target: u32, parameter: u32, value: f32) {
        self.raw.tex_parameterf(target, parameter, value);
    }

    unsafe fn tex_parameter_i32(&self, target: u32, parameter: u32, value: i32) {
        self.raw.tex_parameteri(target, parameter, value);
    }

    unsafe fn texture_parameter_i32(&self, _texture: Self::Texture, _parameter: u32, _value: i32) {
        unimplemented!()
    }

    unsafe fn tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        ty: u32,
        pixels: PixelUnpackData,
    ) {
        match pixels {
            PixelUnpackData::BufferOffset(offset) => self
                .raw
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_i32(
                    target,
                    level,
                    x_offset,
                    y_offset,
                    width,
                    height,
                    format,
                    ty,
                    offset as i32,
                ),
            PixelUnpackData::Slice(data) => {
                let data = texture_data_view(ty, data);
                self.raw
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                        target,
                        level,
                        x_offset,
                        y_offset,
                        width,
                        height,
                        format,
                        ty,
                        Some(&data),
                    )
            }
        }
        .unwrap(); // TODO: Handle return value?
    }

    unsafe fn texture_sub_image_3d(
        &self,
        _texture: Self::Texture,
        _level: i32,
        _x_offset: i32,
        _y_offset: i32,
        _z_offset: i32,
        _width: i32,
        _height: i32,
        _depth: i32,
        _format: u32,
        _ty: u32,
        _pixels: PixelUnpackData,
    ) {
        unimplemented!()
    }

    unsafe fn compressed_tex_sub_image_2d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: u32,
        pixels: CompressedPixelUnpackData,
    ) {
        match pixels {
            CompressedPixelUnpackData::BufferRange(range) => {
                self.raw.compressed_tex_sub_image_2d_with_i32_and_i32(
                    target,
                    level,
                    x_offset,
                    y_offset,
                    width,
                    height,
                    format,
                    (range.end - range.start) as i32,
                    range.start as i32,
                )
            }
            CompressedPixelUnpackData::Slice(data) => {
                let data = texture_data_view(BYTE, data);
                self.raw.compressed_tex_sub_image_2d_with_array_buffer_view(
                    target, level, x_offset, y_offset, width, height, format, &data,
                )
            }
        }
    }

    unsafe fn tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        ty: u32,
        pixels: PixelUnpackData,
    ) {
        match pixels {
            PixelUnpackData::BufferOffset(offset) => self.raw.tex_sub_image_3d_with_i32(
                target,
                level,
                x_offset,
                y_offset,
                z_offset,
                width,
                height,
                depth,
                format,
                ty,
                offset as i32,
            ),
            PixelUnpackData::Slice(slice) => {
                let slice = texture_data_view(ty, slice);
                self.raw.tex_sub_image_3d_with_opt_array_buffer_view(
                    target,
                    level,
                    x_offset,
                    y_offset,
                    z_offset,
                    width,
                    height,
                    depth,
                    format,
                    ty,
                    Some(&slice),
                )
            }
        }
        .unwrap(); // TODO: Handle return value?
    }

    unsafe fn compressed_tex_sub_image_3d(
        &self,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        pixels: CompressedPixelUnpackData,
    ) {
        match pixels {
            CompressedPixelUnpackData::BufferRange(range) => {
                self.raw.compressed_tex_sub_image_3d_with_i32_and_i32(
                    target,
                    level,
                    x_offset,
                    y_offset,
                    z_offset,
                    width,
                    height,
                    depth,
                    format,
                    (range.end - range.start) as i32,
                    range.start as i32,
                )
            }
            CompressedPixelUnpackData::Slice(data) => {
                let data = texture_data_view(BYTE, data);
                self.raw.compressed_tex_sub_image_3d_with_array_buffer_view(
                    target, level, x_offset, y_offset, z_offset, width, height, depth, format,
                    &data,
                )
            }
        }
    }

    unsafe fn depth_func(&self, func: u32) {
        self.raw.depth_func(func as u32);
    }

    unsafe fn depth_range_f32(&self, near: f32, far: f32) {
        self.raw.depth_range(near, far);
    }

    unsafe fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.raw.scissor(x, y, width, height);
    }

    unsafe fn vertex_array_attrib_binding_f32(
        &self,
        _vao: Self::VertexArray,
        _index: u32,
        _binding_index: u32,
    ) {
        unimplemented!()
    }

    unsafe fn vertex_array_attrib_format_f32(
        &self,
        _vao: Self::VertexArray,
        _index: u32,
        _size: i32,
        _data_type: u32,
        _normalized: bool,
        _relative_offset: u32,
    ) {
        unimplemented!()
    }

    unsafe fn vertex_array_attrib_format_i32(
        &self,
        _vao: Self::VertexArray,
        _index: u32,
        _size: i32,
        _data_type: u32,
        _relative_offset: u32,
    ) {
        unimplemented!()
    }

    unsafe fn vertex_array_element_buffer(
        &self,
        _vao: Self::VertexArray,
        _buffer: Option<Self::Buffer>,
    ) {
        unimplemented!()
    }

    unsafe fn vertex_array_vertex_buffer(
        &self,
        _vao: Self::VertexArray,
        _binding_index: u32,
        _buffer: Option<Self::Buffer>,
        _offset: i32,
        _stride: i32,
    ) {
        unimplemented!()
    }

    unsafe fn vertex_attrib_divisor(&self, index: u32, divisor: u32) {
        self.raw.vertex_attrib_divisor(index, divisor);
    }

    unsafe fn vertex_attrib_pointer_f32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        self.raw
            .vertex_attrib_pointer_with_i32(index, size, data_type, normalized, stride, offset);
    }

    unsafe fn vertex_attrib_pointer_i32(
        &self,
        index: u32,
        size: i32,
        data_type: u32,
        stride: i32,
        offset: i32,
    ) {
        self.raw
            .vertex_attrib_i_pointer_with_i32(index, size, data_type, stride, offset)
    }

    unsafe fn vertex_attrib_1_f32(&self, index: u32, x: f32) {
        self.raw.vertex_attrib1f(index, x);
    }

    unsafe fn vertex_attrib_2_f32(&self, index: u32, x: f32, y: f32) {
        self.raw.vertex_attrib2f(index, x, y);
    }

    unsafe fn vertex_attrib_3_f32(&self, index: u32, x: f32, y: f32, z: f32) {
        self.raw.vertex_attrib3f(index, x, y, z);
    }

    unsafe fn vertex_attrib_4_f32(&self, index: u32, x: f32, y: f32, z: f32, w: f32) {
        self.raw.vertex_attrib4f(index, x, y, z, w);
    }

    unsafe fn vertex_attrib_1_f32_slice(&self, index: u32, v: &[f32]) {
        self.raw.vertex_attrib1fv_with_f32_array(index, v);
    }

    unsafe fn vertex_attrib_2_f32_slice(&self, index: u32, v: &[f32]) {
        self.raw.vertex_attrib2fv_with_f32_array(index, v);
    }

    unsafe fn vertex_attrib_3_f32_slice(&self, index: u32, v: &[f32]) {
        self.raw.vertex_attrib3fv_with_f32_array(index, v);
    }

    unsafe fn vertex_attrib_4_f32_slice(&self, index: u32, v: &[f32]) {
        self.raw.vertex_attrib4fv_with_f32_array(index, v);
    }

    unsafe fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.raw.viewport(x, y, width, height);
    }

    unsafe fn blend_func(&self, src: u32, dst: u32) {
        self.raw.blend_func(src as u32, dst as u32);
    }

    unsafe fn blend_func_separate(
        &self,
        src_rgb: u32,
        dst_rgb: u32,
        src_alpha: u32,
        dst_alpha: u32,
    ) {
        self.raw.blend_func_separate(
            src_rgb as u32,
            dst_rgb as u32,
            src_alpha as u32,
            dst_alpha as u32,
        );
    }

    unsafe fn blend_equation(&self, mode: u32) {
        self.raw.blend_equation(mode as u32);
    }

    unsafe fn blend_equation_separate(&self, mode_rgb: u32, mode_alpha: u32) {
        self.raw
            .blend_equation_separate(mode_rgb as u32, mode_alpha as u32)
    }

    unsafe fn stencil_func(&self, func: u32, reference: i32, mask: u32) {
        self.raw.stencil_func(func as u32, reference, mask);
    }

    unsafe fn stencil_func_separate(&self, face: u32, func: u32, reference: i32, mask: u32) {
        self.raw
            .stencil_func_separate(face as u32, func as u32, reference, mask)
    }

    unsafe fn stencil_mask(&self, mask: u32) {
        self.raw.stencil_mask(mask);
    }

    unsafe fn stencil_mask_separate(&self, face: u32, mask: u32) {
        self.raw.stencil_mask_separate(face as u32, mask);
    }

    unsafe fn stencil_op(&self, stencil_fail: u32, depth_fail: u32, pass: u32) {
        self.raw
            .stencil_op(stencil_fail as u32, depth_fail as u32, pass as u32)
    }

    unsafe fn stencil_op_separate(&self, face: u32, stencil_fail: u32, depth_fail: u32, pass: u32) {
        self.raw.stencil_op_separate(
            face as u32,
            stencil_fail as u32,
            depth_fail as u32,
            pass as u32,
        );
    }

    unsafe fn get_uniform_block_index(&self, program: Self::Program, name: &str) -> Option<u32> {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        let index = self.raw.get_uniform_block_index(raw_program, name);
        if index == INVALID_INDEX {
            None
        } else {
            Some(index)
        }
    }

    unsafe fn uniform_block_binding(&self, program: Self::Program, index: u32, binding: u32) {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        self.raw.uniform_block_binding(raw_program, index, binding);
    }

    unsafe fn read_buffer(&self, src: u32) {
        self.raw.read_buffer(src);
    }

    unsafe fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        gltype: u32,
        pixels: PixelPackData,
    ) {
        match pixels {
            PixelPackData::BufferOffset(offset) => self
                .raw
                .read_pixels_with_i32(x, y, width, height, format, gltype, offset as i32)
                .unwrap(),
            PixelPackData::Slice(bytes) => {
                let data = texture_data_view(gltype, bytes);
                self.raw
                    .read_pixels_with_opt_array_buffer_view(
                        x,
                        y,
                        width,
                        height,
                        format,
                        gltype,
                        Some(&data),
                    )
                    .unwrap();
            }
        }
    }

    unsafe fn texture_storage_3d(
        &self,
        _texture: Self::Texture,
        _levels: i32,
        _internal_format: u32,
        _width: i32,
        _height: i32,
        _depth: i32,
    ) {
        unimplemented!()
    }

    unsafe fn get_uniform_i32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [i32],
    ) {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        let value = self.raw.get_uniform(&raw_program, location);
        use wasm_bindgen::JsCast;
        if let Some(value) = value.as_f64() {
            v[0] = value as i32;
        } else if let Some(values) = value.dyn_ref::<js_sys::Int32Array>() {
            values.copy_to(v)
        }
    }

    unsafe fn get_uniform_f32(
        &self,
        program: Self::Program,
        location: &Self::UniformLocation,
        v: &mut [f32],
    ) {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        let value = self.raw.get_uniform(&raw_program, location);
        use wasm_bindgen::JsCast;
        if let Some(value) = value.as_f64() {
            v[0] = value as f32;
        } else if let Some(values) = value.dyn_ref::<js_sys::Float32Array>() {
            values.copy_to(v)
        }
    }

    unsafe fn begin_query(&self, target: u32, query: Self::Query) {
        let queries = self.queries.borrow();
        let raw_query = queries.get_unchecked(query);
        self.raw.begin_query(target, raw_query);
    }

    unsafe fn end_query(&self, target: u32) {
        self.raw.end_query(target);
    }

    unsafe fn get_query_parameter_u32(&self, query: Self::Query, parameter: u32) -> u32 {
        let queries = self.queries.borrow();
        let raw_query = queries.get_unchecked(query);
        self.raw
            .get_query_parameter(raw_query, parameter)
            .as_f64()
            .map(|v| v as u32)
            .unwrap_or(0)
    }

    unsafe fn create_transform_feedback(&self) -> Result<Self::TransformFeedback, String> {
        let raw_transform_feedback = self.raw.create_transform_feedback();

        match raw_transform_feedback {
            Some(t) => {
                let key = self.transform_feedbacks.borrow_mut().insert(t);
                Ok(key)
            }
            None => Err(String::from("Unable to create TransformFeedback object")),
        }
    }

    unsafe fn delete_transform_feedback(&self, transform_feedback: Self::TransformFeedback) {
        let mut transform_feedbacks = self.transform_feedbacks.borrow_mut();
        if let Some(ref t) = transform_feedbacks.remove(transform_feedback) {
            self.raw.delete_transform_feedback(Some(t));
        }
    }

    unsafe fn bind_transform_feedback(
        &self,
        target: u32,
        transform_feedback: Option<Self::TransformFeedback>,
    ) {
        let transform_feedbacks = self.transform_feedbacks.borrow();
        let raw_transform_feedback =
            transform_feedback.map(|tf| transform_feedbacks.get_unchecked(tf));

        self.raw
            .bind_transform_feedback(target, raw_transform_feedback)
    }

    unsafe fn begin_transform_feedback(&self, primitive_mode: u32) {
        self.raw.begin_transform_feedback(primitive_mode);
    }

    unsafe fn end_transform_feedback(&self) {
        self.raw.end_transform_feedback();
    }

    unsafe fn pause_transform_feedback(&self) {
        self.raw.pause_transform_feedback();
    }

    unsafe fn resume_transform_feedback(&self) {
        self.raw.resume_transform_feedback();
    }

    unsafe fn transform_feedback_varyings(
        &self,
        program: Self::Program,
        varyings: &[&str],
        buffer_mode: u32,
    ) {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        let js_varyings = Array::new();
        for &v in varyings {
            js_varyings.push(&v.into());
        }
        self.raw
            .transform_feedback_varyings(raw_program, &js_varyings, buffer_mode);
    }

    unsafe fn get_transform_feedback_varying(
        &self,
        program: Self::Program,
        index: u32,
    ) -> Option<ActiveTransformFeedback> {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);
        self.raw
            .get_transform_feedback_varying(raw_program, index)
            .map(|info| ActiveTransformFeedback {
                size: info.size(),
                tftype: info.type_(),
                name: info.name(),
            })
    }

    unsafe fn get_active_uniform_block_parameter_i32(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
    ) -> i32 {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        self.raw
            .get_active_uniform_block_parameter(raw_program, uniform_block_index, parameter)
            .unwrap()
            .as_f64()
            .map(|v| v as i32)
            // Errors will be caught by the browser or through `get_error`
            // so return a default instead
            .unwrap_or(0)
    }

    unsafe fn get_active_uniform_block_parameter_i32_slice(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
        parameter: u32,
        out: &mut [i32],
    ) {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        let value = self
            .raw
            .get_active_uniform_block_parameter(raw_program, uniform_block_index, parameter)
            .unwrap();

        use wasm_bindgen::JsCast;
        if let Some(value) = value.as_f64() {
            out[0] = value as i32;
        } else if let Some(values) = value.dyn_ref::<js_sys::Uint32Array>() {
            // To maintain compatibility with the pointers returned by
            // desktop GL, which are signed, an extra copy is needed
            // here.
            values
                .to_vec()
                .into_iter()
                .zip(out.iter_mut())
                .for_each(|(val, target)| *target = val as i32)
        }
    }
    unsafe fn get_active_uniform_block_name(
        &self,
        program: Self::Program,
        uniform_block_index: u32,
    ) -> String {
        let programs = self.programs.borrow();
        let raw_program = programs.get_unchecked(program);

        self.raw
            .get_active_uniform_block_name(raw_program, uniform_block_index)
            .unwrap()
    }

    unsafe fn max_shader_compiler_threads(&self, _count: u32) {
        // WebGL doesn't use this
    }
}

/// Sending texture data requires different data views for different data types.
/// This function reinterprets the byte data into the correct type for the texture.
/// The lookup is generated from this table: https://www.khronos.org/registry/webgl/specs/latest/2.0/#TEXTURE_PIXELS_TYPE_TABLE
unsafe fn texture_data_view(ty: u32, bytes: &[u8]) -> js_sys::Object {
    use std::mem::size_of;
    use std::slice::from_raw_parts;

    match ty {
        BYTE => {
            let data = from_raw_parts(bytes.as_ptr() as *const i8, bytes.len() / size_of::<i8>());
            js_sys::Int8Array::view(data).into()
        }

        SHORT => {
            #[allow(clippy::cast_ptr_alignment)]
            let data = from_raw_parts(bytes.as_ptr() as *const i16, bytes.len() / size_of::<i16>());
            js_sys::Int16Array::view(data).into()
        }

        UNSIGNED_SHORT
        | UNSIGNED_SHORT_5_6_5
        | UNSIGNED_SHORT_5_5_5_1
        | UNSIGNED_SHORT_4_4_4_4
        | HALF_FLOAT => {
            #[allow(clippy::cast_ptr_alignment)]
            let data = from_raw_parts(bytes.as_ptr() as *const u16, bytes.len() / size_of::<u16>());
            js_sys::Uint16Array::view(data).into()
        }

        INT => {
            #[allow(clippy::cast_ptr_alignment)]
            let data = from_raw_parts(bytes.as_ptr() as *const i32, bytes.len() / size_of::<i32>());
            js_sys::Int32Array::view(data).into()
        }

        UNSIGNED_INT
        | UNSIGNED_INT_5_9_9_9_REV
        | UNSIGNED_INT_2_10_10_10_REV
        | UNSIGNED_INT_10F_11F_11F_REV
        | UNSIGNED_INT_24_8 => {
            #[allow(clippy::cast_ptr_alignment)]
            let data = from_raw_parts(bytes.as_ptr() as *const u32, bytes.len() / size_of::<u32>());
            js_sys::Uint32Array::view(data).into()
        }

        FLOAT => {
            #[allow(clippy::cast_ptr_alignment)]
            let data = from_raw_parts(bytes.as_ptr() as *const f32, bytes.len() / size_of::<f32>());
            js_sys::Float32Array::view(data).into()
        }

        UNSIGNED_BYTE | _ => js_sys::Uint8Array::view(bytes).into(),
    }
}
