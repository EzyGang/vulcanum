use vulcanum_shared::api_types::WorkRunType;

const OPENCODE_TOOL_PREFIX_TS: &str = r#"import { tool } from "@opencode-ai/plugin"
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

const OPENCODE_TOOL_SUFFIX_TS: &str = r#"

    mkdirSync(dirname(path), { recursive: true })
    writeFileSync(path, JSON.stringify(artifact, null, 2))
    return `finish artifact written to ${path}`
  },
})
"#;

const OPENCODE_IMPLEMENTATION_TOOL_BODY_TS: &str = r#"export default tool({
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

const OPENCODE_REVIEW_TOOL_BODY_TS: &str = r#"export default tool({
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

const OMP_TOOL_PREFIX_TS: &str = r#"import { dirname, join } from "node:path"
import { mkdirSync, writeFileSync } from "node:fs"
import { z } from "zod"

export const name = "finish_run"
export const description = "Call this when the Vulcanum job is complete. REQUIRED at end of every run."

function stringOrUndefined(value: unknown): string | undefined {
  return typeof value === "string" && value.length > 0 ? value : undefined
}

function stringArrayOrEmpty(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.filter((item): item is string => typeof item === "string" && item.length > 0)
}

function artifactPath(): string {
  const configured = stringOrUndefined(process.env.FINISH_ARTIFACT_PATH)
  if (configured) return configured

  const home = stringOrUndefined(process.env.HOME)
  if (home) return join(home, "finish_artifact.json")

  return join(process.cwd(), "finish_artifact.json")
}

"#;

const OMP_TOOL_SUFFIX_TS: &str = r#"

  const path = artifactPath()
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, JSON.stringify(artifact, null, 2))
  return `finish artifact written to ${path}`
}
"#;

const OMP_IMPLEMENTATION_TOOL_BODY_TS: &str = r#"export const parameters = z.object({
  status: z.enum(["completed", "failed", "blocked"]),
  summary: z.string().optional(),
  pr_urls: z.array(z.string()).optional(),
})

export default async function run(input: z.infer<typeof parameters>) {
  const artifact = {
    status: input.status,
    pr_urls: stringArrayOrEmpty(input.pr_urls),
    summary: stringOrUndefined(input.summary),
    review_url: undefined,
    review_body: undefined,
    review_already_exists: false,
  }"#;

const OMP_REVIEW_TOOL_BODY_TS: &str = r#"export const parameters = z.object({
  status: z.enum(["completed", "failed", "blocked"]),
  summary: z.string().optional(),
  review_url: z.string().optional(),
  review_body: z.string().optional(),
  review_already_exists: z.boolean().optional(),
})

export default async function run(input: z.infer<typeof parameters>) {
  const artifact = {
    status: input.status,
    pr_urls: [],
    summary: stringOrUndefined(input.summary),
    review_url: stringOrUndefined(input.review_url),
    review_body: stringOrUndefined(input.review_body),
    review_already_exists: input.review_already_exists === true,
  }"#;

pub trait FinishToolRenderer {
    fn render(&self, work_type: WorkRunType) -> String;
}

pub struct OpenCodeFinishToolRenderer;

impl FinishToolRenderer for OpenCodeFinishToolRenderer {
    fn render(&self, work_type: WorkRunType) -> String {
        format!(
            "{}{}{}",
            OPENCODE_TOOL_PREFIX_TS,
            opencode_finish_tool_body_ts(work_type),
            OPENCODE_TOOL_SUFFIX_TS
        )
    }
}

pub struct OmpFinishToolRenderer;

impl FinishToolRenderer for OmpFinishToolRenderer {
    fn render(&self, work_type: WorkRunType) -> String {
        format!(
            "{}{}{}",
            OMP_TOOL_PREFIX_TS,
            omp_finish_tool_body_ts(work_type),
            OMP_TOOL_SUFFIX_TS
        )
    }
}

#[must_use]
pub fn finish_run_tool_ts(work_type: WorkRunType) -> String {
    let renderer = OpenCodeFinishToolRenderer;
    renderer.render(work_type)
}

#[must_use]
pub fn omp_finish_run_tool_ts(work_type: WorkRunType) -> String {
    let renderer = OmpFinishToolRenderer;
    renderer.render(work_type)
}

#[must_use]
fn opencode_finish_tool_body_ts(work_type: WorkRunType) -> &'static str {
    match work_type {
        WorkRunType::Implementation => OPENCODE_IMPLEMENTATION_TOOL_BODY_TS,
        WorkRunType::PullRequestReview => OPENCODE_REVIEW_TOOL_BODY_TS,
    }
}

#[must_use]
fn omp_finish_tool_body_ts(work_type: WorkRunType) -> &'static str {
    match work_type {
        WorkRunType::Implementation => OMP_IMPLEMENTATION_TOOL_BODY_TS,
        WorkRunType::PullRequestReview => OMP_REVIEW_TOOL_BODY_TS,
    }
}
