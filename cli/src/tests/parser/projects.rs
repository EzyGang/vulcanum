use clap::Parser;
use uuid::Uuid;

use super::TEAM;
use crate::commands::app::args::{ProjectReposCommand, ProjectsCommand};
use crate::commands::app::projects::args::{ProjectAutomationCommand, ProjectColumnsCommand};
use crate::{Cli, Command};

#[test]
fn project_command_forms_parse_exactly() {
    let expected = Uuid::parse_str(TEAM).expect("team UUID should parse");
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
}

#[test]
fn malformed_project_team_uuid_fails_during_parsing() {
    assert!(
        Cli::try_parse_from(["vulcanum", "projects", "list", "--team", "not-a-uuid",]).is_err()
    );
}
