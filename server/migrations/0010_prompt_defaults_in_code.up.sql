UPDATE teams
SET prompt_template = ''
WHERE prompt_template LIKE 'Review {{task_title}}%'
  AND prompt_template LIKE '%{{task_body}}%'
  AND prompt_template LIKE '%{{repo_urls}}%'
  AND prompt_template LIKE '%Follow the repository instructions and keep the final response concise.%';

ALTER TABLE teams
    ALTER COLUMN prompt_template DROP DEFAULT;
