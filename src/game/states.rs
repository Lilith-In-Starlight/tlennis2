use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::team::TeamId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Space {
    First,
    Middle,
    Third,
}

impl Space {
    pub fn farthest<R: Rng>(self, rng: &mut R) -> Self {
        match self {
            Self::First => Self::Third,
            Self::Middle => loop {
                let x: Self = rng.gen();
                if x != self {
                    break x;
                }
            },
            Self::Third => Self::First,
        }
    }
}

impl Distribution<Space> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Space {
        match rng.gen_range::<usize, _>(0..=2) {
            0 => Space::First,
            1 => Space::Middle,
            2 => Space::Third,
            _ => unreachable!(),
        }
    }
}

pub struct PlayerState {
    pub(super) team: TeamId,
    pub(super) space: Space,
    pub(super) score: usize,
}
