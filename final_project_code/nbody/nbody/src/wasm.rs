use std::num::NonZeroU32;

use winit::{dpi::LogicalSize, window::Window};

// Intellisense probably doesn't work here (it's fine, trust)
pub fn insert_canvas(window: &Window) {
    use softbuffer::{Surface, SurfaceExtWeb};
    use winit::platform::web::WindowExtWebSys;

    let canvas = window.canvas();

    // let mut surface = Surface::from_canvas(canvas.clone()).unwrap();
    // surface
    //     .resize(
    //         NonZeroU32::new(canvas.width()).unwrap(),
    //         NonZeroU32::new(canvas.height()).unwrap(),
    //     )
    //     .unwrap();
    // let mut buffer = surface.buffer_mut().unwrap();
    // buffer.fill(0xFFF0000);
    // buffer.present().unwrap();

    let wswin = web_sys::window().unwrap();
    let document = wswin.document().unwrap();
    let body = document.body().unwrap();

    window.set_inner_size(LogicalSize::new(
        wswin.inner_width().unwrap().as_f64().unwrap() as f32,
        wswin.inner_height().unwrap().as_f64().unwrap() as f32,
    ));

    // let style = &canvas.style();

    body.append_child(&canvas);
}
