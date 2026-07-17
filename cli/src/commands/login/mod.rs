use std::io::{self, IsTerminal, Write};

use anyhow::Context;
use vulcanum_shared::client::{probe_url_with_scheme_fallback, ApiClient};
use vulcanum_shared::constants::DEFAULT_TEAM_ID;
use vulcanum_shared::state::app::{self as app_state, AppSession};

use crate::prompts::prompt_instance_url;

pub async fn run(
    instance: Option<String>,
    password_stdin: bool,
    auth_code: Option<String>,
    no_browser: bool,
) -> anyhow::Result<()> {
    let stdin_is_terminal = io::stdin().is_terminal();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut read_line = || {
        let mut value = String::new();
        io::stdin()
            .read_line(&mut value)
            .context("failed to read from stdin")?;
        Ok(value)
    };
    let mut prompt_instance = |initial: Option<String>| prompt_instance_url(initial.as_deref());
    let mut prompt_password = || {
        dialoguer::Password::new()
            .with_prompt("Instance password")
            .validate_with(|value: &String| match value.is_empty() {
                true => Err("Instance password is required"),
                false => Ok(()),
            })
            .interact()
            .map_err(Into::into)
    };
    let mut prompt_code = || {
        dialoguer::Input::<String>::new()
            .with_prompt("One-time code")
            .validate_with(|value: &String| match value.trim().is_empty() {
                true => Err("One-time code is required"),
                false => Ok(()),
            })
            .interact_text()
            .map_err(Into::into)
    };
    let mut open_browser = |url: &str| open::that(url).map_err(Into::into);
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = LoginRuntime {
        stdin_is_terminal,
        stdout: &mut stdout,
        stderr: &mut stderr,
        read_line: &mut read_line,
        prompt_instance: &mut prompt_instance,
        prompt_password: &mut prompt_password,
        prompt_code: &mut prompt_code,
        open_browser: &mut open_browser,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };

    run_with(
        instance,
        password_stdin,
        auth_code,
        no_browser,
        &mut runtime,
    )
    .await
}

struct LoginRuntime<'a> {
    stdin_is_terminal: bool,
    stdout: &'a mut dyn Write,
    stderr: &'a mut dyn Write,
    read_line: &'a mut dyn FnMut() -> anyhow::Result<String>,
    prompt_instance: &'a mut dyn FnMut(Option<String>) -> anyhow::Result<String>,
    prompt_password: &'a mut dyn FnMut() -> anyhow::Result<String>,
    prompt_code: &'a mut dyn FnMut() -> anyhow::Result<String>,
    open_browser: &'a mut dyn FnMut(&str) -> anyhow::Result<()>,
    load_session: &'a mut dyn FnMut() -> anyhow::Result<Option<AppSession>>,
    save_session: &'a mut dyn FnMut(&AppSession) -> anyhow::Result<()>,
}

async fn run_with(
    instance: Option<String>,
    password_stdin: bool,
    auth_code: Option<String>,
    no_browser: bool,
    runtime: &mut LoginRuntime<'_>,
) -> anyhow::Result<()> {
    let (selected_instance, existing_session) = resolve_instance(instance, runtime)?;
    let (canonical_instance, _) = probe_url_with_scheme_fallback(&selected_instance).await?;
    let client = ApiClient::new(&canonical_instance);
    let mode = client.auth_mode().await?;

    let tokens = match mode.is_single_user {
        true => {
            validate_single_user_flags(auth_code.as_deref(), no_browser)?;
            let password = read_password(password_stdin, runtime)?;
            client.instance_login(&password).await?
        }
        false => {
            if password_stdin {
                anyhow::bail!("--password-stdin is only valid for single-user instances");
            }
            let code = resolve_auth_code(auth_code, no_browser, &canonical_instance, runtime)?;
            client.exchange_auth_code(&code).await?
        }
    };
    let team_id = match mode.is_single_user {
        true => Some(DEFAULT_TEAM_ID),
        false => existing_session
            .filter(|session| session.instance_url == canonical_instance)
            .and_then(|session| session.team_id),
    };

    let session = AppSession {
        instance_url: canonical_instance.clone(),
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        refresh_expires_at: tokens.refresh_expires_at,
        team_id,
    };
    (runtime.save_session)(&session)?;
    writeln!(runtime.stderr, "  Logged in to {canonical_instance}")?;
    Ok(())
}

fn resolve_instance(
    instance: Option<String>,
    runtime: &mut LoginRuntime<'_>,
) -> anyhow::Result<(String, Option<AppSession>)> {
    match instance {
        Some(instance) => Ok((instance, None)),
        None if !runtime.stdin_is_terminal => {
            anyhow::bail!("stdin is not a terminal; pass --instance <URL>")
        }
        None => {
            let current = (runtime.load_session)()?;
            let initial = current.as_ref().map(|session| session.instance_url.clone());
            let selected = (runtime.prompt_instance)(initial)?;
            Ok((selected, current))
        }
    }
}

fn validate_single_user_flags(auth_code: Option<&str>, no_browser: bool) -> anyhow::Result<()> {
    if auth_code.is_some() {
        anyhow::bail!("--auth-code is only valid for multi-user instances");
    }
    if no_browser {
        anyhow::bail!("--no-browser is only valid for multi-user instances");
    }
    Ok(())
}

fn read_password(password_stdin: bool, runtime: &mut LoginRuntime<'_>) -> anyhow::Result<String> {
    let password = match password_stdin {
        true => {
            let mut value = (runtime.read_line)()?;
            if value.ends_with('\n') {
                value.pop();
                if value.ends_with('\r') {
                    value.pop();
                }
            }
            value
        }
        false if !runtime.stdin_is_terminal => {
            anyhow::bail!("stdin is not a terminal; pass --password-stdin")
        }
        false => (runtime.prompt_password)()?,
    };

    if password.is_empty() {
        anyhow::bail!("instance password is required");
    }
    Ok(password)
}

fn resolve_auth_code(
    auth_code: Option<String>,
    no_browser: bool,
    canonical_instance: &str,
    runtime: &mut LoginRuntime<'_>,
) -> anyhow::Result<String> {
    match auth_code {
        Some(code) => {
            let code = code.trim();
            if code.is_empty() {
                anyhow::bail!("--auth-code must not be empty");
            }
            Ok(code.to_owned())
        }
        None => {
            let login_url = github_login_url(canonical_instance)?;
            writeln!(runtime.stdout, "{login_url}")?;
            if !no_browser {
                match (runtime.open_browser)(login_url.as_str()) {
                    Ok(()) => (),
                    Err(error) => writeln!(
                        runtime.stderr,
                        "  [WARNING] Could not open the default browser: {error}"
                    )?,
                }
            }

            if !runtime.stdin_is_terminal {
                let instruction =
                    format!("vulcanum login --instance {canonical_instance} --auth-code <CODE>");
                writeln!(
                    runtime.stderr,
                    "Complete sign-in at the URL above, then rerun: {instruction}"
                )?;
                anyhow::bail!("one-time code requires terminal input");
            }

            let code = (runtime.prompt_code)()?;
            let code = code.trim();
            if code.is_empty() {
                anyhow::bail!("one-time code is required");
            }
            Ok(code.to_owned())
        }
    }
}

fn github_login_url(canonical_instance: &str) -> anyhow::Result<url::Url> {
    let mut url = url::Url::parse(canonical_instance)
        .with_context(|| format!("invalid instance URL: {canonical_instance}"))?
        .join("/api/v1/auth/github/start")?;
    url.query_pairs_mut().append_pair("return_to", "/cli-login");
    Ok(url)
}

#[cfg(test)]
mod tests;
