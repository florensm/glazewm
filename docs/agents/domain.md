# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`docs/agents/GLOSSARY.md`**: this repo keeps its glossary here, not at the repo root.
- **`docs/agents/adr/`**: read ADRs that touch the area you're about to work in.
- **The animation glossary** at the top of `packages/wm/src/animation/mod.rs` defines peek, stand-in, surrogate, hand-back and companion. Use those terms; don't redefine them in `GLOSSARY.md`.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`) creates them lazily when terms or decisions actually get resolved.

## File structure

Single-context. Everything the agent skills read or write lives under `docs/agents/`, kept apart from the feature docs in `docs/features/`:

```
docs/
├── features/          ← feature docs for people
└── agents/
    ├── issue-tracker.md
    ├── triage-labels.md
    ├── domain.md
    ├── GLOSSARY.md    ← created lazily
    └── adr/           ← created lazily
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `docs/agents/GLOSSARY.md` (or the animation glossary). Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders), but worth reopening because…_
