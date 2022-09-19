use once_cell::unsync::OnceCell;
use wasm_bindgen_test::wasm_bindgen_test;

thread_local! {
     static LOGGER: OnceCell<()> = OnceCell::new();
}

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn browser_desktop_base() {
    LOGGER.with(|cell| {
        cell.get_or_init(|| wasm_logger::init(wasm_logger::Config::default()));
    });

    log::info!("eh! oh! let's go!");
}
