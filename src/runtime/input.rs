use skyline::nn::hid;
pub(super) const L: u64 = 1 << 6;
pub(super) const R: u64 = 1 << 7;
pub(super) const ZL: u64 = 1 << 8;
pub(super) const ZR: u64 = 1 << 9;
pub(super) const LEFT: u64 = 1 << 12;
pub(super) const UP: u64 = 1 << 13;
pub(super) const RIGHT: u64 = 1 << 14;
pub(super) const DOWN: u64 = 1 << 15;

/// Same HID families as the gamepad overlay; no game input is modified.
pub(super) unsafe fn buttons() -> u64 {
    for id in [0u32, 0x20] {
        let styles = hid::GetNpadStyleSet(&id).flags;
        if styles == 0 {
            continue;
        }
        if styles & (1 << 5) != 0 {
            let mut state = hid::NpadGcState::default();
            hid::GetNpadGcState(&mut state, &id);
            return state.Buttons;
        }
        let mut state = hid::NpadHandheldState::default();
        if styles & (1 << 1) != 0 {
            hid::GetNpadHandheldState(&mut state, &id);
        } else if styles & (1 << 2) != 0 {
            hid::GetNpadJoyDualState(&mut state, &id);
        } else if styles & (1 << 3) != 0 {
            hid::GetNpadJoyLeftState(&mut state, &id);
        } else if styles & (1 << 4) != 0 {
            hid::GetNpadJoyRightState(&mut state, &id);
        } else {
            hid::GetNpadFullKeyState(&mut state, &id);
        }
        return state.Buttons;
    }
    0
}
