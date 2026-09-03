# How-To: Configure MCP Servers

This guide explains how to register backend Model Context Protocol (MCP) servers under MCPGate, configure tool namespacing, and set security policies.

---

## 1. Goal

Connect one or more external MCP servers (such as a local filesystem server or remote database explorer) to MCPGate, ensuring tools are namespaced and governed.

---

## 2. Prerequisites

- MCP server binary or script available locally (or remote SSE endpoint).
- SWE_SEED repository initialized.

---

## 3. Procedure

### Step 1: Update MCPGate Configuration
Open `.agent-harness/config.yaml` (or your gateway configuration block):

```yaml
gateway:
  listen: "stdio" # or "127.0.0.1:8080" for SSE
  servers:
    filesystem:
      transport: "stdio"
      command: "npx"
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/home/user/projects"]
    github:
      transport: "stdio"
      command: "npx"
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: "${GITHUB_TOKEN}"

  policies:
    # Allow read operations automatically
    - pattern: "filesystem.read_*"
      action: "allow"
    - pattern: "github.get_*"
      action: "allow"
    # Require human approval for writes
    - pattern: "filesystem.write_file"
      action: "require_approval"
    # Block destructive actions completely
    - pattern: "github.delete_*"
      action: "block"
```

### Step 2: Test Server Discovery
Inspect the virtualized catalog to confirm that tools are correctly discovered and namespaced:

```bash
cargo run -q -p swe-seed -- gateway list-tools
```

### Expected Output:
```text
Discovered 2 backend servers:
  - filesystem:
      * filesystem.read_file
      * filesystem.list_directory
      * filesystem.write_file
  - github:
      * github.get_issue
      * github.create_issue
      * github.delete_repo [BLOCKED BY POLICY]
```

### Step 3: Configure Coding Agent Host
Point your coding agent (e.g. Claude Code or Antigravity) to MCPGate:
In `.claude/settings.json` or host configuration:
```json
{
  "mcpServers": {
    "mcpgate": {
      "command": "swe-seed",
      "args": ["gateway", "serve"]
    }
  }
}
```

---

## 4. Common Failure Symptoms

- **Tool Collision**: Two servers offer a tool named `search`. MCPGate resolves this by prefixing server IDs (`serverA.search`, `serverB.search`).
- **Policy Block**: Calling a tool matches a `block` rule. MCPGate returns JSON-RPC error code `-32001` (Forbidden).
