use crate::Team;
use std::fmt::Display;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::Data;

use super::{Game, Run, Side};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weather {
    None,
    Feedback,
    Reverb,
    Observation,
    Omni,
    Unpredictable,
}

impl Display for Weather {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Clear"),
            Self::Feedback => write!(f, "Feedback"),
            Self::Reverb => write!(f, "Reverb"),
            Self::Observation => write!(f, "Observation"),
            Self::Unpredictable => write!(f, "???"),
            Self::Omni => write!(f, "All"),
        }
    }
}

impl Distribution<Weather> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Weather {
        match rng.gen_range::<usize, _>(0..=4) {
            0 => Weather::None,
            1 => Weather::Feedback,
            2 => Weather::Reverb,
            3 => Weather::Observation,
            4 => Weather::Unpredictable,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum WeatherResult {
    Prevent,
    Nothing,
}

impl Weather {
    fn weather_announce(self, hitter: Side, game: &mut Game, data: &Data) {
        match self {
            Self::None => game.report("It's a sunny day!".to_owned(), data),
            Self::Feedback => {
                game.report("The feedback gathers around the players.".to_owned(), data);
            }
            Self::Reverb => game.report("The ground tremors with reverb.".to_owned(), data),
            Self::Observation => game.report("The clouds reveal eyes in the sky.".to_owned(), data),
            Self::Omni => game.report("We're experiencing everything.".to_owned(), data),
            Self::Unpredictable => {
                game.report("We don't know what the sky is doing.".to_owned(), data);
            }
        }
    }
    pub(super) fn pre_hit<R: Rng>(
        self,
        hitter: Side,
        game: &mut Game,
        data: &mut Data,
        rng: &mut R,
    ) -> WeatherResult {
        let (hitter_state, _) = match hitter {
            Side::Home => (&mut game.home, &mut game.away),
            Side::Away => (&mut game.away, &mut game.home),
        };
        match self {
            Self::None => WeatherResult::Nothing,
            Self::Feedback => {
                if rng.gen::<f64>() < 0.05 {
                    let away_id = game.away.team.get_current_player(data).unwrap();
                    let home_id = game.home.team.get_current_player(data).unwrap();
                    *game.home.team.get_current_player_mut(data).unwrap() = away_id;
                    *game.away.team.get_current_player_mut(data).unwrap() = home_id;
                    game.report(
                        format!(
                            "{} has been feedbacked with {}!",
                            data.get_player(&home_id).unwrap().get_name(),
                            data.get_player(&away_id).unwrap().get_name()
                        ),
                        data,
                    );
                }
                WeatherResult::Nothing
            }
            Self::Reverb => {
                if rng.gen::<f64>() < 0.05 {
                    data.get_team_mut(&game.home.team)
                        .unwrap()
                        .shuffle_players(rng);
                    data.get_team_mut(&game.away.team)
                        .unwrap()
                        .shuffle_players(rng);

                    game.report("The teams are caught in the reverb!!".to_string(), data);
                }

                WeatherResult::Nothing
            }
            Self::Observation => {
                if rng.gen::<f64>() < 0.05 {
                    let old_player = data
                        .get_team(&hitter_state.team)
                        .and_then(Team::get_current_player)
                        .unwrap();

                    let new_player = data.new_player(rng);
                    *data
                        .get_team_mut(&hitter_state.team)
                        .and_then(|x| x.get_current_player_mut())
                        .unwrap() = new_player;

                    game.report(
                        format!(
                            "The observers have defragged {}.",
                            data.get_player(&old_player).unwrap().get_name(),
                        ),
                        data,
                    );

                    game.report(
                        format!(
                            "{} has been created in their place! They don't know what's going on!",
                            data.get_player(&new_player).unwrap().get_name(),
                        ),
                        data,
                    );

                    WeatherResult::Prevent
                } else if rng.gen::<f64>() < 0.1 {
                    let hitter = data
                        .get_team(&hitter_state.team)
                        .and_then(Team::get_current_player)
                        .unwrap();

                    game.report(
                        format!(
                            "The overseers watch {} with intent.",
                            data.get_player(&hitter).unwrap().get_name(),
                        ),
                        data,
                    );
                    WeatherResult::Nothing
                } else {
                    WeatherResult::Nothing
                }
            }
            Self::Omni => rng.gen::<Self>().pre_hit(hitter, game, data, rng),
            Self::Unpredictable => {
                if rng.gen::<f64>() < 0.05 {
                    game.weather = rng.gen();
                    game.weather.weather_announce(hitter, game, data);
                    game.weather.pre_hit(hitter, game, data, rng)
                } else {
                    WeatherResult::Nothing
                }
            }
        }
    }
}
