use std::io::{self, IsTerminal};

use dialoguer::{MultiSelect, Select};

pub(crate) struct ProjectsRuntime {
    pub(crate) stdin_is_terminal: bool,
    pub(crate) select: Box<Selector>,
    pub(crate) select_many: Box<MultiSelector>,
}

impl ProjectsRuntime {
    pub(super) fn real() -> Self {
        Self {
            stdin_is_terminal: io::stdin().is_terminal(),
            select: Box::new(select),
            select_many: Box::new(select_many),
        }
    }
}

pub(crate) type Selector = dyn FnMut(&str, &[String]) -> anyhow::Result<usize>;
pub(crate) type MultiSelector = dyn FnMut(&str, &[String], &[bool]) -> anyhow::Result<Vec<usize>>;

fn select(prompt: &str, labels: &[String]) -> anyhow::Result<usize> {
    Select::new()
        .with_prompt(prompt)
        .items(labels)
        .default(0)
        .interact()
        .map_err(Into::into)
}

fn select_many(prompt: &str, labels: &[String], defaults: &[bool]) -> anyhow::Result<Vec<usize>> {
    MultiSelect::new()
        .with_prompt(prompt)
        .items(labels)
        .defaults(defaults)
        .interact()
        .map_err(Into::into)
}
