use std::mem::MaybeUninit;
use std::sync::{Mutex, MutexGuard};

use ssbu_online_deluxe_rewrite::acmd::{replacements, Value};
use ssbu_online_deluxe_rewrite::config::FxMode;
use ssbu_online_deluxe_rewrite::eff_variant::{
    self, VariantAvailability, PATH_HASH, REDUCED, SOURCE,
};
use ssbu_online_deluxe_rewrite::effect_context::Context;
use ssbu_online_deluxe_rewrite::effects::hash40;
use ssbu_online_deluxe_rewrite::menu;

fn value(raw: u64) -> Value {
    Value {
        kind: 2,
        metadata: 0,
        raw,
        indirect_return_padding: MaybeUninit::uninit(),
    }
}

fn up_air() -> Context {
    Context {
        category: 0,
        kind: 0x36,
        motion: hash40("attack_air_hi"),
        status: 0x36,
    }
}

fn other_air() -> Context {
    Context {
        motion: hash40("attack_air_f"),
        ..up_air()
    }
}

static GATES: Mutex<()> = Mutex::new(());

struct IsolatedGates<'a> {
    _guard: MutexGuard<'a, ()>,
}

impl IsolatedGates<'_> {
    fn enter() -> Self {
        let guard = GATES
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        eff_variant::reset_for_test();
        Self { _guard: guard }
    }
}

impl Drop for IsolatedGates<'_> {
    fn drop(&mut self) {
        eff_variant::reset_for_test();
    }
}

#[test]
fn up_air_maps_only_after_the_callback_succeeds() {
    let _gates = IsolatedGates::enter();
    assert_eq!(PATH_HASH, 0x0027_070a_c2fb);
    assert_eq!(SOURCE, hash40("palutena_backlight2"));
    assert_eq!(REDUCED, hash40("palutena_backlight2_reduced"));
    let args = [value(SOURCE)];

    assert_eq!(
        replacements(FxMode::Reduced, &args, 1, Some(up_air())),
        [None, None]
    );
    assert_eq!(
        eff_variant::forward(FxMode::On, true, SOURCE, Some(up_air())),
        SOURCE
    );
    assert_ne!(
        replacements(FxMode::Off, &args, 1, Some(up_air()))[0],
        Some(REDUCED)
    );

    eff_variant::set_availability_for_test(VariantAvailability::Ready);
    assert_eq!(
        replacements(FxMode::Reduced, &args, 1, Some(up_air()))[0],
        Some(REDUCED)
    );
    assert_eq!(
        replacements(FxMode::Reduced, &args, 1, Some(other_air()))[0],
        None
    );
    assert_eq!(
        replacements(FxMode::On, &args, 1, Some(up_air())),
        [None, None]
    );
    assert_eq!(
        eff_variant::forward(FxMode::Reduced, false, SOURCE, Some(up_air())),
        SOURCE
    );
    assert_ne!(
        replacements(FxMode::Off, &args, 1, Some(up_air()))[0],
        Some(REDUCED)
    );

    eff_variant::set_availability_for_test(VariantAvailability::Unavailable);
    assert_eq!(
        replacements(FxMode::Reduced, &args, 1, Some(up_air())),
        [None, None]
    );
    assert_eq!(
        menu::fx_row_value(FxMode::Reduced, false, VariantAvailability::Unavailable),
        "Reduced"
    );
    assert_eq!(
        menu::fx_row_value(FxMode::Reduced, false, VariantAvailability::Pending),
        "Reduced"
    );
    assert_eq!(
        menu::fx_row_value(FxMode::Reduced, false, VariantAvailability::Ready),
        "Reduced"
    );
    assert_eq!(
        menu::fx_file_debug(
            FxMode::Reduced,
            false,
            VariantAvailability::Unavailable,
            "sibling ready"
        ),
        "original (callback unavailable)"
    );
    assert_eq!(
        menu::fx_file_debug(
            FxMode::Reduced,
            false,
            VariantAvailability::Pending,
            "sibling ready"
        ),
        "original (callback pending)"
    );
    assert_eq!(
        menu::fx_file_debug(
            FxMode::Reduced,
            false,
            VariantAvailability::Ready,
            "sibling ready"
        ),
        "sibling ready"
    );
    assert_eq!(
        menu::fx_file_debug(
            FxMode::Reduced,
            false,
            VariantAvailability::Ready,
            "life 3 to 2 candidate"
        ),
        "life 3 to 2 candidate"
    );
    assert_eq!(
        menu::fx_file_debug(
            FxMode::Off,
            false,
            VariantAvailability::Ready,
            "sibling ready"
        ),
        "not selected"
    );
    assert_eq!(
        menu::fx_row_value(FxMode::Reduced, true, VariantAvailability::Unavailable),
        "Reduced (tracker unavailable)"
    );
    assert_eq!(
        menu::fx_row_value(FxMode::On, false, VariantAvailability::Unavailable),
        "On"
    );
    assert_eq!(
        menu::fx_row_value(FxMode::Off, false, VariantAvailability::Ready),
        "Off"
    );
    eff_variant::reset_for_test();
}

