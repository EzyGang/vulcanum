use clap::Parser;
use uuid::Uuid;

use crate::commands::app::args::{
    DirectModelProviderAuth, ModelProvidersCommand, ProjectReposCommand, ProjectsCommand,
    RunsCommand, SettingsCommand, SettingsTeamCommand, WorkersCommand,
};
use crate::commands::app::board::args::{BoardCommand, BoardTasksCommand};
use crate::commands::app::projects::args::{ProjectAutomationCommand, ProjectColumnsCommand};
use crate::{Cli, Command, WorkerCommand};

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

    let runs = Cli::try_parse_from(["vulcanum", "runs", "list", "--team", TEAM])
        .expect("runs team override should parse");
    assert!(matches!(
        runs.command,
        Command::Runs {
            cmd: RunsCommand::List { team: Some(team) }
        } if team == expected
    ));

    let projects = Cli::try_parse_from([
        "vulcanum",
        "projects",
        "add",
        "--provider",
        "00000000-0000-0000-0000-000000000003",
        "--workspace",
        "core",
        "--project",
        "KAN",
        "--repo",
        "acme/api",
        "--team",
        TEAM,
    ])
    .expect("project add should parse");
    assert!(matches!(
        projects.command,
        Command::Projects {
            cmd: ProjectsCommand::Add {
                provider: Some(_),
                workspace: Some(workspace),
                project: Some(project),
                repos,
                team: Some(team),
            }
        } if workspace == "core"
            && project == "KAN"
            && repos == ["acme/api"]
            && team == expected
    ));

    let available_repos =
        Cli::try_parse_from(["vulcanum", "projects", "repos", "list", "--team", TEAM])
            .expect("available repositories should parse");
    assert!(matches!(
        available_repos.command,
        Command::Projects {
            cmd: ProjectsCommand::Repos {
                cmd: ProjectReposCommand::List { team: Some(team) }
            }
        } if team == expected
    ));

    let project_id = "00000000-0000-0000-0000-000000000004";
    let repos = Cli::try_parse_from([
        "vulcanum", "projects", "repos", "set", project_id, "--repo", "acme/api", "--repo",
        "acme/web", "--team", TEAM,
    ])
    .expect("project repositories should parse");
    assert!(matches!(
        repos.command,
        Command::Projects {
            cmd: ProjectsCommand::Repos {
                cmd: ProjectReposCommand::Set {
                    project_id: id,
                    repos,
                    clear: false,
                    team: Some(team),
                }
            }
        } if id == Uuid::from_u128(4)
            && repos == ["acme/api", "acme/web"]
            && team == expected
    ));

    assert!(Cli::try_parse_from([
        "vulcanum", "projects", "repos", "set", project_id, "--repo", "acme/api", "--clear",
    ])
    .is_err());

    let automation = Cli::try_parse_from([
        "vulcanum",
        "projects",
        "automation",
        "enable",
        project_id,
        "--team",
        TEAM,
    ])
    .expect("project automation should parse");
    assert!(matches!(
        automation.command,
        Command::Projects {
            cmd: ProjectsCommand::Automation {
                cmd: ProjectAutomationCommand::Enable {
                    project_id: id,
                    team: Some(team)
                }
            }
        } if id == Uuid::from_u128(4) && team == expected
    ));

    let columns = Cli::try_parse_from([
        "vulcanum",
        "projects",
        "columns",
        "set",
        project_id,
        "--pickup",
        "To Do",
        "--in-progress",
        "In Progress",
        "--in-review",
        "In Review",
        "--done",
        "Done",
    ])
    .expect("project columns should parse");
    assert!(matches!(
        columns.command,
        Command::Projects {
            cmd: ProjectsCommand::Columns {
                cmd: ProjectColumnsCommand::Set {
                    project_id: id,
                    pickup: Some(pickup),
                    in_progress: Some(progress),
                    in_review: Some(review),
                    done: Some(done),
                    ..
                }
            }
        } if id == Uuid::from_u128(4)
            && pickup == "To Do"
            && progress == "In Progress"
            && review == "In Review"
            && done == "Done"
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
fn settings_mutation_branches_parse() {
    for args in [
        vec![
            "vulcanum", "settings", "models", "primary", "set", "openai", "gpt-5",
        ],
        vec!["vulcanum", "settings", "models", "primary", "clear"],
        vec![
            "vulcanum",
            "settings",
            "models",
            "small",
            "set",
            "openai",
            "gpt-5-mini",
        ],
        vec!["vulcanum", "settings", "models", "small", "clear"],
        vec![
            "vulcanum",
            "settings",
            "task-trackers",
            "add",
            "--name",
            "Kaneo",
            "--instance-url",
            "https://tasks.example",
            "--credentials-stdin",
        ],
        vec![
            "vulcanum",
            "settings",
            "task-trackers",
            "update",
            TEAM,
            "--prompt-credentials",
        ],
        vec!["vulcanum", "settings", "task-trackers", "remove", TEAM],
        vec![
            "vulcanum",
            "settings",
            "model-providers",
            "add",
            "anthropic",
        ],
        vec![
            "vulcanum",
            "settings",
            "model-providers",
            "update",
            TEAM,
            "--auth",
            "none",
        ],
        vec!["vulcanum", "settings", "model-providers", "remove", TEAM],
        vec![
            "vulcanum",
            "settings",
            "model-providers",
            "connect-openai",
            "--no-browser",
        ],
        vec!["vulcanum", "settings", "github", "connect", "--no-browser"],
        vec!["vulcanum", "settings", "github", "disconnect"],
    ] {
        Cli::try_parse_from(args).expect("settings mutation branch should parse");
    }
}

#[test]
fn model_provider_add_defaults_to_api_key_auth() {
    let cli = Cli::try_parse_from([
        "vulcanum",
        "settings",
        "model-providers",
        "add",
        "anthropic",
    ])
    .expect("provider add should parse");
    assert!(matches!(
        cli.command,
        Command::Settings {
            cmd: SettingsCommand::ModelProviders {
                cmd: ModelProvidersCommand::Add {
                    auth: DirectModelProviderAuth::ApiKey,
                    ..
                }
            }
        }
    ));
}

#[test]
fn settings_credential_conflicts_and_invalid_values_fail_to_parse() {
    for args in [
        vec![
            "vulcanum",
            "settings",
            "task-trackers",
            "update",
            TEAM,
            "--credentials-stdin",
            "--prompt-credentials",
        ],
        vec![
            "vulcanum",
            "settings",
            "model-providers",
            "update",
            TEAM,
            "--credentials-stdin",
            "--prompt-credentials",
        ],
        vec!["vulcanum", "settings", "task-trackers", "remove", "invalid"],
        vec!["vulcanum", "settings", "model-providers", "add"],
        vec!["vulcanum", "settings", "models", "primary", "set", "openai"],
        vec![
            "vulcanum",
            "settings",
            "model-providers",
            "add",
            "openai",
            "--auth",
            "device-oauth",
        ],
    ] {
        assert!(Cli::try_parse_from(args).is_err());
    }
}

#[test]
fn board_command_branches_parse_with_project_guidance_inputs() {
    let project_id = "00000000-0000-0000-0000-000000000004";
    for args in [
        vec!["vulcanum", "board", "view", project_id, "--limit", "3"],
        vec![
            "vulcanum",
            "board",
            "column",
            project_id,
            "In Progress",
            "--page",
            "2",
            "--page-size",
            "10",
        ],
        vec![
            "vulcanum",
            "board",
            "tasks",
            "create",
            project_id,
            "New task",
            "--body-stdin",
        ],
        vec!["vulcanum", "board", "tasks", "get", project_id, "VLC-42"],
        vec![
            "vulcanum", "board", "tasks", "edit", project_id, "VLC-42", "--title", "Updated",
        ],
        vec![
            "vulcanum", "board", "tasks", "move", project_id, "VLC-42", "Done",
        ],
        vec![
            "vulcanum", "board", "tasks", "search", project_id, "--query", "parser", "--label",
            "backend",
        ],
    ] {
        Cli::try_parse_from(args).expect("board command branch should parse");
    }

    let get = Cli::try_parse_from(["vulcanum", "board", "tasks", "get", project_id, "VLC-42"])
        .expect("board task get should parse");
    assert!(matches!(
        get.command,
        Command::Board {
            cmd: BoardCommand::Tasks {
                cmd: BoardTasksCommand::Get {
                    project_id: id,
                    task,
                    ..
                }
            }
        } if id == Uuid::from_u128(4) && task == "VLC-42"
    ));

    assert!(Cli::try_parse_from([
        "vulcanum",
        "board",
        "tasks",
        "create",
        project_id,
        "New task",
        "--body",
        "inline",
        "--body-stdin",
    ])
    .is_err());
}

#[test]
fn malformed_team_uuid_fails_during_parsing() {
    for args in [
        vec!["vulcanum", "workers", "list", "--team", "not-a-uuid"],
        vec!["vulcanum", "runs", "list", "--team", "not-a-uuid"],
        vec!["vulcanum", "projects", "list", "--team", "not-a-uuid"],
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
