"""Cross-repo test: all four adapter modules must agree on domain_model_hash.

Verifies that:
1. All four adapter repos derive domain_model_hash from the same source.
2. All four hashes are identical.
3. The hash equals sha256(sea_file_bytes).
"""
from __future__ import annotations

import hashlib
import importlib.util
import sys
from pathlib import Path

import pytest

from workspace_roots import SEA_ROOT, SWE_SEED_ROOT, GODSPEED_AGENT_ROOT, CONTEXT_KERNEL_ROOT


DOMAIN_DIR = SEA_ROOT / "docs/specs/domains/agentic_capability_loop"
SEA_FILE = DOMAIN_DIR / "agentic_capability_loop.sea"


def _load_module(name: str, path: Path, *, package_root: Path | None = None):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    inserted_by_us = False
    if package_root and str(package_root) not in sys.path:
        sys.path.insert(0, str(package_root))
        inserted_by_us = True
    try:
        spec.loader.exec_module(module)
    finally:
        if inserted_by_us:
            sys.path.remove(str(package_root))
    return module


def test_all_adapter_hashes_match_sea_file_content() -> None:
    """All four adapter modules must produce a domain_model_hash equal to sha256(sea_file)."""
    expected_hash = hashlib.sha256(SEA_FILE.read_bytes()).hexdigest()

    adapters = {
        "SWE_SEED": (SWE_SEED_ROOT / "agentic_capability_loop/adapters.py", SWE_SEED_ROOT),
        "SEA-Forge": (SEA_ROOT / "libs/agentic_capability_loop/domain_model.py", SEA_ROOT),
        "GodSpeed-Agent": (GODSPEED_AGENT_ROOT / "agentic_capability_loop/adapters.py", GODSPEED_AGENT_ROOT),
    }

    observed: dict[str, str] = {}
    for repo_name, (adapter_path, root) in adapters.items():
        assert adapter_path.exists(), f"Adapter not found: {adapter_path}"
        mod = _load_module(f"{repo_name}_adapters", adapter_path, package_root=root)
        observed[repo_name] = mod.DOMAIN_MODEL_HASH

    # All must be identical
    hashes = list(observed.values())
    assert len(set(hashes)) == 1, (
        "domain_model_hash is not consistent across repos:\n"
        + "\n".join(f"  {k}: {v[:16]}…" for k, v in observed.items())
    )

    # All must match the .sea file content
    assert hashes[0] == expected_hash, (
        f"domain_model_hash ({hashes[0][:16]}…) does not match "
        f"sha256({SEA_FILE.name}) ({expected_hash[:16]}…).\n"
        "Regenerate the manifest and update all adapters."
    )


def test_ck_adapter_hash_matches_sea_file_content() -> None:
    """Context Kernel Python integration adapter must also carry the correct hash."""
    ck_adapter_path = CONTEXT_KERNEL_ROOT / "integration/agentic_capability_loop/adapters.py"
    if not ck_adapter_path.exists():
        pytest.skip(f"CK adapter not found: {ck_adapter_path}")

    expected_hash = hashlib.sha256(SEA_FILE.read_bytes()).hexdigest()
    mod = _load_module("ck_adapters", ck_adapter_path, package_root=CONTEXT_KERNEL_ROOT)
    assert mod.DOMAIN_MODEL_HASH == expected_hash, (
        f"CK adapter domain_model_hash ({mod.DOMAIN_MODEL_HASH[:16]}…) "
        f"!= sha256({SEA_FILE.name}) ({expected_hash[:16]}…)"
    )
