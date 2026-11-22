# `castle publish`

Deterministic public mint for LinkedIn carousel / post / newsletter and gatekeeper packs.
**Typst/JSON mint is ALIVE; live LinkedIn send is stop-worked** until the Jobs AND-gate unlocks.

Module map: `anonymize` (refuse court) · `brand` (manifest + honesty floors) · `digest_bind` (ggen `AotSealReceipt` layout digest → mint; wire bind **WIRED** when `--ggen-layout-digest` / pin supplied — Foundry/zero-HITL crown still orthogonal; see `docs/sjira/v26.11.22/DIGEST-BIND-ADMIT.md`) · `wire` (orchestration) · `linkedin` · `typst_pdf` (optional host `typst`; missing binary → `pdf.standing = gap`).

## Mandatory CLI gates

Public targets require **both**:

1. **`--public-anonymize`** — omit → `REFUSED_NO_PUBLIC_ANONYMIZE` / `REFUSED_NO_AUTHORITY` (exit 2).
2. **`--brand <path>`** — explicit path only (e.g. `brands/fourth-watch.brand.toml`). Omit / empty → `REFUSED_BRAND` (**no soft default**). Bad path → `REFUSED_BRAND` unreadable.

```sh
./target/debug/castle publish post \
  --subject t \
  --public-anonymize \
  --brand brands/fourth-watch.brand.toml
```

Courts: F-ANON-1..4 + F-BRAND-1 (`docs/sjira/v26.11.20/falsifier-checklist.md`).

## Refuse-only `RedactionCourt`

Production seal is [`RedactionCourt::admit`](anonymize.rs) via `wire::seal_public_artifact`.

- **Law:** refuse private residue — **never strip-and-ship**.
- Findings never echo the raw private string (`[REDACTED]` / `[REDACTED_CITE]` only).
- Brand floors (top-level `[redaction_court]`): private client names, home paths, bearer residuals, calendar links, emails, external links in post body (when policy says so).
- ALIVE receipt records `"collaborator": "RedactionCourt"`.

There is no production redact-and-continue surface.

## Digest-bind (optional pin → mint `ggen_layout_digest`)

API: `digest_bind::{admit_ggen_layout_digest,bind_into_mint,assert_mint_binds_digest}`.
Wire: after `RedactionCourt::admit`, `wire::seal_public_artifact` calls `bind_into_mint`
when a pin is threaded. CLI: `--ggen-layout-digest <64-hex>` (AotSealReceipt-shaped).
Omit pin → sibling redaction/brand courts stay green without the key.
Chicago court: `cargo test --test publish_digest_bind_court` — all PASS when wired.
Wire **WIRED** ≠ Foundry / zero-HITL crown `ALIVE`.

## F-ANON-3 — Featured CTA in `first_comment` only

- Post **body** stays scheme-free (no `https://` / `http://` / `www.`); Featured host honesty may appear as prose (`cos.fnox.io`).
- Absolute Featured URL ships only in **`first_comment`** from `messaging.cta.featured_url_template` via `cta_link`.
- Leaky subjects under `--public-anonymize --brand …` refuse at the court (exit 2) — they are not scrubbed through.

## AND-gate blast unlock (not shipping)

Jobs primary outbound unlock is an **AND-gate**, recorded on the brand twin:

| Field | Meaning |
|---|---|
| `extensions.honesty.blast_unlock_primary` | `/readyz` **200** ∧ 2× durable LOW-only — not `/readyz` alone |
| `extensions.honesty.low_durable_status` | required second gate; status only — not unlock by itself |
| `messaging.cta.drop_gated_note` / `required_cta` | must surface SlotLease `/readyz` ∧ 2× LOW |

**Today:** mint yes · LinkedIn send **no** until live cos `/readyz` 200 **and** 2× durable LOW receipts exist on disk.

## FFB / package honesty (out of publish, in release voice)

Do **not** claim published `@fnox/*` GA from this surface. Frontend-for-Backend delivery in the release fence is **on-disk `repop` / `compat` shims only** — see `docs/releases/v26.11.20-PRFAQ.md`.

## Intake host

Host of record remains **`cos.fnox.io`** (`/drop/:lease_id`). Never invent `revops.fnox.io`.
