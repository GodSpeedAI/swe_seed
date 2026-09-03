# How-To: Manage Federation Keys

This guide explains how to generate Ed25519 federation keypairs, encrypt private keys at rest using Mozilla SOPS, and rotate keys securely.

---

## 1. Goal

Establish cryptographically verified signing identities for participating in SEA-Loop federation without exposing plaintext private keys in version control.

---

## 2. Prerequisites

- `sops` and `age` installed on your machine.
- An age identity configured (e.g. in `~/.config/sops/age/keys.txt`).

---

## 3. Procedure

### Step 1: Generate a New Keypair
Run the keygen command with a unique key identifier:

```bash
cargo run -q -p swe-seed -- federation keygen node-alpha
```

### Generated Files:
- Committed Public Key: `.agent-harness/federation/keys/node-alpha.pub`
- Gitignored Private Key: `.swe-seed/federation/keys/node-alpha.key`

### Step 2: Encrypt the Private Key at Rest
Immediately encrypt the private key using the `just` recipe:

```bash
just federation-encrypt-key node-alpha
```

Behind the scenes, this runs:
```bash
sops --encrypt --in-place .swe-seed/federation/keys/node-alpha.key
```

### Step 3: Verify Encryption
Inspect the file to confirm it is encrypted:

```bash
cat .swe-seed/federation/keys/node-alpha.key
```

Verify that the file contains SOPS metadata and encrypted cipher blocks, not a plaintext private key.

### Step 4: Decrypt for Inspection (Optional)
To inspect the decrypted private key in memory:

```bash
just federation-decrypt-key node-alpha
```

### Step 5: Rotate Keys
To retire an old key and activate a new key:
1. Generate `node-beta`:
   ```bash
   cargo run -q -p swe-seed -- federation keygen node-beta
   just federation-encrypt-key node-beta
   ```
2. Commit the new public key `.agent-harness/federation/keys/node-beta.pub`.
3. Update `.agent-harness/config.yaml` to point `federation.active_key_id: "node-beta"`.
4. Keep `node-alpha.pub` in the public key directory so historical traces signed by the old key continue to pass verification.
