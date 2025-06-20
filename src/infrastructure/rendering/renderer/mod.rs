//! WebGPU renderer responsible for drawing the chart.
//!
//! This module manages GPU buffers and performs the render loop. The renderer
//! is kept behind a global handle to simplify access from the UI layer.

use crate::domain::market_data::Candle;
use crate::domain::{
    chart::Chart,
    logging::{LogComponent, get_logger},
};
use crate::infrastructure::rendering::gpu_structures::{
    CandleInstance, CandleVertex, ChartUniforms,
};
use gloo::utils::document;
use js_sys;
use leptos::SignalSet;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;
thread_local! {
    static GLOBAL_RENDERER: RefCell<Option<Rc<RefCell<WebGpuRenderer>>>> = const { RefCell::new(None) };
}

/// Number of samples for MSAA
pub const MSAA_SAMPLE_COUNT: u32 = 4;

/// Store the global renderer instance
pub fn set_global_renderer(renderer: Rc<RefCell<WebGpuRenderer>>) {
    GLOBAL_RENDERER.with(|cell| {
        *cell.borrow_mut() = Some(renderer);
    });
    GLOBAL_RENDERER.with(|cell| {
        if let Some(rc) = &*cell.borrow() {
            crate::app::global_line_visibility().set(rc.borrow().line_visibility.clone());
        }
    });
}

/// Obtain a mutable reference to the global renderer
pub fn with_global_renderer<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut WebGpuRenderer) -> R,
{
    GLOBAL_RENDERER.with(|cell| {
        let opt = cell.borrow();
        if let Some(rc) = opt.as_ref() {
            rc.try_borrow_mut().ok().map(|mut r| f(&mut r))
        } else {
            None
        }
    })
}

/// Actual WebGPU renderer for candles
pub struct WebGpuRenderer {
    _canvas_id: String,
    width: u32,
    height: u32,

    // WGPU state
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    // Rendering pipeline
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    msaa_texture: wgpu::Texture,
    msaa_view: wgpu::TextureView,
    template_vertices: u32,

    // 🗄️ Cached data
    cached_vertices: Vec<CandleVertex>,
    cached_uniforms: ChartUniforms,
    cached_candle_count: usize,
    cached_zoom_level: f64,
    cached_hash: u64,
    cached_data_hash: u64,
    cached_line_visibility: LineVisibility,

    // 🔍 Zoom and pan parameters
    zoom_level: f64,
    pan_offset: f64,

    // ⏱️ Performance metrics
    last_frame_time: f64,
    fps_log: VecDeque<f64>,

    // 📊 Indicator line visibility
    line_visibility: LineVisibility,
}

/// State of indicator line visibility
#[derive(Debug, Clone, PartialEq)]
pub struct LineVisibility {
    pub sma_20: bool,
    pub sma_50: bool,
    pub sma_200: bool,
    pub ema_12: bool,
    pub ema_26: bool,
}

impl Default for LineVisibility {
    fn default() -> Self {
        Self { sma_20: true, sma_50: true, sma_200: true, ema_12: true, ema_26: true }
    }
}

mod geometry;
pub use geometry::{
    EDGE_GAP, MAX_ELEMENT_WIDTH, MIN_ELEMENT_WIDTH, SPACING_RATIO, candle_x_position,
    spacing_ratio_for,
};
mod initialization;
mod performance;
mod render_loop;
mod render_queue;

pub use render_queue::{enqueue_render_task, init_render_queue};

#[allow(invalid_value)]
pub fn dummy_renderer() -> WebGpuRenderer {
    use std::collections::VecDeque;
    unsafe {
        WebGpuRenderer {
            _canvas_id: String::new(),
            width: 800,
            height: 600,
            surface: std::mem::MaybeUninit::zeroed().assume_init(),
            device: std::mem::MaybeUninit::zeroed().assume_init(),
            queue: std::mem::MaybeUninit::zeroed().assume_init(),
            config: std::mem::MaybeUninit::zeroed().assume_init(),
            render_pipeline: std::mem::MaybeUninit::zeroed().assume_init(),
            vertex_buffer: std::mem::MaybeUninit::zeroed().assume_init(),
            uniform_buffer: std::mem::MaybeUninit::zeroed().assume_init(),
            uniform_bind_group: std::mem::MaybeUninit::zeroed().assume_init(),
            msaa_texture: std::mem::MaybeUninit::zeroed().assume_init(),
            msaa_view: std::mem::MaybeUninit::zeroed().assume_init(),
            template_vertices: 0,
            cached_vertices: Vec::new(),
            cached_uniforms: ChartUniforms::new(),
            cached_candle_count: 0,
            cached_zoom_level: 1.0,
            cached_hash: 0,
            cached_data_hash: 0,
            cached_line_visibility: LineVisibility::default(),
            zoom_level: 1.0,
            pan_offset: 0.0,
            last_frame_time: 0.0,
            fps_log: VecDeque::new(),
            line_visibility: LineVisibility::default(),
        }
    }
}