#[test]
fn shadow_flare_maps_only_from_its_own_ready_gate() {
    let _gates = IsolatedGates::enter();
    let bullet = hash40("edge_shadowflare_bullet");
    let reduced = hash40("edge_shadowflare_bullet_reduced");
    let aura = hash40("edge_shadowflare_aura");
    let start = hash40("edge_shadowflare_start");
    let hold = hash40("edge_shadowflare_hold");
    let side = Context {
        category: 0,
        kind: 0x59,
        motion: hash40("special_s"),
        status: 0x1dd,
    };
    let flare = Context {
        category: 1,
        kind: 0x248,
        motion: 0,
        status: 0,
    };
    let args = [value(bullet)];
    eff_variant::set_availability_for_test(VariantAvailability::Unavailable);
    eff_variant::set_edge_availability_for_test(VariantAvailability::Ready);
    assert_eq!(
        replacements(FxMode::Reduced, &args, 1, Some(side))[0],
        Some(reduced)
    );
    assert_eq!(
        replacements(FxMode::Reduced, &[value(bullet)], 1, Some(flare))[0],
        Some(reduced)
    );
    assert_eq!(
        replacements(FxMode::Reduced, &[value(aura)], 1, Some(side))[0],
        None
    );
    assert_eq!(
        replacements(FxMode::Reduced, &[value(start)], 1, Some(side))[0],
        Some(hash40("edge_shadowflare_start_reduced"))
    );
    assert_eq!(
        replacements(FxMode::Reduced, &[value(hold)], 1, Some(side))[0],
        Some(hash40("edge_shadowflare_hold_reduced"))
    );
    assert_eq!(
        replacements(FxMode::On, &[value(start)], 1, Some(side))[0],
        None
    );
    assert_ne!(
        replacements(FxMode::Off, &[value(start)], 1, Some(side))[0],
        Some(hash40("edge_shadowflare_start_reduced"))
    );
    let smash = Context {
        category: 0,
        kind: 0x59,
        motion: hash40("attack_s4"),
        status: 0x2f,
    };
    assert_eq!(
        replacements(FxMode::Reduced, &[value(start)], 1, Some(smash))[0],
        None
    );
    assert_eq!(
        eff_variant::cleanup_alias(start),
        Some(hash40("edge_shadowflare_start_reduced"))
    );
    assert_eq!(
        replacements(FxMode::Reduced, &[value(SOURCE)], 1, Some(up_air()))[0],
        None
    );
    assert_ne!(
        replacements(FxMode::Off, &args, 1, Some(side))[0],
        Some(reduced)
    );
    eff_variant::set_edge_availability_for_test(VariantAvailability::Unavailable);
    eff_variant::set_availability_for_test(VariantAvailability::Ready);
    assert_eq!(replacements(FxMode::Reduced, &args, 1, Some(side))[0], None);
    assert_eq!(
        replacements(FxMode::Reduced, &[value(start)], 1, Some(side))[0],
        None
    );
    assert_eq!(
        eff_variant::cleanup_alias(start),
        Some(hash40("edge_shadowflare_start_reduced"))
    );
    assert_eq!(
        eff_variant::cleanup_alias(hash40("edge_shadowflare_start_reduced")),
        None
    );
    assert_eq!(
        replacements(FxMode::Reduced, &[value(SOURCE)], 1, Some(up_air()))[0],
        Some(REDUCED)
    );
    eff_variant::reset_for_test();
}

#[test]
fn roller_splash_uses_its_own_gate() {
    let _gates = IsolatedGates::enter();
    let splash = hash40("inkling_roller_splash_run");
    let reduced = hash40("inkling_roller_splash_run_reduced");
    eff_variant::set_edge_availability_for_test(VariantAvailability::Ready);
    eff_variant::set_availability_for_test(VariantAvailability::Ready);
    assert_eq!(
        replacements(FxMode::Reduced, &[value(splash)], 1, None)[0],
        None
    );
    eff_variant::set_ink_availability_for_test(VariantAvailability::Ready);
    assert_eq!(
        replacements(FxMode::Reduced, &[value(splash)], 1, None)[0],
        Some(reduced)
    );
    assert_eq!(replacements(FxMode::On, &[value(splash)], 1, None)[0], None);
    assert_eq!(
        replacements(FxMode::Off, &[value(splash)], 1, None)[0],
        None
    );
    eff_variant::set_ink_availability_for_test(VariantAvailability::Unavailable);
    assert_eq!(
        replacements(FxMode::Reduced, &[value(SOURCE)], 1, Some(up_air()))[0],
        Some(REDUCED)
    );
    eff_variant::reset_for_test();
}

#[test]
fn chrom_life_gate_does_not_clear_edge_or_become_off() {
    let _gates = IsolatedGates::enter();
    let chrom = hash40("chrom_mc_1");
    let reduced = hash40("chrom_mc_1_reduced");
    let bullet = hash40("edge_shadowflare_bullet");
    let bullet_reduced = hash40("edge_shadowflare_bullet_reduced");
    let side = Context {
        category: 0,
        kind: 0x59,
        motion: hash40("special_s"),
        status: 0x1dd,
    };
    eff_variant::set_edge_availability_for_test(VariantAvailability::Ready);
    eff_variant::set_chrom_availability_for_test(VariantAvailability::Unavailable);
    assert_eq!(
        replacements(FxMode::Reduced, &[value(chrom)], 1, None)[0],
        None
    );
    assert_eq!(
        replacements(FxMode::Reduced, &[value(bullet)], 1, Some(side))[0],
        Some(bullet_reduced)
    );
    assert_eq!(eff_variant::edge_availability(), VariantAvailability::Ready);
    eff_variant::set_chrom_availability_for_test(VariantAvailability::Ready);
    assert_eq!(
        replacements(FxMode::Reduced, &[value(chrom)], 1, None)[0],
        Some(reduced)
    );
    assert_ne!(
        replacements(FxMode::Off, &[value(chrom)], 1, None)[0],
        Some(reduced)
    );
    assert_eq!(replacements(FxMode::On, &[value(chrom)], 1, None)[0], None);
    eff_variant::reset_for_test();
}
