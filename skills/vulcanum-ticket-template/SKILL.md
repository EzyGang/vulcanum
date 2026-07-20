---
name: vulcanum-ticket-template
description: Use when a user asks to create, add, file, or draft a ticket, issue, task, or work item through the Vulcanum CLI. Defines the required title, Goal, Requirements, Technical Details, Dependencies, and Validation structure and the safe multiline `vulcanum board tasks create` workflow.
---

# Vulcanum Ticket Template

Create self-contained task-tracker tickets through the Vulcanum CLI. The ticket body is the contract for an implementation agent that may have no conversation history, so include enough project and product context to execute the work correctly.

Load the `vulcanum-cli` skill as well when login, team selection, project discovery, or command behavior is relevant.

## Required ticket structure

### Title

Use a short, focused title in imperative mood, under 80 characters. State one work item clearly.

Good: `Add rate limiting to the search endpoint`

Bad: `Search improvements and other fixes`

### Body

Use these exact sections in this order:

```markdown
Goal
<Business or product outcome, why it matters, and what success looks like.>

Requirements
- <Concrete, verifiable deliverable>
- <Concrete, verifiable deliverable>

Technical Details
- <Relevant architecture, files, interfaces, constraints, or implementation guidance>

Dependencies
<Blocking ticket ID or clear dependency description, or "None">

Validation
- <Objective check that must pass for the implementation to be successful>
- <Another required check>
```


#### Goal

Write from a business or product perspective. Make the goal self-contained: explain the current problem, desired outcome, relevant users or systems, and observable definition of success. Do not rely on phrases such as “as discussed,” “same as before,” or “finish the remaining work.”

#### Requirements

List concrete, independently verifiable deliverables. Capture behavior, boundaries, relevant interfaces, and error handling only when established by the conversation, repository, or existing documentation.

Do not guess requirements. First inspect available conversation context, code, configuration, documentation, and related tickets. If a material product choice remains unknowable and alternatives have different outcomes, ask the user before creating the ticket.

Good:

```markdown
- Limit each source IP to 100 search requests per minute
- Return HTTP 429 with a Retry-After header when the limit is exceeded
```

Bad:

```markdown
- Improve performance
- Make rate limiting work well
```

#### Technical Details

Record grounded implementation context that helps an agent execute efficiently without turning the section into a prescribed solution. Include relevant architecture boundaries, likely files or symbols, existing patterns to reuse, API or data contracts, migration concerns, and technical constraints when they are known.

Keep product behavior in Requirements. Use Technical Details for how the existing system is structured and which engineering constraints shape the implementation. Omit speculative guidance; inspect the repository before naming files, commands, or components.

#### Dependencies

Identify work that must complete before this ticket can start. Prefer a task slug such as `VLC-42` when available. If nothing blocks the task, write `None`; never omit the section.

#### Validation

Write a short checklist of objective conditions that must all pass for the implementation to be considered successful. Each bullet must be directly verifiable and specific to the work. Include repository-wide quality gates required by the project's instructions, relevant automated tests, and any necessary behavioral or manual checks.

Do not encode validation as a type or enum. Do not write vague bullets such as “works correctly” or prescribe irrelevant checks.

Example:

```markdown
Validation
- Applicable `AGENTS.md` instructions are followed and no code-writing rules are violated
- Unit tests covering the limiter boundary and reset behavior are added or updated and pass
- The relevant existing test suite passes
- Requests above the configured limit return HTTP 429 with a valid Retry-After header
```

## Resolve the destination

The create command requires Vulcanum's configured project UUID, not the provider's external project ID. Obtain it with:

```bash
vulcanum projects list [--team <UUID>]
```

If needed, inspect the board and valid initial columns before choosing `--status`:

```bash
vulcanum board view <PROJECT_ID> [--team <UUID>]
```

Do not invent a status or priority. Omit either option when the user and project context do not establish it.

## Create the ticket

Use `--body-stdin` for the required multiline body:

```bash
vulcanum board tasks create <PROJECT_ID> <TITLE> \
  --body-stdin \
  [--status <STATUS>] \
  [--priority <PRIORITY>] \
  [--team <UUID>]
```

`TITLE` is a positional argument. `--body-stdin` consumes the complete standard-input stream and conflicts with `--body`.

Prefer writing the reviewed body to a temporary text file and piping that file to the command. This avoids shell quoting damage and preserves multiline Markdown:

```bash
cat <BODY_FILE> | vulcanum board tasks create <PROJECT_ID> "<TITLE>" --body-stdin
```

PowerShell equivalent:

```powershell
Get-Content -Raw <BODY_FILE> | vulcanum board tasks create <PROJECT_ID> "<TITLE>" --body-stdin
```

Append `--status`, `--priority`, or `--team` only when their values are known and intended.

### Example body

```markdown
Goal
Protect the search API from abusive traffic so one client cannot degrade search availability for other tenants. Success means excess traffic is rejected predictably while normal traffic continues without behavior changes.

Requirements
- Limit each source IP to 100 search requests per minute
- Return HTTP 429 with a Retry-After header when the limit is exceeded
- Use the established client-IP resolution behavior for requests behind the configured proxy
- Add configuration for the request limit without changing the default behavior of unrelated endpoints

Technical Details
- Implement the limiter in the existing HTTP middleware layer
- Reuse the established client-IP extractor rather than parsing forwarding headers again
- Keep configuration in the existing server settings model and environment-loading path

Dependencies
None

Validation
- Applicable `AGENTS.md` instructions are followed and no code-writing rules are violated
- Tests cover requests below, at, and above the limit plus recovery after the window
- The relevant existing test suite passes
- Requests above the configured limit return HTTP 429 with a valid Retry-After header
```

## Pre-creation checklist

Before running the command, confirm:

- The title is focused, imperative, and under 80 characters.
- The body has Goal, Requirements, Technical Details, Dependencies, and Validation in that order.
- The goal is self-contained for an agent with no prior context.
- Every requirement is specific, verifiable, and grounded rather than guessed.
- Technical details are grounded in the repository and distinguish implementation context from product requirements.
- Dependencies are listed or explicitly `None`.
- Validation is a short list of objective checks that must all pass.
- The configured project UUID is known.
- Team, status, and priority are included only when intended.

If any material requirement or validation criterion remains unclear after inspecting available sources, resolve it with the user before creating the ticket.

## Completion

Run the create command and require a successful exit status. Report the created task slug or provider ID, title, and configured project. Do not claim creation when the CLI failed or only a draft was produced.