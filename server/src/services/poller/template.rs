pub struct TemplateVars<'a> {
    pub task_title: &'a str,
    pub task_body: &'a str,
    pub repo_url: &'a str,
    pub repo_urls: &'a str,
    pub repo_names: &'a str,
    pub repo_layout: &'a str,
    pub review_target_pr_url: &'a str,
    pub review_marker: &'a str,
}

pub fn render_template(template: &str, vars: &TemplateVars<'_>) -> String {
    template
        .replace("{{task_title}}", vars.task_title)
        .replace("{{task_body}}", vars.task_body)
        .replace("{{repo_url}}", vars.repo_url)
        .replace("{{repo_urls}}", vars.repo_urls)
        .replace("{{repo_names}}", vars.repo_names)
        .replace("{{repo_layout}}", vars.repo_layout)
        .replace("{{review_target_pr_url}}", vars.review_target_pr_url)
        .replace("{{review_marker}}", vars.review_marker)
}
