// DDD Architecture modules
pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod presentation;

// Re-exports
pub use presentation::*;

// WASM and WebGPU imports
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, window};
use std::rc::Rc;
use std::cell::RefCell;

// DDD imports - правильный поток через Application Layer
use crate::domain::market_data::{Symbol, TimeInterval};
use crate::domain::chart::{Chart, ChartType};
use crate::infrastructure::websocket::BinanceWebSocketClient;
use crate::application::{ChartApplicationCoordinator, ChartRenderData};

// Legacy types для обратной совместимости с WebGPU (временно)
#[derive(Debug, Clone)]
pub struct CandleData {
    pub timestamp: f64,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
}

impl From<crate::domain::market_data::Candle> for CandleData {
    fn from(candle: crate::domain::market_data::Candle) -> Self {
        Self {
            timestamp: candle.timestamp.as_f64(),
            open: candle.ohlcv.open.value(),
            high: candle.ohlcv.high.value(),
            low: candle.ohlcv.low.value(),
            close: candle.ohlcv.close.value(),
            volume: candle.ohlcv.volume.value(),
        }
    }
}

/// Состояние приложения - теперь использует Application Layer
struct ApplicationState {
    coordinator: ChartApplicationCoordinator<BinanceWebSocketClient>,
    chart: Rc<RefCell<Chart>>,
    canvas_width: u32,
    canvas_height: u32,
    needs_resize: bool,
}

impl ApplicationState {
    fn new(width: u32, height: u32) -> Self {
        let chart = Rc::new(RefCell::new(Chart::new(
            "main".to_string(), 
            ChartType::Candlestick, 
            1000
        )));
        
        let ws_client = BinanceWebSocketClient::new();
        let coordinator = ChartApplicationCoordinator::new(ws_client, chart.clone());
        
        Self {
            coordinator,
            chart,
            canvas_width: width,
            canvas_height: height,
            needs_resize: false,
        }
    }
    
    fn start_live_data(&mut self, symbol: &str, interval: &str) -> Result<(), JsValue> {
        let symbol = Symbol::from(symbol);
        let interval = match interval {
            "1m" => TimeInterval::OneMinute,
            "5m" => TimeInterval::FiveMinutes,
            "15m" => TimeInterval::FifteenMinutes,
            "1h" => TimeInterval::OneHour,
            "1d" => TimeInterval::OneDay,
            _ => return Err(JsValue::from_str("Unsupported interval")),
        };
        
        // Используем Application Layer правильно
        self.coordinator.start_live_chart(symbol, interval)
    }
    
    fn get_render_data(&self) -> ChartRenderData {
        let chart = self.chart.borrow();
        self.coordinator.prepare_chart_render(&chart)
    }
    
    fn check_resize(&mut self, canvas: &HtmlCanvasElement) -> bool {
        let new_width = canvas.width();
        let new_height = canvas.height();
        
        if new_width != self.canvas_width || new_height != self.canvas_height {
            self.canvas_width = new_width;
            self.canvas_height = new_height;
            
            // Обновляем viewport через chart
            {
                let mut chart = self.chart.borrow_mut();
                chart.viewport.width = new_width;
                chart.viewport.height = new_height;
            }
            
            self.needs_resize = true;
            true
        } else {
            false
        }
    }
    
    fn get_candle_count(&self) -> usize {
        self.chart.borrow().data.count()
    }
    
    fn get_latest_price(&self) -> Option<f32> {
        let chart = self.chart.borrow();
        let candles = chart.data.get_candles();
        candles.last().map(|candle| candle.ohlcv.close.value())
    }
}

struct RenderState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    app_state: Rc<RefCell<ApplicationState>>,
}

impl RenderState {
    fn render(&mut self) -> Result<(), JsValue> {
        let frame = self.surface.get_current_texture()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        // Получаем данные для рендеринга через Application Layer
        let render_data = self.app_state.borrow().get_render_data();
        
        // Логируем статистику (временно)
        if render_data.candle_count > 0 {
            if let Some(latest_price) = self.app_state.borrow().get_latest_price() {
                #[allow(unused_unsafe)]
                unsafe {
                    web_sys::console::log_1(&format!(
                        "Render: {} candles, latest price: ${:.2}",
                        render_data.candle_count,
                        latest_price
                    ).into());
                }
            }
        }

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas = document
        .get_element_by_id("chart-canvas")
        .ok_or("no canvas")?
        .dyn_into::<HtmlCanvasElement>()?;

    let instance = wgpu::Instance::default();
    
    // Create surface using unsafe method for WebGL/WebGPU
    let value: &wasm_bindgen::JsValue = &canvas;
    let obj = core::ptr::NonNull::from(value).cast();
    let raw_window_handle = raw_window_handle::WebCanvasWindowHandle::new(obj).into();
    let raw_display_handle = raw_window_handle::WebDisplayHandle::new().into();
    
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle,
            raw_window_handle,
        })
    }.map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to get adapter: {:?}", e)))?;

    let (device, queue) = adapter
        .request_device(&Default::default())
        .await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

    let size = (canvas.width(), canvas.height());
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats[0];
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.0,
        height: size.1,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        cache: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // Создаем состояние приложения с правильной DDD архитектурой
    let app_state = Rc::new(RefCell::new(ApplicationState::new(size.0, size.1)));
    
    #[allow(unused_unsafe)] 
    unsafe { web_sys::console::log_1(&"🏗️ DDD Application initialized".into()); }
    
    // Запускаем live данные через Application Layer
    match app_state.borrow_mut().start_live_data("btcusdt", "1m") {
        Ok(_) => {
            #[allow(unused_unsafe)] 
            unsafe { web_sys::console::log_1(&"📊 Live chart started via Application Layer".into()); }
        }
        Err(e) => {
            #[allow(unused_unsafe)]
            unsafe { web_sys::console::error_1(&format!("❌ Failed to start live chart: {:?}", e).into()); }
        }
    }
    
    let render_state = Rc::new(RefCell::new(RenderState {
        surface,
        device,
        queue,
        render_pipeline,
        app_state,
    }));

    // Start the render loop
    start_render_loop(render_state)?;

    Ok(())
}

fn start_render_loop(render_state: Rc<RefCell<RenderState>>) -> Result<(), JsValue> {
    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Render frame
        if let Err(e) = render_state.borrow_mut().render() {
                #[allow(unused_unsafe)]
                unsafe {
                    web_sys::console::error_1(&format!("Render error: {:?}", e).into());
                }
        }

        // Schedule next frame
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
