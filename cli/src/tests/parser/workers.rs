use clap::Parser;
use uuid::Uuid;

use super::TEAM;
use crate::commands::app::args::WorkersCommand;
use crate::{Cli, Command};
#[cfg(not(target_os = "windows"))]
use crate::{WorkerCommand, WorkerUpdatesCommand};

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
fn worker_rename_forms_parse_exactly() {
    let worker_id = Uuid::from_u128(7);
    let workers = Cli::try_parse_from([
        "vulcanum",
        "workers",
        "rename",
        &worker_id.to_string(),
        "build-a",
    ])
    .expect("worker rename should parse");
    assert!(matches!(
        workers.command,
        Command::Workers {
            cmd: WorkersCommand::Rename {
                worker_id: parsed_id,
                name,
                team: None,
            }
        } if parsed_id == worker_id && name == "build-a"
    ));

    let expected_team = Uuid::parse_str(TEAM).expect("team UUID should parse");
    let workers = Cli::try_parse_from([
        "vulcanum",
        "workers",
        "rename",
        &worker_id.to_string(),
        "build-a",
        "--team",
        TEAM,
    ])
    .expect("worker rename with team override should parse");
    assert!(matches!(
        workers.command,
        Command::Workers {
            cmd: WorkersCommand::Rename {
                worker_id: parsed_id,
                name,
                team: Some(team),
            }
        } if parsed_id == worker_id && name == "build-a" && team == expected_team
    ));
}

#[test]
fn malformed_worker_team_uuid_fails_during_parsing() {
    assert!(Cli::try_parse_from(["vulcanum", "workers", "list", "--team", "not-a-uuid",]).is_err());
}

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
#[test]
fn worker_update_controls_parse_exactly() {
    let enable = Cli::try_parse_from(["vulcanum", "worker", "updates", "enable"])
        .expect("worker updates enable command should parse");
    assert!(matches!(
        enable.command,
        Command::Worker {
            cmd: WorkerCommand::Updates {
                cmd: WorkerUpdatesCommand::Enable,
            }
        }
    ));

    let disable = Cli::try_parse_from(["vulcanum", "worker", "updates", "disable"])
        .expect("worker updates disable command should parse");
    assert!(matches!(
        disable.command,
        Command::Worker {
            cmd: WorkerCommand::Updates {
                cmd: WorkerUpdatesCommand::Disable,
            }
        }
    ));
}

#[cfg(target_os = "windows")]
#[test]
fn singular_worker_namespace_is_not_available() {
    assert!(Cli::try_parse_from(["vulcanum", "worker", "updates", "enable"]).is_err());
}
