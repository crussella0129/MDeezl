# INT-0003 — LLM-generated scaffold comments

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0003
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Optionally annotate each line of the scaffold tree with a short trailing
`# comment` describing what that file or directory is for, as sketched in
`Scaffolding symbols generator.md`:

```
Kinesin/
├── .github/        # CI configuration
│   └── workflows/  # build and test pipelines
└── README.md       # project overview
```

**Non-goals:** generating comments by default, embedding an API key in the
tool's configuration, or blocking output when annotation is unavailable.

## Acceptance criteria

- Annotation is opt-in; the default output is byte-identical to
  [INT-0001](INT-0001-markdown-repo-context-bundle.md)'s.
- Comments are column-aligned within a directory level so the tree stays
  readable.
- When annotation is requested but the provider is unreachable or unauthorized,
  MDeezl emits the unannotated tree and reports the degradation on stderr.
- No file content leaves the machine unless the user has explicitly enabled
  annotation for that run.

## Rationale

The tree tells an agent what exists; a one-line purpose per entry tells it where
to look first. It is separated from INT-0001 because it is the only part of the
original concept that requires an external model call, which brings
configuration, cost, privacy, and non-determinism that the base tool must not
inherit.

## Alternatives

- **Heuristic comments from filename conventions.** Cheap, offline, and
  deterministic, but shallow; worth evaluating as a fallback tier.
- **Read a `.mdeezl-comments` file maintained by hand.** Deterministic and
  private, but it is documentation the user must maintain.
- **Always annotate.** Rejected: it would make every run non-deterministic and
  network-dependent.

## Consequences

- Introduces the first configuration surface (endpoint, model, credentials) and
  the first privacy-relevant behaviour, since annotation implies sending file
  names — and possibly excerpts — to a third party.
- Annotated output is not reproducible, so tests must exercise the unannotated
  path as the contract.

## Transition history
- 2026-09-17: created as `proposed`; explicitly out of scope for sprint 0.
