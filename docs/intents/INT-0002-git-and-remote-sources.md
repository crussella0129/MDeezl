# INT-0002 — Git and remote sources

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0002
- **State:** planned
- **Work evidence:** [Sprint 3 build plan](../sprints/s3/sprint-plans/build-plan.md), [T-002 URL sources](../sprints/s3/sprint-plans/build-plan.md#t-002-url-sources)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Let MDeezl bundle something other than the working tree on disk: a named git
branch, a worktree, a commit, or a repository hosted on GitHub, GitLab, Gitea,
or Codeberg, as `Scaffolding symbols generator.md` describes.

**Non-goals:** authentication flows for private repositories, cloning as a
side effect the user did not ask for, and any change to the output format
itself, which remains [INT-0001](INT-0001-markdown-repo-context-bundle.md)'s.

## Acceptance criteria

- A source other than the working tree can be named on the command line, and the
  emitted document records which source it came from.
- Reading a git ref does not require a dirty-tree stash or a checkout that
  mutates the user's working directory.
- A remote provider is addressed by URL rather than by a per-provider flag
  wherever the provider's plain HTTPS interface allows it.
- Network and authentication failures produce a diagnostic on stderr and a
  non-zero exit, never a partial document on stdout.
- Fetching a remote never waits on a person. Credential prompts and credential
  helpers are disabled for the fetch, so a private or missing repository fails
  within seconds instead of hanging on a login dialog.
- Whatever is fetched to a temporary location is removed afterwards, whether
  the run succeeded or failed.

## Rationale

Bundling a branch you are not currently on, or a repository you have not cloned,
is the difference between a local convenience and a tool an agent can point at
anything. It is separated from INT-0001 because it introduces process execution
or network I/O, either of which would compromise INT-0001's zero-dependency,
pure-`std` shape if entangled with it.

## Alternatives

- **Shell out to `git archive` / `git ls-tree`.** Plausible and dependency-free,
  at the cost of requiring `git` on PATH.
- **Link a git library.** Would break the zero-dependency property that
  INT-0001 accepted deliberately, so it requires an explicit decision.
- **Require the user to clone first and point MDeezl at the checkout.** This is
  the current behaviour and the reason this intent exists.
- **For remotes: a shallow `git clone` into a temporary directory.** Selected
  at the sprint 3 boundary, for the remote half.
  - **It stays dependency-free.** Rust's standard library has no HTTPS client,
    so the alternative is linking a TLS stack.
  - **It works for every provider's plain HTTPS clone URL.**
  - **The fetch is hardened so it can never wait on a person.** The measured
    command is
    `git -c credential.helper= -c core.askPass= -c core.autocrlf=false clone --depth 1 --quiet -- <url> <dest>`.
    It runs with:
    - `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never` and
      `GIT_ALLOW_PROTOCOL=https:http:file` set;
    - `GIT_ASKPASS`, `SSH_ASKPASS` and the repository-local variables removed.

    Why each part is needed:
    - **Terminal prompts are only one of git's four prompt routes.** The
      other three are `GIT_ASKPASS`, `core.askPass` and `SSH_ASKPASS`.
    - **An `insteadOf` rule can rewrite an https URL to ssh.** The allow-list
      turns that into a fast failure: "transport 'ssh' not allowed", in 122 ms.
    - **The committed bytes are kept.** `core.autocrlf=false` stops a Windows
      system config from converting line endings, so the bundle holds the
      repository's content as committed, the same on every OS.
    - **No console window.** On Windows, git is spawned with
      `CREATE_NO_WINDOW`, so a windowed caller never flashes a console.
  - **It was measured with git 2.54:** a one-commit public repository cloned
    in 0.9 s.
  - **Failures are fast and clean.** With `GIT_TERMINAL_PROMPT=0` and the
    credential helper disabled, a missing repository failed in 0.4 s (exit
    128, "terminal prompts disabled") and left no directory behind.
  - **Only `https://`, `http://` and `file://` count as URLs.** Anything else
    is a path. Git itself read `ftp:/weird` as an ssh address, so the tool must
    decide what a URL is. `file://` is a URL git clones, and accepting it lets
    the whole remote path be tested offline.
  - **The heading records the source.** For a URL source, the document's
    heading is `# <url>`, and the tree root is the repository's name. That uses
    the existing heading line rather than adding any new element to the
    format, so local-path output stays byte-identical.

## Consequences

- Whichever mechanism is chosen, this intent ends MDeezl's status as a pure
  local-filesystem tool and introduces failure modes (missing `git`, network,
  rate limits) that INT-0001 does not have.
- Deferring it keeps sprint 0 small; the cost is that branch-scoped bundling is
  unavailable until it is scheduled.

## Transition history
- 2026-09-17: created as `proposed`; explicitly out of scope for sprint 0.
- 2026-09-19: amended while `proposed`, by sprint 3 research. The desktop GUI
  of [INT-0006](INT-0006-desktop-gui.md) takes a path *or URL*, so the remote
  half is scheduled ahead of the git-ref half, which backlog T-103 keeps. The
  measured clone mechanism was added under Alternatives. Two criteria were
  added: no waiting on a person, and removal of the temporary copy.
- 2026-09-19: `proposed → planned` for sprint 3. Work evidence: T-002 delivers
  the remote half. The git-ref half and its criterion stay in backlog T-103.
- 2026-09-19: amended while `planned`, at sprint 3 planning. Two decisions
  were added: `file://` counts as a URL, and a URL source's heading is its URL.
- 2026-09-19: amended while `planned`, from the sprint 3 plan critique. The
  fetch was hardened against the other prompt routes (askpass, and ssh
  reached through an `insteadOf` rewrite), keeps committed bytes with
  `core.autocrlf=false`, and spawns git without a console window on Windows.
  All of it was measured before being recorded.
