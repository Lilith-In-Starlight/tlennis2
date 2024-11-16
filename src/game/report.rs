use crate::{player::PlayerId, team::TeamId, Data};
use std::fmt::Write;

use super::{Game, Space, Weather};

#[derive(Debug)]
pub struct Report {
    home: PlayerStateSnapshot,
    away: PlayerStateSnapshot,

    ball_direction: Option<Space>,

    pub comment: String,
    weather: Weather,
}

#[derive(Debug)]
pub struct PlayerStateSnapshot {
    team: TeamId,
    player: PlayerId,
    score: usize,
    space: Space,
}

impl Report {
    pub fn take_snapshot(game: &Game, data: &Data) -> Self {
        let home = PlayerStateSnapshot {
            team: game.home.team,
            player: game.home.team.get_current_player(data).unwrap(),
            score: game.home.score,
            space: game.home.space,
        };
        let away = PlayerStateSnapshot {
            team: game.away.team,
            player: game.away.team.get_current_player(data).unwrap(),
            score: game.away.score,
            space: game.away.space,
        };

        let ball_direction = Some(game.ball_direction);

        Self {
            home,
            away,
            ball_direction,
            comment: String::new(),
            weather: game.weather,
        }
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = comment;
        self
    }

    pub const fn with_no_ball(mut self) -> Self {
        self.ball_direction = None;
        self
    }

    pub fn get_text(&self, data: &Data) -> String {
        let mut output = String::new();
        writeln!(output, "---------------------[{}]", self.weather).unwrap();
        writeln!(
            output,
            "{}: {}",
            data.get_player(&self.home.player).unwrap().get_name(),
            self.home.score,
        )
        .unwrap();

        writeln!(
            output,
            "{}: {}",
            data.get_player(&self.away.player).unwrap().get_name(),
            self.away.score,
        )
        .unwrap();

        writeln!(output).unwrap();
        writeln!(output, "+++++++++++++++++++++++++").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "{}", self.comment).unwrap();
        writeln!(output, "-------------------------").unwrap();

        output
    }
}
