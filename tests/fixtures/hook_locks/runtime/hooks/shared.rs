// Fixture for tests/hook_locks.rs: a OnceLock initialised on the hook path.
static CELL: OnceLock<u32> = OnceLock::new();

pub fn fetch() -> u32 {
    *CELL.get_or_init(|| 3)
}
