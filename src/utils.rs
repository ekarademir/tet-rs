#[cfg(target_arch = "wasm32")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
