const SAVE_KEY: &str = "evergreen.save";

pub(crate) fn read() -> Option<String> {
    read_storage(SAVE_KEY)
}

pub(crate) fn write(value: &str) -> Result<(), String> {
    write_storage(SAVE_KEY, value)
}

// --- WASM -------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
fn write_storage(key: &str, value: &str) -> Result<(), String> {
    use wasm_bindgen::JsValue;
    use web_sys::window;

    let storage = window()
        .ok_or("no window")?
        .local_storage()
        .map_err(|e| format!("{e:?}"))?
        .ok_or("no localStorage")?;

    storage
        .set_item(key, value)
        .map_err(|e: JsValue| format!("{e:?}"))
}

#[cfg(target_arch = "wasm32")]
fn read_storage(key: &str) -> Option<String> {
    use web_sys::window;

    window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

// --- Native (dev / test) ----------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn write_storage(_key: &str, value: &str) -> Result<(), String> {
    use std::fs;
    let path = native_path();
    fs::create_dir_all(path.parent().expect("path has parent"))
        .map_err(|e| e.to_string())?;
    fs::write(&path, value).map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_storage(_key: &str) -> Option<String> {
    std::fs::read_to_string(native_path()).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn native_path() -> std::path::PathBuf {
    std::path::PathBuf::from("./evergreen_saves/evergreen.save.json")
}
