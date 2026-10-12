//! The one dynamic export other Skyline NROs resolve with `nn::ro::LookupSymbol`
//! to register Plugins-page rows. Skyline loads every plugin before any
//! `main`, so a consumer may call this from its own `main` regardless of
//! load order. Registration is accepted even when this NRO skipped its hooks;
//! `host_status` tells the consumer whether the menu can show the rows.
use super::{record, Code};
use deluxe_plugin_abi as abi;
use skyline::libc::c_char;
use std::ffi::CString;
use std::sync::OnceLock;

// The table shape is version 1 and stays so; descriptor versions 1 and 2 are
// both accepted by `register` (see `plugin_api::parse_descriptor`).
static TABLE: abi::PluginApiV1 = abi::PluginApiV1 {
    struct_size: core::mem::size_of::<abi::PluginApiV1>() as u32,
    abi_version: abi::TABLE_VERSION,
    register,
    host_status,
    host_build,
};

static BUILD: OnceLock<CString> = OnceLock::new();

/// Exported entry named by `deluxe_plugin_abi::API_SYMBOL`.
///
/// # Safety
/// Plain C entry; returns a pointer to a static table valid for the process.
#[no_mangle]
pub unsafe extern "C" fn ssbu_online_deluxe_plugin_api_v1() -> *const abi::PluginApiV1 {
    &TABLE
}

unsafe extern "C" fn register(desc: *const abi::PluginDesc) -> i32 {
    let result = crate::plugin_api::REGISTRY.register_raw(desc);
    let accepted = if result >= 0 {
        crate::plugin_api::REGISTRY
            .snapshot()
            .get(result as usize)
            .map(|p| p.options.len() as u64)
            .unwrap_or(0)
    } else {
        0
    };
    record(Code::PluginRegistered, result as i64 as u64, accepted);
    if result >= 0 {
        skyline::println!(
            "[Deluxe] plugin registered: handle {} with {} option(s)",
            result,
            accepted
        );
    } else {
        skyline::println!("[Deluxe] plugin registration refused: code {}", result);
    }
    result
}

/// Writes queued plugin INI edits. Called only from the background diagnostics
/// thread, never from a draw or game callback. Each write replaces one
/// `key=value` line in the plugin's own file through the same atomic
/// temp-rename-backup path Deluxe uses for `config.ini`.
pub(super) fn flush_persistence() {
    for write in crate::plugin_api::REGISTRY.take_writes() {
        let path = std::path::Path::new(&write.path);
        let result = (|| -> std::io::Result<()> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            crate::config_store::recover(path)?;
            let text = match std::fs::read_to_string(path) {
                Ok(text) => text,
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        || super::logging::file_is_missing(path) =>
                {
                    String::new()
                }
                Err(error) => return Err(error),
            };
            let updated = crate::plugin_api::apply_ini_assignment(&text, &write.key, &write.value);
            crate::config_store::save(path, &updated)
        })();
        record(
            Code::PluginPersisted,
            (write.plugin as u64 & 0xffff) | ((write.option as u64 & 0xffff) << 16),
            u64::from(result.is_ok()),
        );
        if let Err(error) = result {
            skyline::println!(
                "[Deluxe] plugin setting not persisted to {}: {}",
                write.path,
                error
            );
        }
    }
}

unsafe extern "C" fn host_status() -> u32 {
    if super::ui::installed() {
        abi::HOST_STATUS_MENU_ACTIVE
    } else {
        0
    }
}

unsafe extern "C" fn host_build() -> *const c_char {
    BUILD
        .get_or_init(|| {
            CString::new(crate::build_info::BUILD_ID).unwrap_or_else(|_| CString::default())
        })
        .as_ptr()
}
