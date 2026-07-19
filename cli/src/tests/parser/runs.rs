use clap::Parser;
use uuid::Uuid;

use super::TEAM;
use crate::commands::app::args::RunsCommand;
use crate::{Cli, Command};

#[test]
fn run_list_team_override_parses() {
    let expected = Uuid::parse_str(TEAM).expect("team UUID should parse");
    let runs = Cli::try_parse_from(["vulcanum", "runs", "list", "--team", TEAM])
        .expect("runs team override should parse");
    assert!(matches!(
        runs.command,
        Command::Runs {
            cmd: RunsCommand::List { team: Some(team) }
        } if team == expected
    ));
}

#[test]
fn malformed_run_team_uuid_fails_during_parsing() {
    assert!(Cli::try_parse_from(["vulcanum", "runs", "list", "--team", "not-a-uuid",]).is_err());
}
