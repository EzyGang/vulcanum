use super::template::{render_template, TemplateVars};

#[test]
fn interpolates_all_vars() {
    let template = "Task: {{task_title}}\nBody: {{task_body}}\nRepo: {{repo_url}}\nRepos: {{repo_urls}}\nNames: {{repo_names}}\nLayout: {{repo_layout}}";
    let vars = TemplateVars {
        task_title: "Fix login bug",
        task_body: "The login form crashes on submit.",
        repo_url: "https://github.com/org/repo",
        repo_urls: "https://github.com/org/repo\nhttps://github.com/org/other",
        repo_names: "org/repo\norg/other",
        repo_layout: "org/repo: ./org-repo\norg/other: ./org-other",
        review_target_pr_url: "",
        review_marker: "",
    };
    let result = render_template(template, &vars);

    assert!(result.contains("Fix login bug"));
    assert!(result.contains("The login form crashes on submit."));
    assert!(result.contains("https://github.com/org/repo"));
    assert!(result.contains("https://github.com/org/other"));
    assert!(result.contains("org/repo"));
    assert!(result.contains("org/other"));
    assert!(result.contains("org/repo: ./org-repo"));
    assert!(result.contains("org/other: ./org-other"));
    assert!(!result.contains("{{"));
}

#[test]
fn preserves_unknown_vars() {
    let template = "Unknown {{foo}} and {{bar}}";
    let vars = TemplateVars {
        task_title: "",
        task_body: "",
        repo_url: "",
        repo_urls: "",
        repo_names: "",
        repo_layout: "",
        review_target_pr_url: "",
        review_marker: "",
    };
    let result = render_template(template, &vars);
    assert_eq!(result, "Unknown {{foo}} and {{bar}}");
}

#[test]
fn handles_empty_template() {
    let vars = TemplateVars {
        task_title: "",
        task_body: "",
        repo_url: "",
        repo_urls: "",
        repo_names: "",
        repo_layout: "",
        review_target_pr_url: "",
        review_marker: "",
    };
    assert_eq!(render_template("", &vars), "");
}

#[test]
fn handles_partial_template() {
    let template = "Only title: {{task_title}}";
    let vars = TemplateVars {
        task_title: "My Task",
        task_body: "",
        repo_url: "",
        repo_urls: "",
        repo_names: "",
        repo_layout: "",
        review_target_pr_url: "",
        review_marker: "",
    };
    assert_eq!(render_template(template, &vars), "Only title: My Task");
}
