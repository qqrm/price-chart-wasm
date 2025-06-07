use std::cell::RefCell;
use wasm_bindgen::JsValue;
use crate::domain::{
    chart::Chart,
    logging::{LogComponent, get_logger},
};
use crate::infrastructure::rendering::WebGpuRenderer;

/// Глобальный координатор приложения для управления рендерингом и состоянием
pub struct ChartCoordinator {
    webgpu_renderer: Option<WebGpuRenderer>,
    chart: Option<Chart>,
    canvas_id: String,
    width: u32,
    height: u32,
    is_initialized: bool,
}

impl ChartCoordinator {
    pub fn new(canvas_id: String, width: u32, height: u32) -> Self {
        get_logger().info(
            LogComponent::Application("ChartCoordinator"),
            "Creating new chart coordinator"
        );

        Self {
            webgpu_renderer: None,
            chart: None,
            canvas_id,
            width,
            height,
            is_initialized: false,
        }
    }

    /// Асинхронная инициализация с WebGPU рендерером (согласно ARCHITECTURE.md)
    pub fn initialize_renderer(&mut self, renderer: WebGpuRenderer) {
        get_logger().info(
            LogComponent::Application("ChartCoordinator"),
            "🚀 Initializing WebGPU renderer..."
        );
        self.webgpu_renderer = Some(renderer);
        self.is_initialized = true;
        get_logger().info(
            LogComponent::Application("ChartCoordinator"),
            "✅ WebGPU renderer initialized successfully"
        );
    }

    /// Рендеринг графика через WebGPU (согласно ARCHITECTURE.md)
    pub fn render_chart(&self) -> Result<(), JsValue> {
        if !self.is_initialized {
            return Err(JsValue::from_str("Chart coordinator not initialized"));
        }

        if let (Some(chart), Some(renderer)) = (&self.chart, &self.webgpu_renderer) {
            renderer.render(chart)
        } else {
            Err(JsValue::from_str("Chart or renderer not available"))
        }
    }

    /// Установить данные графика
    pub fn set_chart(&mut self, chart: Chart) {
        get_logger().info(
            LogComponent::Application("ChartCoordinator"),
            &format!("Chart data updated: {} candles", chart.get_candle_count())
        );
        self.chart = Some(chart);
    }

    /// Получить ссылку на график
    pub fn get_chart(&self) -> Option<&Chart> {
        self.chart.as_ref()
    }

    /// Получить мутабельную ссылку на график
    pub fn get_chart_mut(&mut self) -> Option<&mut Chart> {
        self.chart.as_mut()
    }

    /// Обновить размеры canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        if let Some(renderer) = &mut self.webgpu_renderer {
            renderer.resize(width, height);
        }

        get_logger().info(
            LogComponent::Application("ChartCoordinator"),
            &format!("Canvas resized to {}x{}", width, height)
        );
    }

    /// Проверить статус инициализации
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Получить информацию о производительности
    pub fn get_performance_info(&self) -> String {
        if let Some(renderer) = &self.webgpu_renderer {
            renderer.get_performance_info()
        } else {
            "{\"backend\":\"none\",\"status\":\"not_initialized\"}".to_string()
        }
    }
}

// Глобальный экземпляр координатора (thread-local для WASM)
thread_local! {
    pub static GLOBAL_COORDINATOR: RefCell<Option<ChartCoordinator>> = RefCell::new(None);
}

/// Инициализация глобального координатора
pub fn initialize_global_coordinator(canvas_id: String, width: u32, height: u32) {
    GLOBAL_COORDINATOR.with(|global| {
        let coordinator = ChartCoordinator::new(canvas_id, width, height);
        *global.borrow_mut() = Some(coordinator);
    });
}

/// Получение ссылки на глобальный координатор для чтения
pub fn with_global_coordinator<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ChartCoordinator) -> R,
{
    GLOBAL_COORDINATOR.with(|global| {
        global.borrow().as_ref().map(f)
    })
}

/// Получение мутабельной ссылки на глобальный координатор
pub fn with_global_coordinator_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut ChartCoordinator) -> R,
{
    GLOBAL_COORDINATOR.with(|global| {
        global.borrow_mut().as_mut().map(f)
    })
} 