//! Horizon QueryMemory ABI from switchbrew/libnx nx/{include/switch/kernel/svc.h,
//! source/kernel/svc.s}. Queries mapping/permissions before native pointer reads.
#[repr(C)]
#[derive(Default)]
struct MemoryInfo {
    address: u64,
    size: u64,
    kind: u32,
    attributes: u32,
    permission: u32,
    ipc_references: u32,
    device_references: u32,
    padding: u32,
}

pub(super) unsafe fn accessible(address: usize, size: usize, write: bool) -> bool {
    permitted(address, size, if write { 3 } else { 1 })
}

/// Readable executable mapping, required before chaining a saved code pointer.
pub(super) unsafe fn executable(address: usize, size: usize) -> bool {
    permitted(address, size, 5)
}

unsafe fn permitted(address: usize, size: usize, required: u32) -> bool {
    let Some(end) = address.checked_add(size) else {
        return false;
    };
    if address < 0x10000 || size == 0 {
        return false;
    }
    let mut info = MemoryInfo::default();
    let result: u64;
    // QueryMemory: x0 points at MemoryInfo, x2 is the queried address;
    // result is w0, page-info is w1. No memory/preserved-flags claims.
    core::arch::asm!(
        "svc 0x6",
        inout("x0") &mut info as *mut MemoryInfo as u64 => result,
        lateout("x1") _,
        in("x2") address as u64,
        clobber_abi("C"),
        options(nostack),
    );
    result as u32 == 0
        && info.permission & required == required
        && info.address <= address as u64
        && info
            .address
            .checked_add(info.size)
            .is_some_and(|mapped_end| end as u64 <= mapped_end)
}

/// `svcGetInfo` on the current process: w1 info type, x2 handle, x3 sub id;
/// result in w0, value in x1. Read-only; nothing is mapped or changed.
unsafe fn get_info(info_type: u32, sub_id: u64) -> Option<u64> {
    const CURRENT_PROCESS: u64 = 0xFFFF_8001;
    let result: u64;
    let value: u64;
    core::arch::asm!(
        "svc 0x29",
        lateout("x0") result,
        inout("x1") u64::from(info_type) => value,
        in("x2") CURRENT_PROCESS,
        in("x3") sub_id,
        clobber_abi("C"),
        options(nostack),
    );
    (result as u32 == 0).then_some(value)
}

/// Process memory as the kernel accounts it: total pool granted, bytes in use
/// (code, heap, stacks, every plugin, GPU-backed buffers) and the heap region
/// size. Three system calls; the diagnostics thread reads it every 500 ms.
pub(super) fn memory_snapshot() -> Option<crate::hang_trace::Memory> {
    // 6 TotalMemorySize, 7 UsedMemorySize, 5 HeapRegionSize.
    unsafe {
        Some(crate::hang_trace::Memory {
            total: get_info(6, 0)?,
            used: get_info(7, 0)?,
            heap: get_info(5, 0)?,
        })
    }
}

/// This process's program id (`svcGetInfo` type 18), which the game-aware
/// Horizon-OC fork checks against the application it is running (0.8.1).
pub(super) fn program_id() -> Option<u64> {
    unsafe { get_info(18, 0) }
}

/// Readable mapping that contains `address`, as `(base, size)`.
pub(super) unsafe fn region(address: usize) -> Option<(usize, usize)> {
    if address < 0x10000 {
        return None;
    }
    let mut info = MemoryInfo::default();
    let result: u64;
    core::arch::asm!(
        "svc 0x6",
        inout("x0") &mut info as *mut MemoryInfo as u64 => result,
        lateout("x1") _,
        in("x2") address as u64,
        clobber_abi("C"),
        options(nostack),
    );
    if result as u32 != 0 || info.size == 0 || info.permission & 1 == 0 {
        return None;
    }
    Some((info.address as usize, info.size as usize))
}
