//! Keys the shell takes before the interface maps them.
//!
//! A `NativeActivity`'s window takes the input queue for its native side, so a
//! key the platform has a use for — a volume key turning the system's own
//! stream — never reaches the Java view hierarchy, and this crate's input loop
//! is the only place it can be seen. GPUI has no use for those keys and would
//! otherwise swallow them. A shell installs a filter here and answers for the
//! keys it means to take; a key it takes is not passed to the interface at all.
//!
//! One filter, not a set: a key belongs to the process, and a shell that
//! replaced the filter meant to.

use std::sync::Mutex;

/// Takes a key before GPUI sees it, and says whether it was the shell's.
///
/// `key_code` is an Android key code and `action` is `0` for a press or `1` for
/// its release, the values [`crate::android::AndroidKeyEvent`] carries.
pub type KeyFilter = fn(key_code: i32, action: i32) -> bool;

static FILTER: Mutex<Option<KeyFilter>> = Mutex::new(None);

/// Hand every key to `filter` before it is offered to the interface.
///
/// A filter replaces the one before it; a shell that takes the device is the
/// only one there is.
pub fn set_key_filter(filter: KeyFilter) {
    if let Ok(mut installed) = FILTER.lock() {
        *installed = Some(filter);
    }
}

/// Offer one key to the shell, if it asked for keys.
pub(crate) fn take(key_code: i32, action: i32) -> bool {
    let filter = match FILTER.lock() {
        Ok(installed) => *installed,
        Err(poisoned) => *poisoned.into_inner(),
    };
    filter.is_some_and(|filter| filter(key_code, action))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    static SEEN: AtomicI32 = AtomicI32::new(i32::MIN);

    fn volume(key_code: i32, _: i32) -> bool {
        SEEN.store(key_code, Ordering::Relaxed);
        key_code == 24
    }

    /// A filter is offered every key and answers for the ones it knows; the
    /// rest are still the interface's.
    #[test]
    fn a_filter_takes_only_the_keys_it_answers_for() {
        assert!(!take(24, 0), "nothing is taken before a shell asks");
        set_key_filter(volume);
        assert!(take(24, 0));
        assert_eq!(SEEN.load(Ordering::Relaxed), 24);
        assert!(!take(29, 0), "a letter is still the interface's");
    }
}
