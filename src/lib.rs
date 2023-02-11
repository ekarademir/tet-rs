mod tetrs;
mod utils;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const WINDOW_WIDTH: u32 = 600;
const WINDOW_HEIGHT: u32 = 600;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    wasm_bindgen_futures::spawn_local(execute());
}

async fn execute() {
    #[cfg(target_arch = "wasm32")]
    use winit::platform::web::WindowExtWebSys;
    let event_loop =
        winit::event_loop::EventLoopBuilder::<tetrs::GameEvent>::with_user_event().build();
    let window = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_title("Tetrs")
        .build(&event_loop)
        .expect("Couldn't initialise the window");

    #[cfg(target_arch = "wasm32")]
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");

    let tetrs = tetrs::Tetrs::new(&window, &event_loop)
        .await
        .expect("Can't create tetrs");

    tetrs::run(window, event_loop, tetrs)
        .await
        .expect("Couldn't run tetrs");
}
