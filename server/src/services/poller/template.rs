pub struct TemplateVars<'a> {
    pub task_title: &'a str,
    pub task_body: &'a str,
    pub repo_url: &'a str,
    pub repo_urls: &'a str,
    pub repo_names: &'a str,
    pub repo_layout: &'a str,
    pub review_target_pr_url: &'a str,
}

#[must_use]
pub fn render_template(template: &str, vars: &TemplateVars<'_>) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        let (before, after_start) = rest.split_at(start);
        rendered.push_str(before);

        let Some(end) = after_start.find("}}") else {
            rendered.push_str(after_start);
            return rendered;
        };

        let token = &after_start[..end + 2];
        rendered.push_str(match token {
            "{{task_title}}" => vars.task_title,
            "{{task_body}}" => vars.task_body,
            "{{repo_url}}" => vars.repo_url,
            "{{repo_urls}}" => vars.repo_urls,
            "{{repo_names}}" => vars.repo_names,
            "{{repo_layout}}" => vars.repo_layout,
            "{{review_target_pr_url}}" => vars.review_target_pr_url,
            _ => token,
        });
        rest = &after_start[end + 2..];
    }

    rendered.push_str(rest);
    rendered
}
