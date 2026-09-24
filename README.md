<p align="center">
  <img src="./myscelium_logo_centralized.png" alt="Myscelium logo" width="220">
</p>

<h1 align="center">Myscelium</h1>

<p align="center">
  A Python-first host/client networking runtime backed by Rust.
</p>

<p align="center">
  <a href="./LICENSE">MPL-2.0</a> ·
  <a href="./Doc/Usability/Usability.md">Usage guide</a> ·
  <a href="./Doc/DeveloperBook/MysceliumPythonRustInitializationFlow.md">Architecture</a> ·
  <a href="./INSTRUCTIONS.md">Build instructions</a>
</p>

> [!IMPORTANT]
> Myscelium is in public-release stabilization. The repository is suitable for
> review and development, but the build, test matrix, packaging workflow, and
> public API are not yet declared stable.

## What Myscelium is

Myscelium connects Python processes through a host/client model. Applications
use a Python API to register callbacks, send commands, route responses, and
coordinate work across machines. A PyO3 extension bridges that API to the Rust
runtime responsible for sockets, asynchronous tasks, routing, buffering, and
state management.

```mermaid
flowchart LR
    App[Python application] --> API[Python API]
    API --> Bridge[PyO3 bridge]
    Bridge --> Core[Rust core]
    Core --> Host[Host runtime]
    Core --> Client[Client runtime]
    Host <--> Network[TCP network]
    Client <--> Network
```

```mermaid
flowchart TD

subgraph group_api["Python API"]
  node_python_api["Public Python API"]
  node_command_patterns["Command Validation<br/>[patterns.py]"]
  node_logs_interface["Client Log Interface<br/>[interfaces.py]"]
end

subgraph group_bridge["Rust Bridge"]
  node_pyo3["PyO3 Bridge<br/>[lib.rs]"]
  node_client_entry["Client Entry"]
  node_value_conversion["Value Conversion<br/>[functions.rs]"]
  node_python_task_queue["Python Task Queue<br/>[lib.rs]"]
  node_host_entry["Host Entry"]
end

subgraph group_runtime["Network Runtime"]
  node_rust_backend["Rust Backend"]
  node_host_runtime["Host Runtime"]
  node_client_runtime["Client Runtime"]
  node_tcp_network(("TCP Network"))
end

subgraph group_support["State and Logs"]
  node_client_handlers[("Client Metadata")]
  node_host_logs["Host Log Retrieval"]
  node_client_logs["Client Log Retrieval"]
  node_client_log_store[("Client Log Buffer")]
end

node_app(("Python Application"))

node_app -->|"uses"| node_python_api
node_python_api -->|"validates commands"| node_command_patterns
node_python_api -->|"calls"| node_pyo3
node_pyo3 -->|"dispatches"| node_client_entry
node_client_entry -->|"calls"| node_rust_backend
node_pyo3 -->|"dispatches"| node_host_entry
node_host_entry -->|"calls"| node_rust_backend
node_client_entry -->|"converts values"| node_value_conversion
node_client_entry -.->|"queues Python work"| node_python_task_queue
node_rust_backend -->|"runs"| node_host_runtime
node_rust_backend -->|"runs"| node_client_runtime
node_host_runtime -->|"communicates over TCP"| node_tcp_network
node_client_runtime -->|"communicates over TCP"| node_tcp_network
node_host_logs --> node_client_handlers
node_logs_interface -->|"retrieves logs"| node_client_logs
node_client_logs -->|"reads and deletes"| node_client_log_store

click node_python_api "https://github.com/myscelium/myscelium/tree/main/Myscelium/myscelium"
click node_command_patterns "https://github.com/myscelium/myscelium/blob/main/Myscelium/myscelium/common/patterns.py"
click node_logs_interface "https://github.com/myscelium/myscelium/blob/main/Myscelium/myscelium/client/interfaces.py"
click node_pyo3 "https://github.com/myscelium/myscelium/blob/main/Myscelium/rust/src/lib.rs"
click node_client_entry "https://github.com/myscelium/myscelium/blob/main/Myscelium/rust/src/client_entry_point.rs"
click node_value_conversion "https://github.com/myscelium/myscelium/blob/main/Myscelium/rust/src/common/functions.rs"
click node_python_task_queue "https://github.com/myscelium/myscelium/blob/main/Myscelium/RustPyNet/RustPyNet/src/lib.rs"
click node_rust_backend "https://github.com/myscelium/myscelium/tree/main/Myscelium/OxidizedMysceliumCore"
click node_host_runtime "https://github.com/myscelium/myscelium/tree/main/Myscelium/OxidizedMysceliumCore"
click node_client_runtime "https://github.com/myscelium/myscelium/tree/main/Myscelium/OxidizedMysceliumCore"
click node_host_entry "https://github.com/myscelium/myscelium/blob/main/Myscelium/rust/src/host_entry_point.rs"
click node_client_handlers "https://github.com/myscelium/myscelium/blob/main/Myscelium/myscelium/host_client_events_retriever.py"
click node_host_logs "https://github.com/myscelium/myscelium/blob/main/Myscelium/myscelium/host_logs_retriever.py"
click node_client_logs "https://github.com/myscelium/myscelium/blob/main/Myscelium/myscelium/client_logs_retriever.py"
click node_client_log_store "https://github.com/myscelium/myscelium/blob/main/Myscelium/myscelium/client_logs_retriever.py"

classDef toneNeutral fill:#f8fafc,stroke:#334155,stroke-width:1.5px,color:#0f172a
classDef toneBlue fill:#dbeafe,stroke:#2563eb,stroke-width:1.5px,color:#172554
classDef toneAmber fill:#fef3c7,stroke:#d97706,stroke-width:1.5px,color:#78350f
classDef toneMint fill:#dcfce7,stroke:#16a34a,stroke-width:1.5px,color:#14532d
classDef toneRose fill:#ffe4e6,stroke:#e11d48,stroke-width:1.5px,color:#881337
classDef toneIndigo fill:#e0e7ff,stroke:#4f46e5,stroke-width:1.5px,color:#312e81
classDef toneTeal fill:#ccfbf1,stroke:#0f766e,stroke-width:1.5px,color:#134e4a
class node_python_api,node_command_patterns,node_logs_interface toneBlue
class node_pyo3,node_client_entry,node_value_conversion,node_python_task_queue,node_host_entry toneAmber
class node_rust_backend,node_host_runtime,node_client_runtime,node_tcp_network toneMint
class node_client_handlers,node_host_logs,node_client_logs,node_client_log_store toneRose
class node_app toneIndigo
```

