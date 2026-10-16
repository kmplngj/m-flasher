# Agent instructions

## Start here

- Before changing code, read the relevant issue, `docs/safety.md` and the
  applicable section of `docs/fm1-roadmap.md`. Issues are the work record.
- For branches, commits, review, PRs, upstream contributions or releases, read
  `docs/contributing.md` before acting.
- Preserve unrelated work, dependency/submodule pins, upstream firmware and
  license notices. Keep changes scoped to the selected issue.
- Where RTK is available, prefix shell commands with `rtk`; use `rtk proxy`
  for commands without a wrapper. Do not install tools to satisfy this rule.

## Offline development boundary

- Develop using local files, saved fixtures and injected fakes only. Never
  enumerate, open, probe or command MIDI, serial, USB or SCSI devices, even
  for passive inventory or read-only queries. Keep real adapters unavailable.
- Do not run the flasher against devices, install hardware backends, execute
  or emulate target code, upload artifacts, or write flash during development.
- Firmware builds remain excluded except the selected `ram-reader/` offline
  artifact build documented in `ram-reader/README.md`: existing locked tools,
  cached image, no networking or devices, fresh output below `target/`.
  Its host-native fake harnesses and file-only artifact tests are permitted.
- Hardware work needs a separate explicit human request and the selected
  operation's evidence gates. An issue, PR or this file grants no hardware
  authority. The sole owner FM-1 is never used for writer/fault/agent cycles.

## Implementation and evidence

- Add tests around observable behavior and failure boundaries with independent
  fixtures. Before running a check, establish that it stays offline; an
  application build or passing test does not establish device safety.
- Record exact commands, results and material limits. Separate local,
  committed, pushed, merged, offline-validated, hardware-qualified and released
  states; identify the exact artifact for any qualification claim.
- A failed or missing gate stays visible. Do not substitute generic loader,
  chip, platform or external-device evidence for the selected operation.

## Delegation and publication

- Use subagents only when the human request or applicable instructions
  authorize delegation. Then use GPT-6.1 Sol with medium effort for bounded
  easy work and high effort for hard work; give independent reviewers a
  precise scope and require findings tied to evidence.
- AGENTS.md is guidance, not standing permission to publish. Create or modify
  remote issues, PRs, comments, pushes, merges or releases only when the human
  request covers that action. A release requires explicit release authority.
