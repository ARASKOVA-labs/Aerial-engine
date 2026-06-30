/// aerial-engine/src/utils.rs
/// Panic hook setup for better wasm error messages in the browser console.

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
