use vulcanum_shared::api_types::WorkRunType;

const TOOL_PREFIX_TS: &str = r#"import { tool } from "@opencode-ai/plugin"
import { dirname, join } from "path"
import { mkdirSync, writeFileSync } from "fs"

function stringOrUndefined(value) {
  return typeof value === "string" && value.length > 0 ? value : undefined
}

function stringArrayOrEmpty(value) {
  if (!Array.isArray(value)) return []
  return value.filter((item) => typeof item === "string" && item.length > 0)
}

function artifactPath() {
  const configured = stringOrUndefined(process.env.FINISH_ARTIFACT_PATH)
  if (configured) return configured

  const home = stringOrUndefined(process.env.HOME)
  if (home) return join(home, "finish_artifact.json")

  return join(process.cwd(), "finish_artifact.json")
}

"#;

const TOOL_SUFFIX_TS: &str = r#"

    mkdirSync(dirname(path), { recursive: true })
    writeFileSync(path, JSON.stringify(artifact, null, 2))
    return `finish artifact written to ${path}`
  },
})
"#;

const IMPLEMENTATION_TOOL_BODY_TS: &str = r#"export default tool({
  description: "Call this when the implementation task is complete to submit the final result. REQUIRED at end of every run.",
  args: {
    status: tool.schema.enum(["completed", "failed", "blocked"]).describe("Outcome of the run"),
    summary: tool.schema.string().optional().describe("Brief summary of what was done and which formatter, validation, and test commands were run, or why blocked"),
    pr_urls: tool.schema.array(tool.schema.string()).optional().describe("URLs of all pull requests created across repositories"),
  },
  async execute(args) {
    const path = artifactPath()
    const artifact = {
      status: args.status,
      pr_urls: stringArrayOrEmpty(args.pr_urls),
      summary: stringOrUndefined(args.summary),
      review_url: undefined,
      review_body: undefined,
      review_already_exists: false,
    }"#;

const REVIEW_TOOL_BODY_TS: &str = r#"export default tool({
  description: "Call this when the pull request review is complete to submit the final result. REQUIRED at end of every review run.",
  args: {
    status: tool.schema.enum(["completed", "failed", "blocked"]).describe("Outcome of the review run"),
    summary: tool.schema.string().optional().describe("Brief summary of what was reviewed, what went wrong, or why blocked"),
    review_url: tool.schema.string().optional().describe("URL of the GitHub review that was posted"),
    review_body: tool.schema.string().optional().describe("Body of the GitHub review that was posted or already exists, including any missing or failing formatter, validation, or test commands"),
    review_already_exists: tool.schema.boolean().optional().describe("True only when a suitable review already exists for the current PR head commit. If the PR has new commits after the existing review, post a new review and leave this false."),
  },
  async execute(args) {
    const path = artifactPath()
    const artifact = {
      status: args.status,
      pr_urls: [],
      summary: stringOrUndefined(args.summary),
      review_url: stringOrUndefined(args.review_url),
      review_body: stringOrUndefined(args.review_body),
      review_already_exists: args.review_already_exists === true,
    }"#;

#[must_use]
pub fn finish_run_tool_ts(work_type: WorkRunType) -> String {
    format!(
        "{}{}{}",
        TOOL_PREFIX_TS,
        finish_tool_body_ts(work_type),
        TOOL_SUFFIX_TS
    )
}

#[must_use]
fn finish_tool_body_ts(work_type: WorkRunType) -> &'static str {
    match work_type {
        WorkRunType::Implementation => IMPLEMENTATION_TOOL_BODY_TS,
        WorkRunType::PullRequestReview => REVIEW_TOOL_BODY_TS,
    }
}
