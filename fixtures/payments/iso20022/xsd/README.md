# ISO 20022 XSD pins

XSDs are third-party mirrors of the ISO 20022 message archive. Redistribution terms are unclear, so
only url and sha256 are committed (see `PINS.json`). Set `CASTLE_ISO20022_XSD_DIR` to a directory
containing `pain.001.001.09.xsd` and `pacs.008.001.08.xsd`; the official-conformance test verifies
the sha256 and fails loudly if the directory or files are missing or altered.

## See Also

`tests/payments_iso20022_official.rs`
