# Proportional Development Workflow

Use the least bureaucratic planning level compatible with a task's impact. The
number of changed files does not determine the level by itself.

## Planning levels

### Direct

Use for small, localized, reversible work whose solution is reasonably clear:
bug fixes, regression tests, documentation, visual adjustments, warnings,
private refactors, maintenance, and mechanical changes that do not alter a
public contract.

Investigate, implement, validate, and report. Do not create an OpenSpec change,
proposal, design, delta spec, task file, ADR, or GitHub Issue automatically.

### Issue

Use a GitHub Issue as the primary contract for relevant user-visible work with
clear design and PR-sized scope, including non-trivial bugs, CLI options,
composed components, scaffolding improvements, and changes that need acceptance
criteria or public traceability.

Prefer an existing Issue. If none exists, draft or agree on a concise contract
without mutating GitHub unless the user explicitly requests it. Include only the
sections the work needs:

```markdown
## Problem

## Expected result

## Scope

## Out of scope

## Acceptance criteria

## Validation evidence
```

A short implementation plan may remain conversational. Do not duplicate the
Issue into an OpenSpec change.

### OpenSpec

Reserve OpenSpec for structural or contractual work: public API or lifecycle
redesigns, breaking changes, runtime architecture, new framework concepts,
significant CLI contracts, cross-crate behavior, important competing designs,
hard-to-reverse decisions, migrations, formal compatibility requirements, or
work expected across multiple PRs.

The GitHub Issue represents the public problem and objective. The OpenSpec
change records detailed requirements, contracts, and decisions. Public docs and
GitHub Project state remain authoritative for their own concerns.

## Classification

Consider whether the work changes public API, can break consumers, introduces
an architectural concept, crosses important crate boundaries, has meaningful
design alternatives, requires migration or compatibility policy, spans several
PRs, or must preserve formal requirements beyond an Issue.

- No relevant structural impact: use `Direct`.
- Moderate functional impact with clear design: use `Issue`.
- Several structural or contractual concerns: recommend `OpenSpec`.

This is judgment, not a score. State the classification briefly when it affects
the workflow; do not add ceremony to routine Direct work.

## Authority and state

- An agent may recommend OpenSpec and briefly explain why.
- Do not create a new change until the user explicitly authorizes it.
- An explicit request or `/opsx` proposal command is authorization; do not ask
  for redundant confirmation.
- If the user names an existing change, use it normally and do not reclassify it
  or create another change.
- Exploration, brainstorming, architecture discussion, code review, diagnosis,
  prototyping, documentation, maintenance, estimation, and Issue planning never
  create a change automatically.
- Do not create or modify Issues, Project items, milestones, or their fields
  without an explicit request.

The GitHub Project is the operational source for priority, status, roadmap,
milestone, area, work type, and the conceptual field
`Planning = Direct | Issue | OpenSpec`. OpenSpec does not duplicate that board.

## Intent routing

- **Explore:** investigate code and alternatives, identify risks, prototype only
  when requested, and recommend a planning level. Do not create a change.
- **Implement:** handle Direct work, an Issue contract, or an existing OpenSpec
  change and run proportional validation. If unexpected structural impact would
  change important contracts, stop before that change, explain it, and recommend
  OpenSpec.
- **Spec:** create or refine proposal, specs, design, tasks, sync, validation,
  and archive only after explicit OpenSpec authorization or for an existing
  change.
- **Review:** compare implementation with the request, Issue, or OpenSpec;
  prioritize regressions, compatibility, tests, and docs. Findings do not create
  changes automatically.
- **Release:** check readiness, versions, changelog, examples, consumers,
  compatibility, and packaging. Operational release fixes do not create changes
  automatically.

## OpenSpec storage

- `openspec/specs/`: consolidated current contracts.
- `openspec/changes/`: active structural work.
- `openspec/changes/archive/`: operational history.

Do not move, compact, or rewrite these areas merely to classify work.

## Generated agent integrations

`openspec update` regenerates the `openspec-*` skills under `.codex/skills/`,
`.claude/skills/`, and `.opencode/skills/` from the globally installed OpenSpec
package. It can also manage `opsx` commands according to the global delivery
setting. Treat those files as generated and recheck their Nive-specific entry
guards after an OpenSpec upgrade or forced update.

The repository instructions in `AGENTS.md`, this document, and the custom
`nive-development-workflow` skill are the local policy layer. OpenSpec's updater
does not manage the custom skill name, so it is the stable guard when generated
workflow files change.

Codex, Claude, and OpenCode intentionally have no user-global OpenSpec agent
integrations. Their OpenSpec skills and commands live only in this repository.
The globally installed OpenSpec CLI and its configuration remain as technical
support for those local integrations; `delivery: skills` prevents global Codex
`opsx` prompts from being regenerated. Do not install user-global agent files,
change global delivery, or run a forced update without reviewing its effects on
every configured agent.

## Reference scenarios

| Request | Expected routing |
|---|---|
| Fix a missing-template error message | Direct; implement without a change |
| Investigate why a popup loses focus | Explore first; no change |
| Add icon preset selection to `nive new` | Issue by default |
| Redesign the public application lifecycle | Recommend OpenSpec and wait for authorization |
| Create an OpenSpec change for declarative commands | Create it without another confirmation |
| Implement the next task of a named change | Use that change; create no replacement |
| Review a PR for regressions | Review directly; classify follow-up findings separately |
