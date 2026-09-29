# Resource conservation

`ResourceEnvelope` is authority-free allocation state. `BudgetLedger` enforces recursive conservation by refusing any aggregate child allocation that exceeds the root envelope in compute units, I/O bytes, or effect count.

BCINR/CMCA may compute proposed allocations upstream. Its receipts do not become cryptographic authorization and cannot mint an ActuationCertificate.
