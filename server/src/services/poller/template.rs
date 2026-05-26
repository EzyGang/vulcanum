pub struct TemplateVars<'a> {
    pub task_title: &'a str,
    pub task_body: &'a str,
    pub repo_url: &'a str,
}

pub fn render_template(template: &str, vars: &TemplateVars<'_>) -> String {
    template
        .replace("{{task_title}}", vars.task_title)
        .replace("{{task_body}}", vars.task_body)
        .replace("{{repo_url}}", vars.repo_url)
}
