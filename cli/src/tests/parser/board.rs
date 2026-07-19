use clap::Parser;
use uuid::Uuid;

use crate::commands::app::board::args::{BoardCommand, BoardTasksCommand};
use crate::{Cli, Command};

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
