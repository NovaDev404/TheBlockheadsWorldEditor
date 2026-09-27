#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};
#[cfg(target_arch = "wasm32")]
use the_blockheads_tools_gui::app::EditorApp;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

// Global storage for world data passed from JavaScript
#[cfg(target_arch = "wasm32")]
static WORLD_DATA: Mutex<Option<Vec<u8>>> = Mutex::new(None);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas: HtmlCanvasElement, data: Option<Vec<u8>>) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    eframe::WebLogger::init(log::LevelFilter::Trace).ok();

    // Store world data globally if provided
    if let Some(world_data) = data {
        *WORLD_DATA.lock().unwrap() = Some(world_data);
    }

    // Request WebGPU's 128 MiB baseline for each byte-packed voxel segment.
    const VOXEL_SEGMENT_BYTES: u64 = 128 * 1024 * 1024;

    // Try WebGPU with high limits first
    let webgpu_options = {
        let mut options = eframe::WebOptions::default();
        options.wgpu_options.wgpu_setup =
            eframe::egui_wgpu::WgpuSetup::CreateNew(eframe::egui_wgpu::WgpuSetupCreateNew {
                device_descriptor: Arc::new(|_| eframe::wgpu::DeviceDescriptor {
                    label: Some("egui wgpu device"),
                    required_limits: eframe::wgpu::Limits {
                        max_texture_dimension_2d: 8192,
                        max_buffer_size: VOXEL_SEGMENT_BYTES,
                        max_storage_buffer_binding_size: VOXEL_SEGMENT_BYTES as u32,
                        ..eframe::wgpu::Limits::default()
                    },
                    ..Default::default()
                }),
                ..Default::default()
            });
        options
    };

    let webgpu_result = eframe::WebRunner::new()
        .start(
            canvas.clone(),
            webgpu_options,
            Box::new(|cc| {
                // If we have world data, use with_initial_data
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(world_data) = WORLD_DATA.lock().unwrap().take() {
                        return Ok(Box::new(EditorApp::with_initial_data(cc, world_data)));
                    }
                }

                Ok(Box::new(EditorApp::new(cc)))
            }),
        )
        .await;

    if webgpu_result.is_ok() {
        return;
    }

    // WebGPU failed, try with default setup (will use WebGL fallback)
    log::warn!("WebGPU with high limits failed, trying default setup with WebGL fallback");

    let fallback_options = eframe::WebOptions::default();

    eframe::WebRunner::new()
        .start(
            canvas,
            fallback_options,
            Box::new(|cc| {
                // If we have world data, use with_initial_data
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(world_data) = WORLD_DATA.lock().unwrap().take() {
                        return Ok(Box::new(EditorApp::with_initial_data(cc, world_data)));
                    }
                }

                Ok(Box::new(EditorApp::new(cc)))
            }),
        )
        .await
        .expect("Failed to start eframe with fallback options")
}
