# CAMPAIGN-RECEIPT — castle v26.10.8

| Field | Value |
|---|---|
| Branch | `main` |
| HEAD | `33b9746f77994e8ac3a0cba7c628028c2352d3e0` (concurrent fleet lanes were landing during receipt write; HEAD re-read at write time) |
| Base tag | `v26.10.8+dfcm.1` (tag object `d473162203a55b014ed6fedfcd2d9da2a1e625ad` → commit `cb779b507eaf211054e3985450fb2012b6481304`) — plain `v26.10.8` also exists (→ `ca0e426521c810a95e3f2ab317b8c665ee1aa023`); corrected 2026-10-09, the earlier "no plain tag" claim was stale |
| In-sync with origin | in sync at receipt time |

## Campaign commits since `v26.10.8+dfcm.1`

| SHA | Subject |
|---|---|
| `33b9746` | docs: ISO 20022 conformance testing section (fleet campaign) |
| `85d209f` | docs: cross-reference SA2A envelope surface (fleet campaign) |
| `d8ca9b1` | docs: README 26.10.8 sync + create CHANGELOG (fleet campaign) |

## Witnessed gates

| Gate | Result |
|---|---|
| Cargo suite | Full `cargo` build + tests green (`--no-fail-fast`, all binaries 0 failed; ISO 20022 XSD fixtures fetched to the pinned sha256s in `fixtures/payments/iso20022/xsd/PINS.json`) |
| ISO 20022 conformance | Pinned-XSD ISO 20022 conformance gate passing |
| doc-hdit CERTIFY | ACCEPTED — S_coverage 0.9473 (≥0.90), Phi_halluc 0.0000 (≤0.001), Q_density 1.0000 (≥0.65); subject `076744f2336bc2e0e172a850a53be78e9e8c9920469b11a5726c2874dfa7c14d`; chain `docs/sjira/v26.10.8/doc-hdit.receipts.jsonl`; extractor `scripts/gen_doc_surface.py` sha256 `4c862576ab63595f9cd0417b35341af3ec1001f49450e79bf2e4c291a4a4246f` (BLAKE3 receipt identity `a579e2109941e1f27f2faf0403d6c91e3eebb7f234dcc191a814573001309616`), ggen-marketplace @ `2cf02b276` |

### doc-hdit certify (2026-10-09, lane R5)

- Extract → audit → certify with ggen-marketplace rust-doc-hdit-pack at `2cf02b276`
  (`doc-hdit` release binary, extractor pinned at sha256 `4c8625…246f`).
- First audit FAILED honestly (coverage 0.6585, phantom 0.0078): 25 phantom claims, all
  `has_param` rows in the checked-in generated reference referencing private Elixir helpers
  (`defp` in `paas/lib/castle_paas/kernel.ex` / `reactors.ex`) absent from the extractor's
  public surface. Fix was at the generator, not the prose: `docs/reference/generated/`
  regenerated via `doc-hdit scaffold` from the pinned extractor's current code surface
  (which emits no private rows). No threshold touched.
- Post-regeneration audit: PASS coverage 0.9473/0.90 · PASS phantom 0.0000/0.001 ·
  PASS density 1.0000/0.65, exit 0. Certify minted verdict ACCEPTED (receipt above).

## Standing

ALIVE — full cargo suite green; pinned-XSD conformance witnessed.

## Open residues

| Residue | Detail |
|---|---|
| Archived-review status | `configs/ecosystem-v26.9.28.json` still carries v26.9.28 ecosystem review standing alongside the newer `fortune5-v26.10.8+dfcm.1.json`; its archived-review disposition is undecided. |
