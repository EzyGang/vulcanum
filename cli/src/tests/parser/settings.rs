use clap::Parser;
use uuid::Uuid;

use super::TEAM;
use crate::commands::app::args::{
    DirectModelProviderAuth, ModelProvidersCommand, SettingsCommand, SettingsTeamCommand,
};
use crate::{Cli, Command};

#[test]
fn settings_team_forms_parse_exactly() {
    let expected = Uuid::parse_str(TEAM).expect("team UUID should parse");
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
fn malformed_settings_team_uuid_fails_during_parsing() {
    for args in [
        vec!["vulcanum", "settings", "list", "--team", "not-a-uuid"],
        vec!["vulcanum", "settings", "team", "set", "not-a-uuid"],
    ] {
        assert!(Cli::try_parse_from(args).is_err());
    }
}
