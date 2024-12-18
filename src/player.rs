use rand::Rng;
use uuid::Uuid;

use crate::NameGenerator;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PlayerId {
    uuid: Uuid,
}

pub struct Player {
    name: String,
    control: f64,
    speed: f64,
    distractability: f64,
}

impl Player {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub(super) fn new<R: Rng>(name_generator: &NameGenerator, rng: &mut R) -> (PlayerId, Self) {
        let new_id = PlayerId {
            uuid: Uuid::new_v4(),
        };

        let new_player = Self {
            name: name_generator.generate(rng),
            speed: rng.gen(),
            control: rng.gen(),
            distractability: rng.gen::<f64>().powi(2),
        };

        (new_id, new_player)
    }

    /// Successful if player is not distracted
    pub fn distraction_check<R: Rng>(&self, rng: &mut R) -> bool {
        rng.gen::<f64>() < self.distractability
    }

    /// Successful if player is fast enough
    pub fn speed_check<R: Rng>(&self, rng: &mut R) -> bool {
        rng.gen::<f64>() > self.speed
    }
    /// Successful if player has control
    pub fn control_check<R: Rng>(&self, rng: &mut R) -> bool {
        rng.gen::<f64>() > self.control
    }
}
