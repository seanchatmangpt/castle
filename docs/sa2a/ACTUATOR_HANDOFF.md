# Independent actuator handoff

The next runtime owner consumes the portable schemas in `schemas/sa2a`.

Required order:

1. reconstruct `PreparedEffect` and recompute its portable SHA-256 digest;
2. verify exact principal, audience, policy epoch, revocation epoch, generation, validity window, and certificate signatures;
3. require custodian-distinct k-of-n quorum;
4. claim `(effect_digest, generation)` in an actuator-local durable state machine;
5. execute only a typed effector admitted for the certificate audience;
6. store executed or unknown-outcome state before returning a receipt.

No distributed Erlang trust relationship or ambient control-plane credential is part of this handoff.

## See Also

- `../gymact/docs/reference/reference.md` ("SA2A replan envelope", external
  sibling repo `seanchatmangpt/gymact`) — the consumer side of the portable
  SA2A replan envelope (`src/gymact/sa2a_envelope.py`), including its typed
  refusal codes (`SA2A_EXACT_SUBJECT_REQUIRED`, `SA2A_EXACT_SUBJECT_NONFINITE_NUMBER`,
  `SA2A_EXACT_SUBJECT_MISMATCH`). CASTLE mirrors subject mismatch as
  `REFUSED:SA2A_EXACT_SUBJECT_MISMATCH` in `src/v26_9_28/mod.rs`.
