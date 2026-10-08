# Generated reference — castle code surface

<!-- GENERATED — DO NOT EDIT BY HAND. Everything in this directory is a
     projection of the castle code surface via ggen-marketplace
     rust-doc-hdit-pack (`doc-hdit scaffold`). -->

Regenerate with:

```sh
python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py code /Users/sac/castle \
  > /tmp/hdit/castle.code.v3.json
/Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit scaffold \
  --code /tmp/hdit/castle.code.v3.json \
  --templates /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/templates \
  --out /Users/sac/castle/docs/reference/generated
```

Contents:

- `reference.md` — per-module item tables (AGENT-FORBIDDEN rigid rows rendered
  from the code surface)
- `how_to.md` — task skeletons (commentary only inside AGENT-COMMENTARY fences)
- `explanation.md` — concept skeleton (same fence rule)

Re-scaffolding merges existing AGENT-COMMENTARY content by marker position;
edits outside the fences are lost on regenerate.

## See Also

- `packs/rust-doc-hdit-pack` in `~/ggen-marketplace` — pack, court, templates
- `docs/reference/generated/reference.md` — the rigid reference body
