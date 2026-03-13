# Upstream Tracking and Guided Submodule Updates

## Goal

Make `git-dom` feel less like a thin wrapper over submodule commands and more like a guide that can explain submodule state, detect risky situations, and recommend the next safe action.

This is especially important when:

- a submodule is checked out detached at the recorded gitlink
- the upstream branch moves ahead
- the upstream branch is force-pushed or rebased
- the submodule has dirty or conflicted local state
- the parent repo has an unstaged or staged submodule pointer update

## Current Gaps

The current implementation already discovers submodules, shows local worktree state, and can update submodules from upstream. The rough edges are mainly about state modeling and action guidance.

### 1. `status` conflates checkout state with tracking state

Today `branch` is used for both:

- a real branch name when `HEAD` is attached
- a short commit hash when `HEAD` is detached

That makes detached submodule checkouts look like they are "on" a branch named after a commit hash, which is confusing. It also prevents clear messaging like:

- checkout: detached at `a82e147`
- tracks: `origin/scottatron-tilde-path-expansion`
- parent records: `a82e147`
- upstream tip: `91bc123`

### 2. Upstream tracking is under-modeled

The implementation computes ahead/behind using `origin/<head.shorthand()>`. That works for attached branches, but it does not represent the very common submodule case where the checkout is detached while the submodule is still logically tracking a branch configured in `.gitmodules`.

### 3. Conflict state is not first-class

When a submodule has unresolved conflicts, the important user-facing state is not just "modified content". The tool should explicitly say the submodule is conflicted and block unsafe update operations until that is resolved.

### 4. Force-pushed upstreams are not recognized as a distinct scenario

When a tracked upstream branch is rewritten, the user needs different guidance from a normal fast-forward update. This should be surfaced explicitly as an "upstream rewritten" or "non-fast-forward upstream change" state.

### 5. Parent repo staging is too broad during update

The current `pull` command stages everything in the parent repo with `git add -A`. That can accidentally include unrelated parent-repo changes when the intent was only to stage updated submodule pointers.

### 6. Prompt mode preview happens too early

The current `pull --commit=prompt` flow prints a cached diff before staging, which can make the preview incomplete or empty.

## Better State Model

For each submodule, `git-dom` should track these values separately:

- submodule name
- submodule path
- URL
- configured tracking ref
- configured tracking branch, if any
- parent `HEAD` gitlink OID
- parent index gitlink OID
- submodule worktree `HEAD` OID
- checkout state: attached branch or detached
- fetched upstream OID for the tracked ref
- ahead/behind relative to tracked upstream
- whether upstream changed since last fetch
- whether upstream was rewritten
- whether the submodule worktree is clean, dirty, or conflicted
- counts for staged, modified, untracked, and conflicted paths
- whether the parent repo has a staged or unstaged pointer change

This gives `git-dom` enough context to explain what happened and choose the right recommendation.

## Recommended Command Model

### `git dom track <name> --branch <branch>`

Set the logical upstream for a submodule. For standard branch-tracking cases, write the branch into `.gitmodules`. For more advanced cases, allow:

- `--ref refs/heads/<branch>`
- `--ref refs/tags/<tag>`

This makes the tracking relationship explicit and gives `git-dom` a clear source of truth for future updates.

### `git dom doctor [name] [--fetch]`

Produce an actionable diagnosis instead of a raw status dump.

Example output:

```text
obsidian-agent-client
  checkout: detached at a82e147
  tracks: origin/scottatron-tilde-path-expansion
  parent records: a82e147
  upstream: 91bc123
  state: upstream rewritten
  worktree: clean

Next step:
  git dom update obsidian-agent-client --reset-to-upstream
```

This is the command that should turn confusing git/submodule situations into a clear plan.

### `git dom update [name]`

This should become the main "move the submodule to what it should be tracking" command.

It should choose behavior based on state:

- clean + detached + tracked upstream moved normally:
  - fetch
  - update submodule checkout to tracked upstream
  - stage only the submodule pointer in the parent repo
- clean + upstream rewritten:
  - explain that the tracked branch was force-pushed
  - offer or perform a reset-to-upstream workflow
- dirty worktree:
  - stop and explain available options
- conflicted worktree:
  - stop and require conflict resolution or abort first

Possible explicit flags:

- `--ff-only`
- `--reset-to-upstream`
- `--rebase-local`
- `--stage`
- `--commit`

### `git dom status [name]`

Keep this as the readable snapshot, but upgrade the output to reflect the richer model:

- checkout state
- tracked upstream
- parent recorded gitlink
- local worktree `HEAD`
- parent staged/unstaged pointer updates
- clean / dirty / conflicted classification

## Safer Update Rules

The default update path should be conservative and specific:

1. Fetch the tracked refs.
2. Detect whether the upstream moved normally or was rewritten.
3. Refuse unsafe updates when the submodule has dirty or conflicted state.
4. Stage only affected submodule paths and `.gitmodules` when relevant.
5. Never sweep unrelated parent-repo changes into an update commit.

This is especially important for repos where the superproject often has unrelated local edits alongside submodule work.

## Suggested User Experience for Rewritten Upstreams

When a tracked branch is force-pushed or rebased, `git-dom` should explicitly say so.

Example:

```text
obsidian-agent-client
  checkout: detached at a82e147
  tracks: origin/scottatron-tilde-path-expansion
  upstream: 91bc123
  state: upstream rewritten

Safe actions:
  1. Reset submodule checkout to tracked upstream and stage parent pointer
  2. Inspect rewritten commits
  3. Abort
```

That messaging is much closer to how people actually reason about submodule maintenance than a plain `git submodule update --remote --merge`.

## Implementation Notes

Some likely implementation improvements:

- store checkout state separately from tracking state
- read configured branch/ref from `.gitmodules` or `git config`
- count conflicted entries explicitly
- distinguish parent staged pointer changes from unstaged pointer changes
- treat untracked files as part of a dirty submodule state for summaries
- avoid `git add -A` in parent update flows
- preview actual staged changes in prompt mode

## Phased Roadmap

### Phase 1

- enrich submodule state model
- improve `status`
- add conflict and rewritten-upstream detection

### Phase 2

- introduce `doctor`
- introduce guided `update`
- stage only relevant paths

### Phase 3

- add explicit `track` support for branches and refs
- add machine-readable output for automation, such as `--json`

## Summary

The core opportunity is to make `git-dom` explain submodule state in terms a human can act on:

- what the parent repo records
- what the submodule currently has checked out
- what upstream it is supposed to follow
- whether the next safe action is update, reset, inspect, or resolve conflicts

If `git-dom` gets that model right, it can turn awkward scenarios like detached submodules, rebased upstream branches, and parent-pointer drift into a guided workflow instead of a Git puzzle.
