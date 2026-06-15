export const DEFAULT_PROJECT_PROMPT_TEMPLATE = `Review the following task and implement the required changes.

Task: {{task_title}}

Description:
{{task_body}}

Repository: {{repo_url}}

Follow the repository instructions and keep the final response concise.`;
