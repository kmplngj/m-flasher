> Archived 2026-10-16 from the local upstream-preparation handoff. Historical statements and test/review counts below are preserved reports, not checks rerun for this archive. Sanitization and current-state verification are documented in [README.md](README.md).

# Draft discussion: an optional minimal RAM-reader path

Local discussion draft for `jvitkauskas/m-flasher`; not published.

Would you be interested in a separately integrated, minimal read-only RAM reader
for standalone full raw backups? The user goal is to preserve the unit's complete
stored bytes before changing firmware, including encrypted firmware, configuration
and musical data. An optional machine-readable lifecycle could later help an
operator or agent distinguish preparation, acquisition, persistence and return.
It would not authorize autonomous device operations.

The fork implements an experimental FMR1 path alongside the existing loader.
Its pinned 1,456-byte reader exposes identity/chip checks and fixed 256-byte
reads. Its NOR operations are JEDEC identification and raw reads; it contains no
FC0A, write-enable, status-register programming, erase/program operation or
unknown-command fallback. Bootstrap uploads the exact digest-bound image once
and jumps once. RAM execution still changes device mode and can hang.

One selected Stock V15 FM-1 observation on Apple Silicon/macOS 27.0.1 is recorded
against historical host revision `4ce0342`. That session completed two fresh,
equal 1 MiB passes, protected storage with reopening checks, final held-session
reader checks, and a separately observed manual cold return with fresh Stock
identity. Both destinations were on one filesystem; separate physical media were
not established. Later host hardening through `4efefbe` was validated offline.
That newer host source has no new physical qualification from the historical
observation. Neither result establishes other units/platforms, restore or writer
qualification. Device serials, raw-image hashes, samples, detailed receipts and
private filesystem paths are intentionally omitted here.

The proposed integration contract has one explicit selected-session owner spanning
bootstrap, both acquisitions, storage acknowledgement and final continuity checks.
Strict transfer counts, tags, residue, status and deadlines reject ambiguity
without retry, reconnect, reset or loader fallback. Two fresh reads are an
acquisition outcome; protected, reopened copies are a storage outcome; actual
return to the normal application is a separate operator-observed outcome. A
successful capture receipt must not imply that return has occurred.

Before proposing code upstream, could we agree on these ownership/design points?

- Should a new reader/profile remain separate from the existing combined loader,
  and who should own the reader source, artifact pin and ABI evolution?
- Should selected-device native access extend the current backend abstraction or
  use an optional backend, and where should privileged session ownership live?
- Should standalone raw backup be a separate CLI operation, and which explicit
  selection and storage policies belong in its public interface?
- Is a versioned machine-readable status schema wanted now or later, and who
  owns its acquisition/storage/return states and compatibility guarantees?

After agreement, a first small PR could add only the agreed pure reader-profile
contract and independent fake-transport fixtures, with native activation excluded.
Native access, CLI composition and lifecycle output could then be reviewed as
separate increments. The fork's full architecture is offered as discussion
material, not as a single upstream patch.
