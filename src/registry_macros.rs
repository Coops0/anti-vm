#![allow(dead_code)]

use crate::flags::{Flags, Level};

pub struct RegistryRule {
    pub path: &'static str,
    pub checks: Vec<Check>,
}
pub enum Check {
    StringEquals {
        key: &'static str,
        value: &'static str,
        penalty: Level,
    },
    StringStartsWith {
        key: &'static str,
        value: &'static str,
        penalty: Level,
    },
    StringContains {
        key: &'static str,
        value: &'static str,
        penalty: Level,
    },
    StringEqualsAny {
        key: &'static str,
        values: Vec<&'static str>,
        penalty: Level,
    },
    StringContainsAny {
        key: &'static str,
        values: Vec<&'static str>,
        penalty: Level,
    },
    KeyNameContainsAny {
        values: Vec<&'static str>,
        penalty: Level,
    },
    ValueContainsAny {
        values: Vec<&'static str>,
        penalty: Level,
    },
    RecurseKeys {
        checks: Vec<Check>,
    },
    RecursePattern {
        pattern: &'static str,
        checks: Vec<Check>,
    },
}

#[macro_export]
macro_rules! eq {
    ($key:literal, $value:literal => $level:ident) => {
        $crate::registry_macros::Check::StringEquals {
            key: $key,
            value: $value,
            penalty: Level::$level,
        }
    };
}

#[macro_export]
macro_rules! starts_with {
    ($key:literal, $value:literal => $level:ident) => {
        $crate::registry_macros::Check::StringStartsWith {
            key: $key,
            value: $value,
            penalty: Level::$level,
        }
    };
}

#[macro_export]
macro_rules! contains {
    ($key:literal, $value:literal => $level:ident) => {
        $crate::registry_macros::Check::StringContains {
            key: $key,
            value: $value,
            penalty: Level::$level,
        }
    };
}

#[macro_export]
macro_rules! eq_any {
    ($key:literal, $($value:literal)|+ => $level:ident) => {
        $crate::registry_macros::Check::StringEqualsAny {
            key: $key,
            values: vec![$($value),+],
            penalty: Level::$level
        }
    };
}

#[macro_export]
macro_rules! contains_any {
    ($key:literal, $($value:literal)|+ => $level:ident) => {
        $crate::registry_macros::Check::StringContainsAny {
            key: $key,
            values: vec![$($value),+],
            penalty: Level::$level
        }
    };
}

#[macro_export]
macro_rules! key_contains {
    ($($value:literal)|+ => $level:ident) => {
        $crate::registry_macros::Check::KeyNameContainsAny {
            values: vec![$($value),+],
            penalty: Level::$level
        }
    };
}

#[macro_export]
macro_rules! any_value_contains {
    ($($value:literal)|+ => $level:ident) => {
        $crate::registry_macros::Check::ValueContainsAny {
            values: vec![$($value),+],
            penalty: Level::$level
        }
    };
}

#[macro_export]
macro_rules! recurse {
    ($($check:expr),* $(,)?) => {
        $crate::registry_macros::Check::RecurseKeys {
            checks: vec![$($check),*]
        }
    };
}

#[macro_export]
macro_rules! recurse_into {
    ($pattern:literal => { $($check:expr),* $(,)? }) => {
        $crate::registry_macros::Check::RecursePattern {
            pattern: $pattern,
            checks: vec![$($check),*]
        }
    };
}

#[macro_export]
macro_rules! rule {
    ($path:literal => { $($check:expr),* $(,)? }) => {
        $crate::registry_macros::RegistryRule {
            path: $path,
            checks: vec![$($check),*],
        }
    };
}

pub fn execute_checks(
    flags: &mut Flags,
    key: &windows_registry::Key,
    checks: &[Check],
)  {
    for check in checks {
        match check {
            Check::StringEquals {
                key: k,
                value,
                penalty,
            } => {
                if let Ok(v) = key.get_string(k)
                    && v.eq_ignore_ascii_case(value)
                {
                    flags.penalty(*penalty);
                }
            }
            Check::StringStartsWith {
                key: k,
                value,
                penalty,
            } => {
                if let Ok(v) = key.get_string(k)
                    && v.to_lowercase().starts_with(&value.to_lowercase())
                {
                    flags.penalty(*penalty);
                }
            }
            Check::StringContains {
                key: k,
                value,
                penalty,
            } => {
                if let Ok(v) = key.get_string(k)
                    && v.to_lowercase().contains(&value.to_lowercase())
                {
                    flags.penalty(*penalty);
                }
            }
            Check::StringEqualsAny {
                key: k,
                values,
                penalty,
            } => {
                if let Ok(v) = key.get_string(k) {
                    for value in values {
                        if v.eq_ignore_ascii_case(value) {
                            flags.penalty(*penalty);
                            break;
                        }
                    }
                }
            }
            Check::StringContainsAny {
                key: k,
                values,
                penalty,
            } => {
                if let Ok(v) = key.get_string(k) {
                    for value in values {
                        if v.to_lowercase().contains(&value.to_lowercase()) {
                            flags.penalty(*penalty);
                            break;
                        }
                    }
                }
            }
            Check::KeyNameContainsAny { values, penalty } => {
                if let Ok(keys) = key.keys() {
                    for key_name in keys {
                        for value in values {
                            if key_name.to_lowercase().contains(&value.to_lowercase()) {
                                flags.penalty(*penalty);
                            }
                        }
                    }
                }
            }
            Check::ValueContainsAny { values, penalty } => {
                if let Ok(registry_values) = key.values() {
                    for (_, reg_value) in registry_values {
                        if let Ok(string_value) = TryInto::<String>::try_into(reg_value) {
                            for value in values {
                                if string_value.to_lowercase().contains(&value.to_lowercase()) {
                                    flags.penalty(*penalty);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Check::RecurseKeys { checks: sub_checks } => {
                if let Ok(keys) = key.keys() {
                    for key_name in keys {
                        if let Ok(sub_key) = key.open(key_name) {
                            execute_checks(flags, &sub_key, sub_checks);
                        }
                    }
                }
            }
            Check::RecursePattern {
                pattern,
                checks: sub_checks,
            } => {
                if let Ok(sub_key) = key.open(pattern) {
                    execute_checks(flags, &sub_key, sub_checks);
                }
            }
        }
    }
}
