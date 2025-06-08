// === 🦀 LEPTOS BITCOIN CHART WASM ===
// Clean Architecture v3.0 - только нужные модули!

pub mod domain;
pub mod infrastructure; 
pub mod app;

// === WASM EXPORTS ===
use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Log that WASM started
    unsafe { web_sys::console::log_1(&"🚀 WASM module initialized!".into()); }
    
    // Initialize infrastructure services
    crate::infrastructure::initialize_infrastructure_services();
    
    // Mount Leptos app to body
    unsafe { web_sys::console::log_1(&"🎯 Mounting Leptos app...".into()); }
    
    // Hide the loading screen first
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(loading_div) = document.get_element_by_id("loading") {
                let _ = loading_div.set_attribute("style", "display: none;");
            }
        }
    }
    
    leptos::mount_to_body(|| view! { <crate::app::App/> });
    
    unsafe { web_sys::console::log_1(&"✅ Leptos app mounted!".into()); }
}

/// Проверка WebGPU поддержки
#[wasm_bindgen]
pub async fn is_webgpu_supported() -> bool {
    crate::infrastructure::WebGpuRenderer::is_webgpu_supported().await
}

/// Получить производительность рендерера
#[wasm_bindgen]
pub fn get_renderer_performance() -> String {
    // Заглушка - возвращаем статическую информацию
    "{\"backend\":\"WebGPU\",\"status\":\"ready\",\"fps\":60}".to_string()
}

// Clean WASM exports only 