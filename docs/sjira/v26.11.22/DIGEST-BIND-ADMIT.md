# CASTLE digest-bind admit (ggen `AotSealReceipt` → mint)

| Field | Value |
|-------|-------|
| Subject | Digest-bind admit surface for Foundry crown wire (ggen layout/brand digest → CASTLE mint) |
| Castle tip base | `2774039ea81bad508c93785a59dd7a99198ae256` |
| Sidebranch | `cursor/digest-bind-admit-ca8a` |
| Standing | Admit API **PRESENT**; production wire bind **WIRED** (optional pin); Foundry/zero-HITL crown still **UNKNOWN**/orthogonal |
| Honesty | Wire bind ≠ Foundry crown ALIVE; sibling redaction/brand courts ALIVE ≠ digest-bound spine without a pin |

## Plug-in map

| Stage | File | Hook |
|-------|------|------|
| Admit / bind API | `src/publish/digest_bind.rs` | `admit_ggen_layout_digest` / `bind_into_mint` / `assert_mint_binds_digest` |
| Module export | `src/publish/mod.rs` | `pub mod digest_bind` + re-exports |
| Wire seal | `src/publish/wire.rs` → `seal_public_artifact` | After `RedactionCourt::admit`, optional pin → `bind_into_mint` |
| Branded mint | `publish_branded_*` / `publish_subject_post` / `publish_target` | Thread `Option<&GgenLayoutDigestPin>` / CLI hex |
| Brand twin | `brands/fourth-watch.brand.toml` | Omits `extensions.ggen.*` (intentional slim twin); pin via CLI/API |
| CLI flag | `src/bin/castle/verbs/castle_261120_*.rs` | `--ggen-layout-digest <64-hex>` |
| Chicago court | `tests/publish_digest_bind_court.rs` | All green when pin threaded |

## Exact next verifier

```text
cd /path/to/castle   # branch cursor/digest-bind-admit-ca8a
RUSTUP_TOOLCHAIN=nightly cargo test --lib publish::digest_bind -- --nocapture
# expect exit 0

RUSTUP_TOOLCHAIN=nightly cargo test --test publish_digest_bind_court -- --nocapture
# expect all PASS (including mint_receipt_must_bind_…)

cargo test --test publish_redaction_court   # expect 13/13 exit 0
cargo test --test cli_publish_brand         # expect 6/6 exit 0

PIN=$(python3 -c 'print("ab"*16 + "cd"*16)')
./target/debug/castle publish post \
  --subject spine-probe \
  --public-anonymize \
  --brand brands/fourth-watch.brand.toml \
  --ggen-layout-digest "$PIN"
# expect exit 0; redaction_court=ALIVE; ggen_layout_digest == PIN; digest_bind.crown_wire=WIRED
```

Wire **WIRED** on CASTLE does **not** promote Foundry / zero-HITL crown to `ALIVE`.
LinkedIn send remains **BLOCKED** on cos `/readyz` **200** ∧ 2× durable LOW (orthogonal).

## Non-goals / antifalsifiers

- Do **not** promote Foundry / zero-HITL `ALIVE` from CASTLE wire bind alone.
- Do **not** treat sibling `publish_redaction_court` / `cli_publish_brand` ALIVE as spine-crown ALIVE without a bound digest on the mint.
- LinkedIn send remains **BLOCKED** on cos `/readyz` **200** ∧ 2× durable LOW (orthogonal).
