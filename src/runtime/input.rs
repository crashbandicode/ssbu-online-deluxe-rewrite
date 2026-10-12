use skyline::nn::hid;

#[derive(Clone, Copy, Default)]
pub(super) struct Sample {
    pub buttons: u64,
    pub left_y: i32,
    pub update_count: i64,
    pub npad_id: u8,
    pub style_flags: u32,
}

/// Same HID families as the gamepad overlay; no game input is modified.
pub(super) unsafe fn read() -> Sample {
    for id in [0u32, 0x20] {
        let styles = hid::GetNpadStyleSet(&id).flags;
        if styles == 0 {
            continue;
        }
        if styles & (1 << 5) != 0 {
            let mut state = hid::NpadGcState::default();
            hid::GetNpadGcState(&mut state, &id);
            return Sample {
                buttons: state.Buttons,
                left_y: state.LStickY,
                update_count: state.updateCount,
                npad_id: id as u8,
                style_flags: styles,
            };
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
        return Sample {
            buttons: state.Buttons,
            left_y: state.LStickY,
            update_count: state.updateCount,
            npad_id: id as u8,
            style_flags: styles,
        };
    }
    Sample::default()
}
