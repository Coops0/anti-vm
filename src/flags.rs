#![allow(dead_code)]

use std::mem;

use paste2::paste;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Tiny,
    Small,
    Medium,
    Large,
    Extreme,
    EndAll,
}

impl Level {
    const fn value(self) -> i64 {
        match self {
            Self::Tiny => 1,
            Self::Small => 3,
            Self::Medium => 10,
            Self::Large => 20,
            Self::Extreme => 50,
            Self::EndAll => 5000,
        }
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct Flags {
    penalties: Vec<Level>,
    bonuses: Vec<Level>,
}

#[cfg(debug_assertions)]
fn print_caller(t: &str, level: Level, location: &std::panic::Location<'_>) {
    use crate::debug_println;

    debug_println!(
        "{t}: {level:?} at {}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    );
}

impl Flags {
    pub fn new() -> Self {
        Self {
            penalties: Vec::with_capacity(50),
            bonuses: Vec::with_capacity(50),
        }
    }

    #[cfg(debug_assertions)]
    #[track_caller]
    #[inline]
    pub fn penalty(&mut self, level: Level) {
        print_caller("PENALTY GENERIC", level, std::panic::Location::caller());
        self.inner_penalty(level);
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn penalty(&mut self, level: Level) {
        self.inner_penalty(level);
    }

    #[cfg(debug_assertions)]
    #[track_caller]
    #[inline]
    pub fn bonus(&mut self, level: Level) {
        self.inner_bonus(level);
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn bonus(&mut self, level: Level) {
        self.inner_bonus(level);
    }

    #[inline]
    fn inner_penalty(&mut self, level: Level) {
        self.penalties.push(level);
    }

    #[inline]
    fn inner_bonus(&mut self, level: Level) {
        self.bonuses.push(level);
    }

    #[inline]
    pub fn penalties(&self) -> &[Level] {
        &self.penalties
    }

    #[inline]
    pub fn bonuses(&self) -> &[Level] {
        &self.bonuses
    }

    pub fn score(&self) -> i64 {
        let penalty_score: i64 = self.penalties.iter().copied().map(Level::value).sum();
        let bonus_score: i64 = self.bonuses.iter().copied().map(Level::value).sum();
        bonus_score - penalty_score
    }

    pub fn merge(&mut self, other: &mut Self) {
        self.penalties.extend(mem::take(&mut other.penalties));
        self.bonuses.extend(mem::take(&mut other.bonuses));
    }
}

macro_rules! impl_flags {
    ($($name:ident => $level:expr),* $(,)?) => {
        paste! {
            impl Flags {
                $(
                    #[cfg(debug_assertions)]
                    #[track_caller]
                    #[inline]
                    pub fn [<$name _penalty>](&mut self) {
                        print_caller("PENALTY", $level, std::panic::Location::caller());
                        self.inner_penalty($level);
                    }

                    #[cfg(not(debug_assertions))]
                    #[inline]
                    pub fn [<$name _penalty>](&mut self) {
                        self.inner_penalty($level);
                    }

                    #[cfg(debug_assertions)]
                    #[track_caller]
                    #[inline]
                    pub fn [<$name _bonus>](&mut self) {
                        print_caller("BONUS", $level, std::panic::Location::caller());
                        self.inner_bonus($level);
                    }

                    #[cfg(not(debug_assertions))]
                    #[inline]
                    pub fn [<$name _bonus>](&mut self) {
                        self.inner_bonus($level);
                    }
                )*
            }
        }
    };
}

impl_flags! {
    tiny => Level::Tiny,
    small => Level::Small,
    medium => Level::Medium,
    large => Level::Large,
    extreme => Level::Extreme,
    end_all => Level::EndAll,
}
