from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from workspace_roots import SEA_ROOT


DOMAIN_DIR = SEA_ROOT / "docs/specs/domains/agentic_capability_loop"
GENERATOR = DOMAIN_DIR / "tools/generate_contracts.py"
SEA_PYTHON = SEA_ROOT / ".venv/bin/python"


def test_generate_contracts_materializes_all_manifest_events() -> None:
    manifest_path = DOMAIN_DIR / "agentic_capability_loop.manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    event_names = set(manifest["model"]["events"])
    command_names = set(manifest["model"]["cqrs"]["commands"])
    contract_names = event_names | command_names

    result = subprocess.run(
        [str(SEA_PYTHON if SEA_PYTHON.exists() else sys.executable), str(GENERATOR)],
        cwd=SEA_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr

    schema_dir = SEA_ROOT / "generated/schemas/agentic_capability_loop"
    py_types = SEA_ROOT / "generated/python/agentic_capability_loop/types.py"
    ts_types = SEA_ROOT / "generated/typescript/agentic_capability_loop.ts"
    rust_types = SEA_ROOT / "generated/rust/agentic_capability_loop.rs"

    schema_names = {path.stem.removesuffix(".schema") for path in schema_dir.glob("*.schema.json")}
    assert schema_names == contract_names
    assert "class WorkRequestedPayload(TypedDict):" in py_types.read_text(encoding="utf-8")
    assert "export interface WorkRequestedPayload" in ts_types.read_text(encoding="utf-8")
    assert "pub struct WorkRequestedPayload" in rust_types.read_text(encoding="utf-8")


def test_sea_compilation_regenerates_equivalent_manifest_in_temp_dir(tmp_path: Path) -> None:
    sea_file = DOMAIN_DIR / "agentic_capability_loop.sea"
    committed_ast = DOMAIN_DIR / "agentic_capability_loop.ast.json"
    committed_ir = DOMAIN_DIR / "agentic_capability_loop.ir.json"
    committed_manifest = DOMAIN_DIR / "agentic_capability_loop.manifest.json"

    ast_out = tmp_path / "agentic_capability_loop.ast.json"
    ir_out = tmp_path / "agentic_capability_loop.ir.json"
    manifest_out = tmp_path / "agentic_capability_loop.manifest.json"

    commands = [
        [str(SEA_PYTHON if SEA_PYTHON.exists() else sys.executable), str(SEA_ROOT / "tools/sea_parse.py"), str(sea_file), str(ast_out)],
        [str(SEA_PYTHON if SEA_PYTHON.exists() else sys.executable), str(SEA_ROOT / "tools/ast_to_ir.py"), str(ast_out), str(ir_out)],
        [str(SEA_PYTHON if SEA_PYTHON.exists() else sys.executable), str(SEA_ROOT / "tools/ir_to_manifest.py"), str(ir_out), str(manifest_out)],
    ]
    for command in commands:
        result = subprocess.run(
            command,
            cwd=SEA_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        assert result.returncode == 0, result.stderr

    regenerated_manifest = json.loads(manifest_out.read_text(encoding="utf-8"))
    baseline_manifest = json.loads(committed_manifest.read_text(encoding="utf-8"))

    assert ast_out.stat().st_size > 0
    assert ir_out.stat().st_size > 0
    assert manifest_out.stat().st_size > 0
    assert ast_out.stat().st_size == committed_ast.stat().st_size
    assert set(regenerated_manifest["model"]["events"]) == set(baseline_manifest["model"]["events"])
    assert set(regenerated_manifest["model"]["aggregates"]) == set(baseline_manifest["model"]["aggregates"])
    assert regenerated_manifest["context"] == baseline_manifest["context"]
    assert regenerated_manifest["manifestVersion"] == baseline_manifest["manifestVersion"]
    assert committed_ir.stat().st_size > 0

    # Phase 1 invariant: manifest must contain meta.sea_file_hash
    assert "meta" in regenerated_manifest, (
        "Regenerated manifest missing 'meta' key. "
        "Update tools/ir_to_manifest.py per Phase 1 of the implementation plan."
    )
    assert regenerated_manifest["meta"].get("sea_file_hash"), (
        "Regenerated manifest meta.sea_file_hash is absent or empty."
    )

    # Verify the hash matches the actual .sea file
    import hashlib as _hashlib
    _sea_content = sea_file.read_bytes()
    _expected_hash = _hashlib.sha256(_sea_content).hexdigest()
    assert regenerated_manifest["meta"]["sea_file_hash"] == _expected_hash, (
        f"meta.sea_file_hash in regenerated manifest does not match sha256 of {sea_file.name}"
    )
