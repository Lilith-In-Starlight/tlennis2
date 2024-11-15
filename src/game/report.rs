use crate::{player::PlayerId, team::TeamId, Data};
use std::fmt::Write;

use super::{Game, GameSpace, HomeOrAway};

#[derive(Debug)]
pub struct Report {
    home: TeamId,
    away: TeamId,

    home_score: usize,
    away_score: usize,

    current_away_player: PlayerId,
    current_home_player: PlayerId,

    current_away_space: GameSpace,
    current_home_space: GameSpace,

    ball_direction: Option<GameSpace>,

    comment: String,
}

impl Report {
    pub fn take_snapshot(game: &Game, data: &Data) -> Self {
        let home = game.get_team(HomeOrAway::Home);
        let away = game.get_team(HomeOrAway::Away);

        let current_away_player = away.get_current_player(data).unwrap();
        let current_home_player = home.get_current_player(data).unwrap();
        let current_away_space = game.get_team_space(HomeOrAway::Home);
        let current_home_space = game.get_team_space(HomeOrAway::Away);

        let ball_direction = Some(game.ball_direction);

        let home_score = game.get_team_score(HomeOrAway::Home);
        let away_score = game.get_team_score(HomeOrAway::Away);

        Self {
            home,
            away,
            home_score,
            away_score,
            current_away_player,
            current_home_player,
            current_away_space,
            current_home_space,
            ball_direction,
            comment: String::new(),
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
        writeln!(output, "-------------------------").unwrap();
        writeln!(
            output,
            "{}: {}",
            data.get_player(&self.current_home_player)
                .unwrap()
                .get_name(),
            self.home_score,
        )
        .unwrap();

        writeln!(
            output,
            "{}: {}",
            data.get_player(&self.current_away_player)
                .unwrap()
                .get_name(),
            self.away_score,
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
