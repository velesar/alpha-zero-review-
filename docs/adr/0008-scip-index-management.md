# ADR-0008: SCIP Index Management

**Status:** Accepted
**Date:** 2026-01-24
**Deciders:** Architecture Review

> Written retroactively: the decision was implemented in a6198cc
> ("feat: add SCIP index management to setup-cli and codegraph-server")
> and referenced from ARCHITECTURE_OVERVIEW.md before this record existed.

## Context

codegraph-server originally only offered `load_index(scip_path)`: the agent
had to know where an index was, which language it covered, and whether it was
still current. In practice audits target polyglot repositories, indexes are
expensive to build (tens of seconds to minutes), and a stale index silently
produces wrong caller/impact data.

## Decision

1. **Fixed location and naming.** Indexes live in `<project>/.audit/indexes/`
   as `<language>.scip` (`rust.scip`, `python.scip`, `typescript.scip`, ...).
2. **Freshness metadata.** Each index has a sibling `<language>.scip.meta`
   holding the short git commit it was built from. An index is fresh when that
   commit equals the project's current `HEAD`.
3. **Build at setup time.** `setup-audit --with-index` detects languages and
   runs the matching indexer (`rust-analyzer scip`, `scip-python`,
   `scip-typescript`, `scip-go`, `scip-java`); `--install-indexers` installs
   missing ones.
4. **Auto-load at audit time.** A new tool,
   `load_project_indexes(project_path, build_if_missing?)`, discovers every
   index in `.audit/indexes/`, reports missing/stale ones as warnings, and can
   build missing indexes on demand when the indexer is installed.
5. **One queryable graph.** All loaded indexes are merged into a single
   in-memory graph so every codegraph tool sees every language. SCIP symbol
   IDs include the indexer scheme and package, so they do not collide.
6. `load_index` remains for loading an explicit file (SCIP or JSON).

## Consequences

**Positive**
- Viewpoint skills can call `load_project_indexes` without knowing languages
  or paths.
- Stale indexes are detected rather than silently used.
- Index builds happen once per commit, not once per session.

**Negative**
- Freshness is per commit: uncommitted changes are not reflected in a "fresh"
  index.
- On-demand builds can make the first `load_project_indexes` call slow.
- Indexer coverage varies: some indexers do not emit `enclosing_range`, in
  which case `get_callees` approximates a symbol's body as the span up to the
  next function/type definition.

## Related

- ADR-0003: Workspace-Based Modular Architecture
- `codegraph-server/src/index_manager.rs`, `setup-cli/src/indexer.rs`
