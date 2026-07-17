pub(crate) fn normalize_instance_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    }
}

pub(crate) fn nonempty(field: &str, input: &str) -> Result<(), String> {
    if input.trim().is_empty() {
        return Err(format!("{field} is required"));
    }
    Ok(())
}

pub(crate) fn prompt_instance_url(initial: Option<&str>) -> anyhow::Result<String> {
    let mut prompt = dialoguer::Input::<String>::new().with_prompt("Instance URL");
    if let Some(url) = initial {
        prompt = prompt.default(url.to_owned());
    }

    let url = prompt
        .validate_with(|input: &String| {
            let normalized = normalize_instance_url(input);
            nonempty("Instance URL", input)?;
            match url::Url::parse(&normalized) {
                Ok(_) => Ok(()),
                Err(_) => Err("Please enter a valid URL".to_owned()),
            }
        })
        .interact_text()?;
    Ok(normalize_instance_url(&url))
}
