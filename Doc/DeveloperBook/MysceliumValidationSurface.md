> SPDX-License-Identifier: MPL-2.0
> Copyright © 2021-2026 Cristian Camargo Filho

# Myscelium Validation Surface

## Scope

This note keeps the developer-facing validation surface out of the public README and inside the developer docs where it belongs.

It summarizes the current automated test scenario groups that shape the active communication model.

## Current automated scenario groups

The current `tests/test_*` directories cover seven main scenario families:

| Test scenario group | What it validates |
| --- | --- |
| `test_connection` | handshake, sync bootstrap, and basic callback round-trip |
| `test_messages` | message-style remote activation |
| `test_redirect` | cross-node rerouting |
| `test_redirect_messages` | redirected message flows |
| `test_redirect_inplace` | redirected response handling |
| `test_inplace_responses` | response execution on the receiving side |
| `test_management` | internal management flows |

## Why these matter

Taken together, these scenarios validate the parts of Myscelium that are most relevant to its intercommunication model:

1. node-to-host linking
2. sync and capability propagation
3. direct remote callback activation
4. redirect routing
5. response routing
6. internal management commands

## Interpretation

This test surface does not prove that Myscelium is finished.

What it does show is that the architecture is already being exercised as:

1. a targeted command transport
2. a redirect-capable node router
3. a callback execution substrate
4. a management and synchronization control plane

## Suggested use

Use this document when you want to answer developer questions like:

1. which communication behaviors are already covered by automated scenarios
2. which protocol paths are actively exercised
3. where new test families should be added as the runtime evolves into a fuller agent framework
