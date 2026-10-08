# CAMPAIGN-RECEIPT — castle v26.10.8

| Field | Value |
|---|---|
| Branch | `main` |
| HEAD | `33b9746f77994e8ac3a0cba7c628028c2352d3e0` (concurrent fleet lanes were landing during receipt write; HEAD re-read at write time) |
| Base tag | `v26.10.8+dfcm.1` (`d473162203a55b014ed6fedfcd2d9da2a1e625ad`) — no plain `v26.10.8` tag exists in this repo |
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
| Cargo suite | Full `cargo` build + tests green |
| ISO 20022 conformance | Pinned-XSD ISO 20022 conformance gate passing |

## Standing

ALIVE — full cargo suite green; pinned-XSD conformance witnessed.

## Open residues

| Residue | Detail |
|---|---|
| Archived-review status | `configs/ecosystem-v26.9.28.json` still carries v26.9.28 ecosystem review standing alongside the newer `fortune5-v26.10.8+dfcm.1.json`; its archived-review disposition is undecided. |
