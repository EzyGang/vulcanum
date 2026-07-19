use clap::Parser;
use uuid::Uuid;

use super::TEAM;
use crate::commands::app::args::WorkersCommand;
use crate::{Cli, Command, WorkerCommand};

#[test]
fn worker_list_forms_parse_exactly() {
    let workers =
        Cli::try_parse_from(["vulcanum", "workers", "list"]).expect("workers list should parse");
    assert!(matches!(
        workers.command,
        Command::Workers {
            cmd: WorkersCommand::List { team: None }
        }
    ));

    let expected = Uuid::parse_str(TEAM).expect("team UUID should parse");
    let workers = Cli::try_parse_from(["vulcanum", "workers", "list", "--team", TEAM])
        .expect("workers team override should parse");
    assert!(matches!(
        workers.command,
        Command::Workers {
            cmd: WorkersCommand::List { team: Some(team) }
        } if team == expected
    ));
}

#[test]
fn malformed_worker_team_uuid_fails_during_parsing() {
    assert!(Cli::try_parse_from(["vulcanum", "workers", "list", "--team", "not-a-uuid",]).is_err());
}

#[test]
fn singular_worker_commands_remain_available() {
    let daemon =
        Cli::try_parse_from(["vulcanum", "worker", "daemon"]).expect("worker daemon should parse");
    assert!(matches!(
        daemon.command,
        Command::Worker {
            cmd: WorkerCommand::Daemon
        }
    ));

    let setup =
        Cli::try_parse_from(["vulcanum", "worker", "setup"]).expect("worker setup should parse");
    assert!(matches!(
        setup.command,
        Command::Worker {
            cmd: WorkerCommand::Setup { .. }
        }
    ));

    let delete = Cli::try_parse_from(["vulcanum", "worker", "self-delete"])
        .expect("worker self-delete should parse");
    assert!(matches!(
        delete.command,
        Command::Worker {
            cmd: WorkerCommand::SelfDelete
        }
    ));
}
