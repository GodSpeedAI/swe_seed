# Manage Federation Keys

> Design: [Federation Signing and Tamper-Evident Traces](../explanations/federation-signing-and-traces.md).

SWE_Seed signs trace-chain roots with an Ed25519 private key. The private key
stays in SWE_Seed (gitignored, SOPS/age-encrypted at rest); the public key is
committed and shared with SEA-Forge. This is the operator workflow.

## 1. Generate a keypair

```bash
swe-seed federation keygen swe-seed-2026q3
```

Writes:
- the **public** key (base64 raw) to `.agent-harness/federation/keys/swe-seed-2026q3.pub` — safe to commit and share with SEA.
- the **private** key (base64 raw) to `.swe-seed/federation/keys/swe-seed-2026q3.key` — gitignored; **never commit**.

## 2. Encrypt the private key at rest

`.sops.yaml` is configured for `.swe-seed/federation/keys/*.key`. **First**
replace the placeholder age recipient with your real SWE_Seed operator key, then:

```bash
just federation-encrypt-key swe-seed-2026q3   # SOPS-encrypts the .key in place
```

The signing loader is SOPS-aware: it tries plaintext base64 first, and falls
back to `sops -d` if the file is encrypted. So you can encrypt at rest and sign
without changing the workflow. To inspect the key:

```bash
just federation-decrypt-key swe-seed-2026q3   # decrypts to stdout
```

## 3. Share the public key with SEA-Forge

Copy the committed `.pub` into SEA's key directory so SEA can verify by key id:

```
SEA: keys/swe-seed-2026q3.pub   ←   SWE_SEED: .agent-harness/federation/keys/swe-seed-2026q3.pub
```

(One-time, per key rotation. SEA's own signing key lives in SEA under the same
SOPS/age pattern; SWE_Seed receives SEA's public key symmetrically.)

## 4. Sign a trace's chain root

```bash
swe-seed trace start "ship feature X"          # creates the trace + chain genesis
export SWE_SEED_TRACE=$(swe-seed route "ship feature X" --record | jq -r .trace_id)
swe-seed federation sign "$SWE_SEED_TRACE" --key swe-seed-2026q3 > signed-envelope.json
```

`federation sign` reads the trace's chain root from the ledger, builds a
`ProofCompleted` envelope carrying `trace_chain_root`, signs it, and writes the
signed envelope (with `payload.signature`) to stdout.

## 5. Verify a signed envelope

Verify a SWE_Seed-signed envelope by key id (reads the committed `.pub`):

```bash
swe-seed federation verify signed-envelope.json --key swe-seed-2026q3
```

Or against an explicit public-key file:

```bash
swe-seed federation verify signed-envelope.json --pubkey path/to/swe-seed-2026q3.pub
```

Exit 0 = signature valid; non-zero = invalid or malformed. A tampered envelope
(changed `result`, `trace_chain_root`, etc.) fails verification.

## Rotation

Signatures carry a `key_id`. To rotate, generate a new key (`keygen
swe-seed-2026q4`), distribute the new `.pub`, and sign with the new id. Verifiers
select the public key by `key_id`, so old and new signatures verify concurrently
during the cutover.

## Verification contract (both repos must reproduce)

The canonical signing string and a fixed test vector are the cross-repo parity
lock — see [Federation Signing and Tamper-Evident Traces](../explanations/federation-signing-and-traces.md).
If you change the canonical form, regenerate both vectors and confirm SEA still
reproduces the signature.
