# SA2A trust domains

Castle's `sa2a_security` module is a verifier, not an authority service and not an actuator.

The control plane may construct a `PreparedEffect`. An external authority domain owns signing keys and may issue an `ActuationCertificate`. An independent actuator domain recomputes the effect digest, verifies the certificate, fences generation and replay, claims the effect in its local durable store, executes one typed effector, and records completion.

Castle carries no protected actuation credentials through this API. Compromise of the control plane therefore does not imply certificate issuance.
