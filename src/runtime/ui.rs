//! Native CSS callback: never hooks Layout::Draw.
use super::{hooks, input, publish, record, settings, Code};
use crate::menu::{Action, Menu};
use skyline::nn::ui2d::Pane;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

struct UiState {
    menu: Menu,
    previous_buttons: u64,
    last_owner: u64,
}
static UI: Mutex<UiState> = Mutex::new(UiState {
    menu: Menu {
        open: false,
        selected: 0,
    },
    previous_buttons: 0,
    last_owner: 0,
});
static CSS_CALLS: AtomicU64 = AtomicU64::new(0);
static LAST_PANE: AtomicU64 = AtomicU64::new(0);

#[skyline::from_offset(hooks::SET_TEXT)]
fn set_text(pane: *mut Pane, text: *const u8);

unsafe fn banner(owner: u64) -> Option<*mut Pane> {
    let address = crate::ui_safety::banner_address(owner as usize, |address| {
        super::memory::read_pointer(address)
    })?;
    if !super::memory::accessible(
        address,
        std::mem::size_of::<skyline::nn::ui2d::TextBox>(),
        true,
    ) {
        return None;
    }
    let pane = address as *mut Pane;
    let name = &(*pane).name;
    let end = name.iter().position(|v| *v == 0)?;
    let mut bytes = [0u8; 25];
    for (out, value) in bytes.iter_mut().zip(name) {
        *out = *value as u8;
    }
    if !crate::ui_safety::is_title_name(&bytes[..end]) {
        return None;
    }
    let textbox = &*(pane as *const skyline::nn::ui2d::TextBox);
    if textbox.text_buf.is_null()
        || textbox.text_buf_len < 2
        || textbox.text_buf_len > 4096
        || textbox.text_len > textbox.text_buf_len
        || !textbox.font_size_x.is_finite()
        || !textbox.font_size_y.is_finite()
    {
        return None;
    }
    Some(pane)
}

#[skyline::hook(offset = hooks::CSS_UPDATE)]
unsafe fn update_css(owner: u64) {
    let count = CSS_CALLS.fetch_add(1, Ordering::Relaxed);
    let sampled = count % 60 == 0;
    if sampled {
        record(Code::CssEntered, owner, count);
    }
    update_banner(owner);
    call_original!(owner);
    if sampled {
        record(Code::CssExited, owner, count);
    }
}

unsafe fn update_banner(owner: u64) {
    let Some(pane) = banner(owner) else {
        if CSS_CALLS.load(Ordering::Relaxed) % 60 == 1 {
            record(Code::PaneRejected, owner, 0);
        }
        return; // No invisible settings changes when this CSS is unsupported.
    };
    if LAST_PANE.swap(pane as u64, Ordering::Relaxed) != pane as u64 {
        let name = &(*pane).name;
        let mut value = 0u64;
        for byte in name {
            value = value.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        record(Code::PaneAccepted, pane as u64, value);
    }
    let Ok(mut ui) = UI.try_lock() else {
        return;
    };
    let buttons = input::buttons();
    if ui.last_owner != owner {
        ui.menu.open = false;
        ui.last_owner = owner;
        ui.previous_buttons = buttons;
    }
    let pressed = buttons & !ui.previous_buttons;
    let chord = (buttons & (input::L | input::R)) == (input::L | input::R)
        || (buttons & (input::ZL | input::ZR)) == (input::ZL | input::ZR);
    ui.previous_buttons = buttons;
    let action = if chord && pressed & input::DOWN != 0 {
        Some(Action::Toggle)
    } else if ui.menu.open && chord {
        if pressed & input::UP != 0 {
            Some(Action::Up)
        } else if pressed & input::RIGHT != 0 {
            Some(Action::Right)
        } else if pressed & input::LEFT != 0 {
            Some(Action::Left)
        } else {
            None
        }
    } else {
        None
    };
    let mut config = settings();
    if let Some(action) = action {
        if ui.menu.handle(action, &mut config) {
            publish(&config);
            record(
                Code::MenuChanged,
                config.effects as u64,
                config.input_buffer as i64 as u64,
            );
        }
    }
    {
        let mut text = ui.menu.summary(&config);
        text.push('\0');
        // These are existing TextBox panes (same chain as the original Deluxe).
        // Change typography only; avoid raw material/dirty-bit manipulation.
        let textbox = &mut *(pane as *mut skyline::nn::ui2d::TextBox);
        textbox.font_size_x = 20.0;
        textbox.font_size_y = 32.0;
        textbox.line_space = 0.0;
        set_text(pane, text.as_ptr());
    }
}

pub(super) fn install() {
    unsafe {
        if hooks::preflight(hooks::CSS_UPDATE) && hooks::preflight(hooks::SET_TEXT) {
            skyline::install_hook!(update_css);
            record(Code::HookInstalled, hooks::CSS_UPDATE as u64, 0);
        }
    }
}
