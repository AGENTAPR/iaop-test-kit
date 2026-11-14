//! Environment variable guards for tests.
//!
//! `EnvGuard` sets a variable on creation and restores the previous
//! value on drop — safe for parallel tests as long as each test uses
//! distinct variable names.

// Edition 2024 marks std::env::set_var/remove_var as unsafe; usage here is
// deliberate and documented (process-global env mutation in tests only).
#[allow(unsafe_code)]
mod imp {
    use super::EnvGuard;

    impl EnvGuard {
        /// Sets `key` to `value`, remembering the previous value.
        pub fn set(key: &str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            // SAFETY: test-only helper; callers must not mutate the same
            // key from parallel tests.
            unsafe { std::env::set_var(key, value) };
            Self {
                key: key.to_string(),
                previous,
            }
        }

        /// Removes `key`, remembering the previous value.
        pub fn remove(key: &str) -> Self {
            let previous = std::env::var(key).ok();
            // SAFETY: see `set`.
            unsafe { std::env::remove_var(key) };
            Self {
                key: key.to_string(),
                previous,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                // SAFETY: see `set`.
                Some(v) => unsafe { std::env::set_var(&self.key, v) },
                None => unsafe { std::env::remove_var(&self.key) },
            }
        }
    }
}

/// RAII guard that restores an env var on drop.
pub struct EnvGuard {
    key: String,
    previous: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_restores_previous_value() {
        let key = "IAOP_TEST_KIT_GUARD_CHECK";
        let _g = EnvGuard::set(key, "changed");
        assert_eq!(std::env::var(key).unwrap(), "changed");
        drop(_g);
        assert!(std::env::var(key).is_err());
    }
}
