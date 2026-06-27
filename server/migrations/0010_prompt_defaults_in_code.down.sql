ALTER TABLE teams
    ALTER COLUMN prompt_template SET DEFAULT E'Review {{task_title}}\r\n\r\n{{task_body}}\r\n\r\nRepositories:\r\n{{repo_urls}}\r\n\r\nFollow the repository instructions and keep the final response concise.';

UPDATE teams
SET prompt_template = E'Review {{task_title}}\r\n\r\n{{task_body}}\r\n\r\nRepositories:\r\n{{repo_urls}}\r\n\r\nFollow the repository instructions and keep the final response concise.'
WHERE prompt_template = '';
