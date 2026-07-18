use std::future::Future;
use std::io::{self, IsTerminal, Read, Write};
use std::pin::Pin;
use std::time::Duration;

use chrono::{DateTime, Utc};
use dialoguer::Password;

type StdinReader = dyn FnMut() -> anyhow::Result<String>;
type HiddenPrompt = dyn FnMut(&str) -> anyhow::Result<String>;
type BrowserOpener = dyn FnMut(&str) -> anyhow::Result<()>;
type SleepFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
type Sleeper = dyn FnMut(Duration) -> SleepFuture;
type Clock = dyn FnMut() -> DateTime<Utc>;

pub(super) struct SettingsRuntime {
    pub(super) stdin_is_terminal: bool,
    pub(super) stderr: Box<dyn Write>,
    pub(super) read_stdin: Box<StdinReader>,
    pub(super) prompt_hidden: Box<HiddenPrompt>,
    pub(super) open_browser: Box<BrowserOpener>,
    pub(super) sleep: Box<Sleeper>,
    pub(super) now: Box<Clock>,
}

impl SettingsRuntime {
    pub(super) fn real() -> Self {
        Self {
            stdin_is_terminal: stdin_is_terminal(),
            stderr: Box::new(io::stderr()),
            read_stdin: Box::new(read_stdin),
            prompt_hidden: Box::new(prompt_hidden),
            open_browser: Box::new(open_browser),
            sleep: Box::new(sleep),
            now: Box::new(now),
        }
    }
}

pub(super) fn stdin_is_terminal() -> bool {
    io::stdin().is_terminal()
}

pub(super) fn read_stdin() -> anyhow::Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

pub(super) fn prompt_hidden(label: &str) -> anyhow::Result<String> {
    Password::new()
        .with_prompt(label)
        .allow_empty_password(true)
        .interact()
        .map_err(Into::into)
}

pub(super) fn open_browser(url: &str) -> anyhow::Result<()> {
    open::that(url).map_err(Into::into)
}

pub(super) fn sleep(duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    Box::pin(tokio::time::sleep(duration))
}

pub(super) fn now() -> DateTime<Utc> {
    Utc::now()
}
