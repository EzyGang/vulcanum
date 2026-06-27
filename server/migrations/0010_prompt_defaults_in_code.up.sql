UPDATE teams
SET prompt_template = ''
WHERE replace(prompt_template, E'\r\n', E'\n') = $vulcanum$Review {{task_title}}

{{task_body}}

Repositories:
{{repo_urls}}

Follow the repository instructions and keep the final response concise.$vulcanum$;

ALTER TABLE teams
    ALTER COLUMN prompt_template DROP DEFAULT;
