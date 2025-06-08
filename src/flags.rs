#! [allow(dead_code)]

#[derive(Debug)]
pub enum Level {
    Tiny,
    Small,
    Medium,
    Large,
    Extreme,
    EndAll
}

impl Level {
    fn value(&self) -> i32 {
        match self {
            Level::Tiny => 1,
            Level::Small => 3,
            Level::Medium => 10,
            Level::Large => 20,
            Level::Extreme => 50,
            Level::EndAll => 5000,
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

    pub fn tiny_penalty(&mut self) {
        self.penalties.push(Level::Tiny);
    }
    pub fn small_penalty(&mut self) {
        self.penalties.push(Level::Small);
    }
    pub fn medium_penalty(&mut self) {
        self.penalties.push(Level::Medium);
    }
    pub fn large_penalty(&mut self) {
        self.penalties.push(Level::Large);
    }
    pub fn extreme_penalty(&mut self) {
        self.penalties.push(Level::Extreme);
    }
    pub fn end_all_penalty(&mut self) {
        self.penalties.push(Level::EndAll);
    }
    pub fn tiny_bonus(&mut self) {
        self.bonuses.push(Level::Tiny);
    }
    pub fn small_bonus(&mut self) {
        self.bonuses.push(Level::Small);
    }
    pub fn medium_bonus(&mut self) {
        self.bonuses.push(Level::Medium);
    }
    pub fn large_bonus(&mut self) {
        self.bonuses.push(Level::Large);
    }
    pub fn extreme_bonus(&mut self) {
        self.bonuses.push(Level::Extreme);
    }
    pub fn end_all_bonus(&mut self) {
        self.bonuses.push(Level::EndAll);
    }

    pub fn penalties(&self) -> &[Level] {
        &self.penalties
    }

    pub fn bonuses(&self) -> &[Level] {
        &self.bonuses
    }

    pub fn score(&self) -> i32 {
        let penalty_score: i32 = self.penalties.iter().map(Level::value).sum();
        let bonus_score: i32 = self.bonuses.iter().map(Level::value).sum();
        bonus_score - penalty_score
    }
}
