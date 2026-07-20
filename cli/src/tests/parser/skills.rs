use clap::Parser;

use crate::commands::skills::{Skill, SkillsCommand};
use crate::{Cli, Command};

#[test]
fn skills_install_parses_all_and_named_modes() {
    let all = Cli::try_parse_from(["vulcanum", "skills", "install"])
        .expect("all-skills install should parse");
    assert!(matches!(
        all.command,
        Command::Skills {
            cmd: SkillsCommand::Install {
                skill: None,
                stdout: false,
            }
        }
    ));

    let stdout = Cli::try_parse_from([
        "vulcanum",
        "skills",
        "install",
        "ticket-template",
        "--stdout",
    ])
    .expect("named stdout install should parse");
    assert!(matches!(
        stdout.command,
        Command::Skills {
            cmd: SkillsCommand::Install {
                skill: Some(Skill::VulcanumTicketTemplate),
                stdout: true,
            }
        }
    ));
}

#[test]
fn skills_install_stdout_requires_one_skill() {
    assert!(Cli::try_parse_from(["vulcanum", "skills", "install", "--stdout"]).is_err());
}

#[test]
fn skills_install_accepts_canonical_skill_names() {
    for skill in ["vulcanum-cli", "vulcanum-ticket-template"] {
        Cli::try_parse_from(["vulcanum", "skills", "install", skill])
            .expect("canonical skill name should parse");
    }
}
