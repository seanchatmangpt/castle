# CASTLE digest-bind admit (ggen `AotSealReceipt` → mint)

| Field | Value |
|-------|-------|
| Subject | Digest-bind admit surface for Foundry crown wire (ggen layout/brand digest → CASTLE mint) |
| Castle tip base | `2774039ea81bad508c93785a59dd7a99198ae256` |
| Sidebranch | `cursor/digest-bind-admit-ca8a` |
| Standing | Stub/admit API **PRESENT**; production wire bind **ABSENT**; crown wire **UNKNOWN** |
| Honesty | Stub ≠ ALIVE crown wire; sibling redaction/brand courts ALIVE ≠ digest-bound spine |

## Plug-in map (file:line hooks @ base `2774039ea` + this branch)

| Stage | File | Hook |
|-------|------|------|
| Stub admit API | `src/publish/digest_bind.rs` | `admit_ggen_layout_digest` / `bind_into_mint` / `assert_mint_binds_digest` |
| Module export | `src/publish/mod.rs` | `pub mod digest_bind` + re-exports |
| Wire seal (ABSENT call) | `src/publish/wire.rs` → `seal_public_artifact` | Comment + `DIGEST-BIND HOOK` after `RedactionCourt::admit`; **does not** call `bind_into_mint` yet |
| Branded mint return | `src/publish/wire.rs` → `publish_branded_post` / `publish_subject_post` | After `seal_public_artifact`, mint gains `redaction_court` only — no `ggen_layout_digest` |
| Brand twin | `brands/fourth-watch.brand.toml` | Omits `extensions.ggen.*` (intentional slim twin); eventual pin may live here or CLI |
| CLI flag (ABSENT) | `src/bin/castle/verbs/castle_261120_*.rs` | Eventual `--ggen-layout-digest <64-hex>` named by `CLI_FLAG_GGEN_LAYOUT_DIGEST`; clap route not grown |
| Chicago court | `tests/publish_digest_bind_court.rs` | Stub tests green; `mint_receipt_must_bind_admitted_ggen_layout_digest` **red** until wire binds |

## Exact next verifier

```text
cd /path/to/castle   # branch cursor/digest-bind-admit-ca8a (or tip after absorb)
# 1) Stub surface compiles + unit admit courts green:
cargo test --lib publish::digest_bind -- --nocapture
# expect exit 0

# 2) Chicago court — stub rows green; crown falsifier red until wire:
cargo test --test publish_digest_bind_court -- --nocapture
# expect: stub_* PASS; mint_receipt_must_bind_admitted_ggen_layout_digest FAIL
#         (REFUSED_GGEN_DIGEST / mint lacks ggen_layout_digest)

# 3) After wire calls bind_into_mint (signature + CLI pin threaded):
cargo test --test publish_digest_bind_court -- --nocapture
# expect all PASS (including mint_receipt_must_bind_…)

# 4) Replay sibling ALIVE courts (must stay green):
cargo test --test publish_redaction_court   # expect 13/13 exit 0
cargo test --test cli_publish_brand         # expect 6/6 exit 0
./target/debug/castle publish post \
  --subject spine-probe \
  --public-anonymize \
  --brand brands/fourth-watch.brand.toml
# expect exit 0; redaction_court=ALIVE; PLUS ggen_layout_digest asserted
# (once CLI flag exists: add --ggen-layout-digest <64-hex from ggen AotSealReceipt>)
```

Until step 3+4 exit with digest asserted on the mint, **CASTLE on horizontal spine (crown)** stays **UNKNOWN**.

## Non-goals / antifalsifiers

- Do **not** promote Foundry / zero-HITL `ALIVE` from stub land alone.
- Do **not** treat sibling `publish_redaction_court` / `cli_publish_brand` ALIVE as digest-bind ALIVE.
- LinkedIn send remains **BLOCKED** on cos `/readyz` **200** ∧ 2× durable LOW (orthogonal).
