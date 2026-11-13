// Fixture for tests/hook_locks.rs: the locks.
static STATE: Mutex<u32> = Mutex::new(0);

pub fn bump() {
    *STATE.lock().unwrap() += 1;
}

pub fn try_peek() {
    let _ = STATE.try_lock();
}

pub fn thread_only() {
    let _guard = STATE.lock();
}

pub fn install_only() {
    let _guard = STATE.lock();
}

/// A function whose name a hook also uses for a tuple-pattern local: that local's cast is not a call.
pub fn level() -> u32 {
    *OTHER.lock().unwrap()
}

static OTHER: Mutex<u32> = Mutex::new(0);

pub struct Ring {
    rows: Mutex<Vec<u32>>,
}

impl Ring {
    pub fn new() -> Self {
        Ring { rows: Mutex::new(Vec::new()) }
    }

    pub fn push_row(&self, value: u32) {
        self.rows.lock().unwrap().push(value);
    }
}

#[cfg(test)]
mod tests {
    fn in_test() {
        let _guard = super::STATE.lock();
    }
}
