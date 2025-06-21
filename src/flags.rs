#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub struct Flags {
    penalties: Vec<Level>,
    bonuses: Vec<Level>,
}

impl Flags {
    pub fn new() -> Self {
        Self {
            penalties: Vec::with_capacity(50),
            bonuses: Vec::with_capacity(50),
        }
    }

    pub fn penalty(&mut self, level: Level) {
        self.penalties.push(level);
    }

    pub fn bonus(&mut self, level: Level) {
        self.bonuses.push(level);
    }

    pub fn tiny_penalty(&mut self) {
        self.penalty(Level::Tiny);
    }
    pub fn small_penalty(&mut self) {
        self.penalty(Level::Small);
    }
    pub fn medium_penalty(&mut self) {
        self.penalty(Level::Medium);
    }
    pub fn large_penalty(&mut self) {
        self.penalty(Level::Large);
    }
    pub fn extreme_penalty(&mut self) {
        self.penalty(Level::Extreme);
    }
    pub fn end_all_penalty(&mut self) {
        self.penalty(Level::EndAll);
    }
    pub fn tiny_bonus(&mut self) {
        self.bonus(Level::Tiny);
    }
    pub fn small_bonus(&mut self) {
        self.bonus(Level::Small);
    }
    pub fn medium_bonus(&mut self) {
        self.bonus(Level::Medium);
    }
    pub fn large_bonus(&mut self) {
        self.bonus(Level::Large);
    }
    pub fn extreme_bonus(&mut self) {
        self.bonus(Level::Extreme);
    }
    pub fn end_all_bonus(&mut self) {
        self.bonus(Level::EndAll);
    }

    pub fn penalties(&self) -> &[Level] {
        &self.penalties
    }

    pub fn bonuses(&self) -> &[Level] {
        &self.bonuses
    }

    pub fn score(&self) -> i64 {
        let penalty_score: i64 = self.penalties.iter().copied().map(Level::value).sum();
        let bonus_score: i64 = self.bonuses.iter().copied().map(Level::value).sum();
        bonus_score - penalty_score
    }
}
