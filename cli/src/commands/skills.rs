use std::io::{self, Write};
use std::process::{Command as StdCommand, Stdio};

use anyhow::{bail, Context};
use clap::{Subcommand, ValueEnum};
use tokio::process::Command;

const REPOSITORY: &str = "EzyGang/vulcanum";
const CLI_SKILL: &str = include_str!("../../../skills/vulcanum-cli/SKILL.md");
const TICKET_TEMPLATE_SKILL: &str =
    include_str!("../../../skills/vulcanum-ticket-template/SKILL.md");

#[derive(Subcommand)]
pub(crate) enum SkillsCommand {
    /// Install or print Vulcanum's agent skills
    Install {
        /// Install or print one skill; omit to install both skills
        #[arg(value_enum)]
        skill: Option<Skill>,
        /// Print one skill to stdout instead of installing it
        #[arg(long, requires = "skill")]
        stdout: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum Skill {
    #[value(name = "cli", alias = "vulcanum-cli")]
    VulcanumCli,
    #[value(name = "ticket-template", alias = "vulcanum-ticket-template")]
    VulcanumTicketTemplate,
}

pub(crate) async fn run(command: SkillsCommand) -> anyhow::Result<()> {
    match command {
        SkillsCommand::Install { skill, stdout } => match (skill, stdout) {
            (Some(skill), true) => print_skill(skill),
            (skill, false) => install(skill).await,
            (None, true) => bail!("a skill is required when --stdout is used"),
        },
    }
}

fn print_skill(skill: Skill) -> anyhow::Result<()> {
    io::stdout()
        .lock()
        .write_all(skill.content().as_bytes())
        .context("failed to write skill to stdout")
}

async fn install(skill: Option<Skill>) -> anyhow::Result<()> {
    let runner = PackageRunner::detect()
        .context("no supported JavaScript package runner found; install pnpm, npm, Bun, or Yarn")?;
    let command = InstallerCommand::new(runner, skill);
    let status = Command::new(command.executable)
        .args(command.args)
        .status()
        .await
        .with_context(|| format!("failed to start {}", command.executable))?;

    if !status.success() {
        bail!(
            "skill installer exited with status {}",
            status.code().map_or_else(
                || "terminated by signal".to_owned(),
                |code| code.to_string()
            )
        );
    }

    Ok(())
}

impl Skill {
    const ALL: [Self; 2] = [Self::VulcanumCli, Self::VulcanumTicketTemplate];

    const fn name(self) -> &'static str {
        match self {
            Self::VulcanumCli => "vulcanum-cli",
            Self::VulcanumTicketTemplate => "vulcanum-ticket-template",
        }
    }

    const fn content(self) -> &'static str {
        match self {
            Self::VulcanumCli => CLI_SKILL,
            Self::VulcanumTicketTemplate => TICKET_TEMPLATE_SKILL,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PackageRunner {
    Pnpm,
    Npx,
    Bunx,
    Yarn,
}

impl PackageRunner {
    const ALL: [Self; 4] = [Self::Pnpm, Self::Npx, Self::Bunx, Self::Yarn];

    fn detect() -> Option<Self> {
        Self::ALL.into_iter().find(|runner| runner.is_available())
    }

    fn is_available(self) -> bool {
        StdCommand::new(self.executable())
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    const fn executable(self) -> &'static str {
        match self {
            Self::Pnpm => "pnpm",
            Self::Npx => "npx",
            Self::Bunx => "bunx",
            Self::Yarn => "yarn",
        }
    }

    const fn prefix(self) -> &'static [&'static str] {
        match self {
            Self::Pnpm => &["dlx", "skills"],
            Self::Npx | Self::Bunx => &["skills"],
            Self::Yarn => &["dlx", "skills"],
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct InstallerCommand {
    pub(super) executable: &'static str,
    pub(super) args: Vec<&'static str>,
}

impl InstallerCommand {
    pub(super) fn new(runner: PackageRunner, skill: Option<Skill>) -> Self {
        let mut args = Vec::from(runner.prefix());
        args.extend(["add", REPOSITORY]);

        match skill {
            Some(skill) => args.extend(["--skill", skill.name()]),
            None => {
                for skill in Skill::ALL {
                    args.extend(["--skill", skill.name()]);
                }
            }
        }

        Self {
            executable: runner.executable(),
            args,
        }
    }
}
