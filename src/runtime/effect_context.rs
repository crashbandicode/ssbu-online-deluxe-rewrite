//! Read-only actor context. No owner traversal or weapon work-ID reads.
use crate::effect_context::{self, Context};
use std::ffi::c_void;

extern "C" {
    #[link_name = "\u{1}_ZN3app9sv_system22battle_object_categoryEP9lua_State"]
    fn lua_category(lua_state: u64) -> u8;
    #[link_name = "\u{1}_ZN3app9sv_system18battle_object_kindEP9lua_State"]
    fn lua_kind(lua_state: u64) -> i32;
    #[link_name = "\u{1}_ZN3app9sv_system29battle_object_module_accessorEP9lua_State"]
    fn lua_accessor(lua_state: u64) -> *mut c_void;
    #[link_name = "\u{1}_ZN3app7utility8get_kindEPKNS_26BattleObjectModuleAccessorE"]
    fn object_kind(accessor: *mut c_void) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind30MotionModule__motion_kind_implEPNS_26BattleObjectModuleAccessorE"]
    fn motion_kind(accessor: *mut c_void) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind30StatusModule__status_kind_implEPNS_26BattleObjectModuleAccessorE"]
    fn status_kind(accessor: *mut c_void) -> i32;
}

unsafe fn capture(accessor: *mut c_void, category: u8, kind: i32) -> Option<Context> {
    if accessor.is_null() || !effect_context::tracked(category, kind) {
        return None;
    }
    Some(Context {
        category,
        kind,
        motion: motion_kind(accessor),
        status: status_kind(accessor),
    })
}

pub(super) unsafe fn from_lua(lua_state: u64) -> Option<Context> {
    if lua_state == 0 {
        return None;
    }
    let category = lua_category(lua_state);
    if category > 1 {
        return None;
    }
    capture(lua_accessor(lua_state), category, lua_kind(lua_state))
}

pub(super) unsafe fn from_accessor(accessor: *mut c_void) -> Option<Context> {
    if accessor.is_null() {
        return None;
    }
    // Pinned BattleObjectModuleAccessor: vtable at 0, object ID at 8.
    // utility::get_category in the same binding uses the top ID nibble.
    let id = *accessor.cast::<u8>().add(8).cast::<u32>();
    let category = (id >> 28) as u8;
    if category > 1 {
        return None;
    }
    capture(accessor, category, object_kind(accessor))
}
