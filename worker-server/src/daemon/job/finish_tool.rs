pub const FINISH_RUN_TOOL_TS: &str = r#"import { tool } from "@opencode-ai/plugin"
import { writeFileSync } from "fs"

export default tool({
  description: "Call this when the task is complete to submit the final result. REQUIRED at end of every run.",
  args: {
    status: tool.schema.enum(["completed", "failed", "blocked"]).describe("Outcome of the run"),
    pr_url: tool.schema.string().optional().describe("URL of the pull request, if created"),
    summary: tool.schema.string().optional().describe("Brief summary of what was done, what went wrong, or why blocked"),
    blocked_reason: tool.schema.string().optional().describe("If status is 'blocked', explain what input/approval is needed"),
    next_column: tool.schema.string().optional().describe("Suggested Kaneo column to move the task to (e.g. 'In Review', 'Blocked')"),
  },
  async execute(args) {
    writeFileSync(
      process.env.FINISH_ARTIFACT_PATH || `${process.env.HOME}/finish_artifact.json`,
      JSON.stringify(args, null, 2)
    )
    return { ok: true }
  },
})
"#;
