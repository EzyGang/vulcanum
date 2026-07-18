use std::cell::RefCell;
use std::rc::Rc;

use chrono::Utc;
use vulcanum_shared::api::app::model_providers::CatalogProvider;

use crate::commands::app::settings::credentials::{
    model_provider_credentials, task_tracker_credentials,
};
use crate::commands::app::settings::runtime::SettingsRuntime;

#[test]
fn task_tracker_stdin_accepts_only_non_empty_api_key() {
    let mut runtime = runtime_with_input(r#"{"api_key":"tracker-secret"}"#, false);
    let credential = task_tracker_credentials(true, &mut runtime).expect("credential should parse");
    assert_eq!(credential, "tracker-secret");

    for input in [
        "[]",
        r#"{"unknown":"value"}"#,
        r#"{"api_key":7}"#,
        r#"{"api_key":""}"#,
    ] {
        let mut runtime = runtime_with_input(input, false);
        assert!(task_tracker_credentials(true, &mut runtime).is_err());
    }
}

#[test]
fn model_provider_stdin_requires_non_empty_string_map() {
    let mut runtime = runtime_with_input(r#"{"ANTHROPIC_API_KEY":"provider-secret"}"#, false);
    let credentials =
        model_provider_credentials(true, None, &mut runtime).expect("credentials should parse");
    assert_eq!(credentials["ANTHROPIC_API_KEY"], "provider-secret");

    for input in [
        "[]",
        "{}",
        r#"{"": "value"}"#,
        r#"{"KEY":""}"#,
        r#"{"KEY":7}"#,
    ] {
        let mut runtime = runtime_with_input(input, false);
        assert!(model_provider_credentials(true, None, &mut runtime).is_err());
    }
}

#[test]
fn prompt_mode_requires_terminal_and_prompts_catalog_fields_in_order() {
    let mut non_terminal = runtime_with_input("", false);
    let error = task_tracker_credentials(false, &mut non_terminal)
        .expect_err("non-terminal prompt should fail");
    assert_eq!(
        error.to_string(),
        "stdin is not a terminal; pass --credentials-stdin"
    );

    let prompts = Rc::new(RefCell::new(Vec::new()));
    let recorded = Rc::clone(&prompts);
    let mut runtime = runtime_with_input("", true);
    runtime.prompt_hidden = Box::new(move |label| {
        recorded.borrow_mut().push(label.to_owned());
        Ok(match label {
            "A_KEY" => "first-secret".to_owned(),
            _ => String::new(),
        })
    });
    let provider = CatalogProvider {
        id: "provider".to_owned(),
        name: "Provider".to_owned(),
        env: vec!["Z_KEY".to_owned(), "A_KEY".to_owned()],
        models: Vec::new(),
    };
    let credentials = model_provider_credentials(false, Some(&provider), &mut runtime)
        .expect("prompted credentials should parse");

    assert_eq!(&*prompts.borrow(), &["A_KEY", "Z_KEY"]);
    assert_eq!(credentials.as_object().map(serde_json::Map::len), Some(1));
}

#[test]
fn credential_errors_do_not_disclose_supplied_values() {
    let secret = "never-print-this-secret";
    let input = format!(r#"{{"unknown":"{secret}"}}"#);
    let mut runtime = runtime_with_input(&input, false);
    let error =
        task_tracker_credentials(true, &mut runtime).expect_err("unknown key should be rejected");

    assert!(!error.to_string().contains(secret));
    assert!(!format!("{error:?}").contains(secret));
}

fn runtime_with_input(input: &str, terminal: bool) -> SettingsRuntime {
    let input = input.to_owned();
    SettingsRuntime {
        stdin_is_terminal: terminal,
        stderr: Box::new(Vec::<u8>::new()),
        read_stdin: Box::new(move || Ok(input.clone())),
        prompt_hidden: Box::new(|_| Ok("prompt-secret".to_owned())),
        open_browser: Box::new(|_| Ok(())),
        sleep: Box::new(|_| Box::pin(async {})),
        now: Box::new(Utc::now),
    }
}
