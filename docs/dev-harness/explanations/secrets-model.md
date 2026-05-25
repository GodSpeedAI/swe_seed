# Secrets Model

The harness supports SOPS with age so encrypted secrets can live in git while plaintext and local keys stay out of git.

SOPS owns encryption and decryption. `just secrets-*` recipes provide the project command surface. `.sops.yaml` defines encrypted paths and recipients. `.gitignore` blocks plaintext outputs and local key material.

The tradeoff is explicit setup. A developer needs `sops`, `age`, and the correct key material before editing or decrypting secrets. This is preferable to implicit secrets because failures are clear and local plaintext does not become part of the repository.

Do not commit real plaintext secrets. Do not loosen ignore rules for decrypted files. Do not replace SOPS commands with ad hoc shell encryption unless the spec changes.
