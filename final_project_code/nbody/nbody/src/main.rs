mod app;
mod pass;
mod pipelines;
mod simulation;
#[cfg(target_arch = "wasm32")]
mod wasm;

use std::panic;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::app::App;

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! console_log {
    ($($t:tt)*) => (println!($($t)*))
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    console_log!("Starting simulation.");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_maximized(true)
        .build(&event_loop)
        .expect("failed to create window!");

    let app = pollster::block_on(App::new(window));

    start(event_loop, app);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::prelude::*;

    panic::set_hook(Box::new(console_error_panic_hook::hook));

    wasm_bindgen_futures::spawn_local(async move {
        console_log!("Starting simulation.");

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .build(&event_loop)
            .expect("failed to create window!");

        wasm::insert_canvas(&window);

        let app = App::new(window).await;
        let start_closure = Closure::once_into_js(move || start(event_loop, app));

        // make sure to handle JS exceptions thrown inside start.
        // Otherwise wasm_bindgen_futures Queue would break and never handle any tasks again.
        // This is required, because winit uses JS exception for control flow to escape from `run`.
        if let Err(error) = call_catch(&start_closure) {
            let is_control_flow_exception = error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                e.message().includes("Using exceptions for control flow", 0)
            });

            if !is_control_flow_exception {
                web_sys::console::error_1(&error);
            }
        }

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
            fn call_catch(this: &JsValue) -> Result<(), JsValue>;
        }
    });
}

fn start(event_loop: EventLoop<()>, mut app: App) {
    let start_time = chrono::Utc::now();
    event_loop.run(move |event, _, control_flow| {
        app.egui_rp.platform.handle_event(&event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.render_ctx.window.id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::W),
                            ..
                        },
                    ..
                } => {
                    app.render_ctx.zoom += 0.1;
                },
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::S),
                            ..
                        },
                    ..
                } => {
                    app.render_ctx.zoom -= 0.1;
                }
                WindowEvent::Resized(size) => app.resize(*size),
                // WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                //     //resize
                // }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == app.render_ctx.window.id() => {
                app.egui_rp.platform.update_time(
                    (start_time - chrono::Utc::now())
                        .num_microseconds()
                        .unwrap() as f64
                        * 1000000.0,
                );
                match app.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => todo!("resize"),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                app.render_ctx.window.request_redraw();
            }
            _ => {}
        }
    });
}
