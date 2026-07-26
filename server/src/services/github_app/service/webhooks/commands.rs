pub(super) enum CommentCommand {
    Review(Option<String>),
    Implement {
        project_selector: Option<String>,
        ticket_selector: Option<String>,
        request_body: String,
    },
    MalformedImplementation,
    AmbiguousImplementation,
}

pub(super) fn comment_command(body: &str, app_slug: &str) -> Option<CommentCommand> {
    let body_bytes = body.as_bytes();
    let mention = format!("@{app_slug}").to_ascii_lowercase();
    let mention = mention.as_bytes();
    if mention.len() > body_bytes.len() {
        return None;
    }

    let mut found: Option<CommentCommand> = None;
    let mut saw_review_attempt = false;
    for index in 0..=body_bytes.len() - mention.len() {
        let end = index + mention.len();
        if !body_bytes[index..end].eq_ignore_ascii_case(mention)
            || !boundary_before(body_bytes, index)
            || !boundary_after(body_bytes, end)
        {
            continue;
        }
        let suffix = body[end..].trim_start();
        if keyword(suffix, "review") {
            saw_review_attempt = true;
        }
        let Some(command) = parse_comment_command(suffix) else {
            continue;
        };
        if saw_review_attempt && !matches!(&command, CommentCommand::Review(_)) {
            return Some(CommentCommand::AmbiguousImplementation);
        }
        found = match found {
            None => Some(command),
            Some(CommentCommand::Review(selector))
                if matches!(command, CommentCommand::Review(_)) =>
            {
                Some(CommentCommand::Review(selector))
            }
            Some(_) => return Some(CommentCommand::AmbiguousImplementation),
        };
    }

    found
}

fn parse_comment_command(suffix: &str) -> Option<CommentCommand> {
    let suffix = suffix.trim_start();
    if keyword(suffix, "review") {
        return parse_review_command(&suffix[6..]).map(CommentCommand::Review);
    }
    if !keyword(suffix, "implement") {
        return None;
    }

    Some(parse_implementation_command(&suffix[9..]))
}

fn parse_review_command(remainder: &str) -> Option<Option<String>> {
    let selector = trim_selector(remainder.trim());
    if selector.is_empty() {
        return Some(None);
    }
    if selector.split_whitespace().count() != 1
        || !selector
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("project:"))
    {
        return None;
    }

    Some(Some(selector.to_owned()))
}

fn parse_implementation_command(remainder: &str) -> CommentCommand {
    let remainder = remainder.trim_start();
    if remainder.is_empty() {
        return CommentCommand::MalformedImplementation;
    }

    let (project_selector, remainder) = take_selector(remainder, "project:");
    let (ticket_selector, request_body) = take_selector(remainder, "ticket:");
    if request_body.is_empty() {
        return CommentCommand::MalformedImplementation;
    }

    CommentCommand::Implement {
        project_selector,
        ticket_selector: ticket_selector.map(|selector| selector[7..].to_owned()),
        request_body: request_body.to_owned(),
    }
}

fn take_selector<'a>(value: &'a str, prefix: &str) -> (Option<String>, &'a str) {
    let token_end = value.find(char::is_whitespace).unwrap_or(value.len());
    let token = trim_selector(&value[..token_end]);
    if !token
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
    {
        return (None, value);
    }

    (Some(token.to_owned()), value[token_end..].trim_start())
}

fn keyword(value: &str, expected: &str) -> bool {
    value
        .get(..expected.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(expected))
        && value[expected.len()..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace)
}

fn trim_selector(value: &str) -> &str {
    value.trim_matches(|character: char| {
        matches!(
            character,
            '`' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}'
        )
    })
}

fn boundary_before(body: &[u8], index: usize) -> bool {
    index == 0 || !is_login_byte(body[index - 1])
}

fn boundary_after(body: &[u8], index: usize) -> bool {
    index == body.len() || !is_login_byte(body[index])
}

fn is_login_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_')
}
