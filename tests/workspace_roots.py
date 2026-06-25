"""Workspace root discovery for cross-repo integration tests.

Resolves the four repository roots using environment variables first,
then falls back to searching relative to this file's location.

Environment variables (all optional, override auto-discovery):
    GODSPEED_WORKSPACE_ROOT  — parent of all four repos
    SEA_ROOT                 — SEA-Forge repo root
    SWE_SEED_ROOT            — SWE_SEED repo root
    GODSPEED_AGENT_ROOT      — GodSpeed-Agent repo root
    CONTEXT_KERNEL_ROOT      — Context-Kernel repo root

Alternate repo folder names supported (checked in order):
    SEA-Forge:      SEA, SEA-main, SEA-Forge, SEA-Forge-main
    SWE_SEED:       SWE_SEED, SWE_SEED-main, swe_seed, swe_seed-main
    GodSpeed-Agent: godspeed_agent, GodSpeed-Agent, GodSpeed-Agent-main
    Context-Kernel: Context_Kernel_Service_MCP, Context-Kernel, Context-Kernel-main
"""

from __future__ import annotations

import os
from pathlib import Path


class WorkspaceDiscoveryError(Exception):
    """Raised when workspace or repo root discovery fails."""


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

# Alternate folder names tried when a repo cannot be found via env var.
_REPO_NAMES: dict[str, list[str]] = {
    "SEA": ["SEA", "SEA-main", "SEA-Forge", "SEA-Forge-main"],
    "SWE_SEED": ["SWE_SEED", "SWE_SEED-main", "swe_seed", "swe_seed-main"],
    "godspeed_agent": [
        "godspeed_agent",
        "GodSpeed-Agent",
        "GodSpeed-Agent-main",
        "godspeed-agent",
    ],
    "Context_Kernel_Service_MCP": [
        "Context_Kernel_Service_MCP",
        "Context-Kernel",
        "Context-Kernel-main",
        "context-kernel",
    ],
}

# Required sentinel files — if any of these exist the folder is the right repo.
_REPO_SENTINELS: dict[str, list[str]] = {
    "SEA": ["tools/sea_parse.py", "docs/specs", "libs"],
    "SWE_SEED": ["scripts/harness.py", "HARNESS_SPEC.md"],
    "godspeed_agent": ["godspeed_nav/__init__.py", "pyproject.toml"],
    "Context_Kernel_Service_MCP": ["crates/ck-mcp", "Cargo.toml"],
}

# Env-variable names, one per repo.
_REPO_ENV_VARS: dict[str, str] = {
    "SEA": "SEA_ROOT",
    "SWE_SEED": "SWE_SEED_ROOT",
    "godspeed_agent": "GODSPEED_AGENT_ROOT",
    "Context_Kernel_Service_MCP": "CONTEXT_KERNEL_ROOT",
}


# ---------------------------------------------------------------------------
# Discovery logic
# ---------------------------------------------------------------------------

def _workspace_root() -> Path:
    """Return the common parent of all four repos.

    Priority:
    1. GODSPEED_WORKSPACE_ROOT env var
    2. Inferred from this file's location (tests/ → SWE_SEED/ → projects/)
    """
    env = os.environ.get("GODSPEED_WORKSPACE_ROOT")
    if env:
        path = Path(env).resolve()
        if path.is_dir():
            return path
        _fatal(
            f"GODSPEED_WORKSPACE_ROOT={env!r} is set but does not exist.\n"
            f"  Expected a directory containing SEA-Forge, SWE_SEED, GodSpeed-Agent, Context-Kernel."
        )

    markers = ["pyproject.toml", ".git", "setup.cfg"]
    inferred: Path | None = None
    for parent in Path(__file__).resolve().parents:
        if any((parent / m).exists() for m in markers) and (parent / "tests").is_dir():
            inferred = parent.parent
            break
    if inferred is None:
        _fatal(
            "Cannot infer workspace root from this file's location.\n"
            "  Set GODSPEED_WORKSPACE_ROOT to the parent directory of all repos."
        )
    return inferred


def _find_repo(canonical_key: str, workspace_root: Path) -> Path:
    """Locate a repo root, respecting env-var overrides and alternate names."""
    env_var = _REPO_ENV_VARS[canonical_key]
    env_value = os.environ.get(env_var)

    if env_value:
        path = Path(env_value).resolve()
        if path.is_dir():
            return path
        _fatal(
            f"{env_var}={env_value!r} is set but does not exist.\n"
            f"  Point {env_var} to the {canonical_key} repo root."
        )

    # Search under workspace root
    tried: list[str] = []
    for name in _REPO_NAMES[canonical_key]:
        candidate = workspace_root / name
        tried.append(str(candidate))
        if candidate.is_dir():
            # Verify via sentinel file
            sentinels = _REPO_SENTINELS[canonical_key]
            if any((candidate / s).exists() for s in sentinels):
                return candidate

    _fatal(
        f"Cannot find repo '{canonical_key}' under {workspace_root}.\n"
        f"  Tried: {tried}\n"
        f"\n"
        f"  Fix options:\n"
        f"    export {env_var}=/path/to/{canonical_key}\n"
        f"    export GODSPEED_WORKSPACE_ROOT=/path/to/projects/\n"
        f"\n"
        f"  Or ensure the repo folder is one of: {_REPO_NAMES[canonical_key]}"
    )


def _fatal(message: str) -> None:
    """Raise a clear discovery error instead of terminating the process."""
    raise WorkspaceDiscoveryError(
        f"\n{'=' * 70}\n"
        f"WORKSPACE ROOT DISCOVERY FAILED\n"
        f"{'=' * 70}\n"
        f"{message}\n"
        f"{'=' * 70}\n"
    )


# ---------------------------------------------------------------------------
# Public API — resolved at import time
# ---------------------------------------------------------------------------

WORKSPACE_ROOT: Path = _workspace_root()

SEA_ROOT: Path = _find_repo("SEA", WORKSPACE_ROOT)
SWE_SEED_ROOT: Path = _find_repo("SWE_SEED", WORKSPACE_ROOT)
GODSPEED_AGENT_ROOT: Path = _find_repo("godspeed_agent", WORKSPACE_ROOT)
CONTEXT_KERNEL_ROOT: Path = _find_repo("Context_Kernel_Service_MCP", WORKSPACE_ROOT)


def discovery_report() -> str:
    """Return a human-readable discovery report for evidence logging."""
    env_used = {
        k: os.environ.get(v, "(auto-discovered)")
        for k, v in {
            "GODSPEED_WORKSPACE_ROOT": "GODSPEED_WORKSPACE_ROOT",
            "SEA_ROOT": "SEA_ROOT",
            "SWE_SEED_ROOT": "SWE_SEED_ROOT",
            "GODSPEED_AGENT_ROOT": "GODSPEED_AGENT_ROOT",
            "CONTEXT_KERNEL_ROOT": "CONTEXT_KERNEL_ROOT",
        }.items()
    }
    lines = [
        "Workspace root discovery",
        f"  WORKSPACE_ROOT        = {WORKSPACE_ROOT}",
        f"  SEA_ROOT              = {SEA_ROOT}",
        f"  SWE_SEED_ROOT         = {SWE_SEED_ROOT}",
        f"  GODSPEED_AGENT_ROOT   = {GODSPEED_AGENT_ROOT}",
        f"  CONTEXT_KERNEL_ROOT   = {CONTEXT_KERNEL_ROOT}",
        "",
        "  env vars checked:",
    ]
    for k, v in env_used.items():
        lines.append(f"    {k}={v}")
    return "\n".join(lines)
