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
