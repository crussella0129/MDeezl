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
    `git -c credential.helper= -c core.askPass= -c core.autocrlf=false -c core.eol=lf clone --depth 1 --quiet -- <url> <dest>`.
    It runs with:
    - `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`,
      `GIT_ALLOW_PROTOCOL=https:http:file` and `GIT_LFS_SKIP_SMUDGE=1` set;
    - `GIT_ASKPASS`, `SSH_ASKPASS` and the repository-local variables removed.

    Why each part is needed:
    - **Terminal prompts are only one of git's four prompt routes.** The
      other three are `GIT_ASKPASS`, `core.askPass` and `SSH_ASKPASS`.
    - **An `insteadOf` rule can rewrite an https URL to ssh.** The allow-list
      turns that into a fast failure: "transport 'ssh' not allowed", in 122 ms.
    - **No console window.** On Windows, git is spawned with
      `CREATE_NO_WINDOW`, so a windowed caller never flashes a console.
    - **Same line endings on every OS.** `core.autocrlf=false` alone is not
      enough. Measured: a `* text=auto` repository still checked out CRLF on
      Windows. Adding `core.eol=lf` checks text files out with LF everywhere,
      and a file with an explicit `eol` attribute still follows it.
    - **No LFS downloads.** `GIT_LFS_SKIP_SMUDGE=1` leaves LFS-tracked files as
      their small pointer text. A host's git-lfs setup therefore cannot pull
      large objects into a text bundle, or fail the clone on an exhausted quota.
  - **What is shown differs from what is fetched.** The heading, error lines
    and GUI status show the URL with any `user[:password]@` removed, so a token
    in a URL is never copied into a document meant for pasting. The clone uses
    the URL as given, which means a URL that carries its own credentials can
    clone a private repository. MDeezl itself never prompts for credentials and
    uses no credential helper.
  - **The repository name is made safe.** The name comes from the URL's last
    path segment. `.`, `..`, or anything containing `\`, `:`, `?` or `#`,
    becomes `repository`, so the name can never leave its folder when it is
    used as a path.
  - **A failure names its cause.** The reason is git's first `fatal:` line.
    Measured: for a missing repository that line is
    "… does not appear to be a git repository", while the last line is generic
    advice.
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
    heading is `# <display-url>` (see below), and the tree root is the
    repository's name. That uses
    the existing heading line rather than adding any new element to the
    format, so local-path output stays byte-identical.

## Consequences

- Whichever mechanism is chosen, this intent ends MDeezl's status as a pure
  local-filesystem tool and introduces failure modes (missing `git`, network,
  rate limits) that INT-0001 does not have.
- Deferring it keeps sprint 0 small; the cost is that branch-scoped bundling is
  unavailable until it is scheduled.
- An interrupted CLI run, such as Ctrl+C during a clone, can leave its
  temporary clone in the system temp folder. The GUI waits for its worker when
  the window closes, so the guard still runs there.
- LFS-tracked files appear in a URL bundle as pointer text, not content.

## Transition history
- 2026-09-17: created as `proposed`; explicitly out of scope for sprint 0.
- 2026-09-19: amended while `proposed`, by sprint 3 research. The desktop GUI
  of [INT-0006](INT-0006-desktop-gui.md) takes a path *or URL*, so the remote
  half is scheduled ahead of the git-ref half, which backlog T-103 keeps. The
  measured clone mechanism was added under Alternatives. Two criteria were
  added: no waiting on a person, and removal of the temporary copy.
- 2026-09-19: amended while `proposed`, at sprint 3 planning. Two decisions
  were added: `file://` counts as a URL, and a URL source's heading is its URL.
- 2026-09-19: `proposed → planned` for sprint 3. Work evidence: T-002 delivers
  the remote half. The git-ref half and its criterion stay in backlog T-103.
- 2026-09-19: amended while `planned`, from the sprint 3 plan critique. The
  fetch was hardened against the other prompt routes (askpass, and ssh
  reached through an `insteadOf` rewrite), keeps committed bytes with
  `core.autocrlf=false`, and spawns git without a console window on Windows.
  All of it was measured before being recorded.
- 2026-09-19: correction to the entry "amended while `proposed`, at sprint 3
  planning". That amendment was in fact made after the move to `planned`. A
  later revision rewrote the entry in place; it has been restored here, as the
  schema forbids erasing entries, and this note corrects it instead.
- 2026-09-19: amended while `planned`, from the second sprint 3 plan critique,
  which returned `proceed-with-caveats`. Each item was measured where
  measurable:
  - the clone adds `core.eol=lf` and `GIT_LFS_SKIP_SMUDGE=1`;
  - a shown URL drops its userinfo;
  - the repository name is made path-safe;
  - a failure's reason is git's first `fatal:` line.

  Two Consequences were added: Ctrl+C in the CLI can leave a temporary clone,
  and LFS files appear as pointers.
- 2026-09-19: amended while `planned`, from the third sprint 3 plan critique.
  - The Alternatives now state the full command, with `core.eol=lf` and
    `GIT_LFS_SKIP_SMUDGE=1`.
  - The "committed bytes are kept" bullet was removed, because measurement
    disproved it. This supersedes the round-1 entry above, which said the clone
    "keeps committed bytes with `core.autocrlf=false`". That entry stays as
    written.
  - The heading is written as `# <display-url>`.
  - It is now recorded that a URL carrying its own credentials can clone a
    private repository, since MDeezl only guarantees that it never prompts.
