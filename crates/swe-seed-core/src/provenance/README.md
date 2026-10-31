# provenance

## Overview

Provenance and clean-room boundary records (spec 0009). Fail-closed: every
active capability record needs a well-formed `source_hash` and `license_tag`,
`code_copied` must be `false`, and the license status must be `clear`.
Verification runs per record, per directory, and against the seed manifest's
capability list. Also supplies `content_hash`, the crate-wide
`sha256:<hex>` digest helper used by eval, gateway, federation, and fabricator.

## Interface

- `content_hash(bytes: &[u8]) -> String`: `sha256:<lowercase hex>` digest
  [hash.rs:content_hash]
- `pub struct ProvenanceRecord { capability_id, source_id, source_kind,
source_location, source_ref, source_hash, license_tag, license_status,
inspected_at, scan_ref, code_copied, notes }`; `license_status` vocabulary is
  `clear` | `ambiguous` | `restricted`; stored at
  `.swe-seed/provenance/<capability-id>.json` [record.rs:ProvenanceRecord]
- `verify_record(r: &ProvenanceRecord) -> Result<(), ProvenanceProblem>`:
  fail-closed single-record verification [verify.rs:verify_record]
- `verify_dir(dir) -> Result<(Vec<ProvenanceRecord>, Vec<(String,
ProvenanceProblem)>)>`: reads and verifies every `*.json` record under a
  directory, sorted [verify.rs:verify_dir]
- `verify_manifest_records(dir, manifest)`: additionally flags every manifest
  capability that has no provenance record
  [verify.rs:verify_manifest_records]
- `pub enum ProvenanceProblem { MissingRecord, MissingSourceHash,
MalformedSourceHash, MissingLicenseTag, CodeCopied,
LicenseNotClear { status } }` with `message()`
  [verify.rs:ProvenanceProblem], [verify.rs:message]

## Invariants

- Every active capability record needs `source_hash` + `license_tag`, and
  `code_copied` must be `false` (clean-room boundary, spec 0009)
  [verify.rs:verify_record]
- `source_hash` must be `sha256:` followed by one or more hex digits, matching
  the contract produced by `content_hash` [verify.rs:is_valid_source_hash]
- A non-`clear` `license_status` is a problem requiring human approval; it is
  never auto-accepted [verify.rs:verify_record]
- Fail-closed on filesystem errors: an unreadable directory or file is
  propagated as an error, never masked as an empty record set; a MISSING
  directory is the one intentional exception (zero records, vacuously ok)
  [verify.rs:verify_dir]
- Every capability in the seed manifest must have a provenance record;
  otherwise `MissingRecord` is reported [verify.rs:verify_manifest_records]

## Side Effects

- `verify_dir` reads the directory and every `*.json` record file via
  `std::fs::read_dir` / `std::fs::read` [verify.rs:verify_dir]
- `content_hash` is pure computation (sha2) with no I/O [hash.rs:content_hash]

## Verification

- Run: `cargo test -p swe-seed-core`
