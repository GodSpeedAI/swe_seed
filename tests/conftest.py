"""Pytest configuration for agentic_capability_loop integration tests."""

from __future__ import annotations

from workspace_roots import (
    CONTEXT_KERNEL_ROOT,
    GODSPEED_AGENT_ROOT,
    SEA_ROOT,
    SWE_SEED_ROOT,
    WORKSPACE_ROOT,
)


def pytest_report_header() -> list[str]:
    return [
        f"workspace_root={WORKSPACE_ROOT}",
        f"sea_root={SEA_ROOT}",
        f"swe_seed_root={SWE_SEED_ROOT}",
        f"godspeed_agent_root={GODSPEED_AGENT_ROOT}",
        f"context_kernel_root={CONTEXT_KERNEL_ROOT}",
    ]
