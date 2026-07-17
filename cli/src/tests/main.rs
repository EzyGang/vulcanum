use clap::Parser;
use uuid::Uuid;

use crate::{Cli, Command, SettingsCommand, SettingsTeamCommand, WorkerCommand, WorkersCommand};

const TEAM: &str = "00000000-0000-0000-0000-00000000002A";

#[test]
fn app_command_forms_parse_exactly() {
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

    let settings = Cli::try_parse_from(["vulcanum", "settings", "list", "--team", TEAM])
        .expect("settings team override should parse");
    assert!(matches!(
        settings.command,
        Command::Settings {
            cmd: SettingsCommand::List { team: Some(team) }
        } if team == expected
    ));

    let set = Cli::try_parse_from(["vulcanum", "settings", "team", "set", TEAM])
        .expect("team set should parse");
    assert!(matches!(
        set.command,
        Command::Settings {
            cmd: SettingsCommand::Team {
                cmd: SettingsTeamCommand::Set { team }
            }
        } if team == expected
    ));

    let clear = Cli::try_parse_from(["vulcanum", "settings", "team", "clear"])
        .expect("team clear should parse");
    assert!(matches!(
        clear.command,
        Command::Settings {
            cmd: SettingsCommand::Team {
                cmd: SettingsTeamCommand::Clear
            }
        }
    ));
}

#[test]
fn malformed_team_uuid_fails_during_parsing() {
    for args in [
        vec!["vulcanum", "workers", "list", "--team", "not-a-uuid"],
        vec!["vulcanum", "settings", "list", "--team", "not-a-uuid"],
        vec!["vulcanum", "settings", "team", "set", "not-a-uuid"],
    ] {
        assert!(Cli::try_parse_from(args).is_err());
    }
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