## Repository layout

| Path | Purpose |
|---|---|
| `Myscelium/myscelium/` | Public Python API and helper modules |
| `Myscelium/rust/` | PyO3 bridge exposed to Python |
| `Myscelium/OxidizedMysceliumCore/` | Rust backend submodule |
| `Myscelium/tests/` | Python integration and behavior tests |
| `Doc/Usability/` | User-facing documentation |
| `Doc/DeveloperBook/` | Architecture and maintainer documentation |
| `INSTRUCTIONS.md` | Existing build, installation, and test notes |

## Get the source

Clone with submodules so the Rust backend is available to the bridge:

```bash
git clone --recurse-submodules https://github.com/Myscelium/myscelium.git
cd myscelium
git submodule update --init --recursive
```

The intended development stack currently includes Python, Rust, `setuptools-rust`,
and `wheel`. See [INSTRUCTIONS.md](./INSTRUCTIONS.md) for the existing local
workflow. Treat the historical Windows CPython 3.10 binary as a development
artifact, not as a portable release package.

## Documentation

- [Usage guide](./Doc/Usability/Usability.md)
- [Python-to-Rust initialization flow](./Doc/DeveloperBook/MysceliumPythonRustInitializationFlow.md)
- [Open-source maturity plan](./Doc/DeveloperBook/MysceliumOpenSourceMaturityPlan.md)
- [Security cleanup record](./SECURITY_CLEANUP.md)

## Project status

The repository history and source licensing have been sanitized for public
development. Before a stable release, the project still needs a reproducible
clean-machine build, a passing cross-platform test matrix, stable API and
protocol commitments, and repeatable package publishing.

## License

Myscelium is licensed under the [Mozilla Public License 2.0](./LICENSE).

Copyright © 2021-2026 Cristian Camargo Filho.
