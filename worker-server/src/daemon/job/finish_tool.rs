pub const FINISH_RUN_TOOL_TS: &str = r#"import { tool } from "@opencode-ai/plugin"
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

export default tool({
  description: "Call this when the task is complete to submit the final result. REQUIRED at end of every run.",
  args: {
    status: tool.schema.enum(["completed", "failed", "blocked"]).describe("Outcome of the run"),
    pr_urls: tool.schema.array(tool.schema.string()).optional().describe("URLs of all pull requests created across repositories"),
    summary: tool.schema.string().optional().describe("Brief summary of what was done, what went wrong, or why blocked"),
    review_url: tool.schema.string().optional().describe("URL of the GitHub review that was posted, for review runs"),
    review_body: tool.schema.string().optional().describe("Body of the GitHub review that was posted, for review runs"),
    review_already_exists: tool.schema.boolean().optional().describe("True when the required Vulcanum review marker already existed and no duplicate review was posted"),
  },
  async execute(args) {
    const path = artifactPath()
    const artifact = {
      status: args.status,
      pr_urls: stringArrayOrEmpty(args.pr_urls),
      summary: stringOrUndefined(args.summary),
      review_url: stringOrUndefined(args.review_url),
      review_body: stringOrUndefined(args.review_body),
      review_already_exists: args.review_already_exists === true,
    }

    mkdirSync(dirname(path), { recursive: true })
    writeFileSync(path, JSON.stringify(artifact, null, 2))
    return `finish artifact written to ${path}`
  },
})
"#;
