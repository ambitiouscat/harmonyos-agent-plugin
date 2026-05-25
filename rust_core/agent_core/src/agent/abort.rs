use std::sync::atomic::AtomicBool;

pub static ABORT_FLAG: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_abort_and_reset() {
        assert!(!ABORT_FLAG.load(Ordering::Relaxed));
        ABORT_FLAG.store(true, Ordering::Relaxed);
        assert!(ABORT_FLAG.load(Ordering::Relaxed));
        ABORT_FLAG.store(false, Ordering::Relaxed);
        assert!(!ABORT_FLAG.load(Ordering::Relaxed));
    }
}
