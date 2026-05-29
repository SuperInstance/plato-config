# plato-config — Configuration Management for Plato Agents

Agent identity, room settings, fleet parameters, and arbitrary key-value state — all from a single JSON config file. `pip install plato-config`

**Part of the [Plato](https://github.com/SuperInstance/plato-shell) ecosystem.**

## What This Gives You

- **AgentConfig** — name, role, model, capabilities, context size, deadband threshold
- **RoomConfig** — name, domain, max tiles, temperature, auto-pruning
- **JSON persistence** — `load()` and `save()` for any config file
- **Key-value extras** — `get()`/`set()` for arbitrary configuration beyond the schema
- **Default everything** — sensible defaults, zero config to start

## Quick Start

```python
from plato_config import PlatoConfig, AgentConfig, RoomConfig

# Create with defaults
config = PlatoConfig()
config.agent.name = "my-agent"
config.agent.model = "glm-5.1"
config.agent.capabilities = ["code_generation"]

# Add a room
config.add_room("graph-theory", domain="math", max_tiles=5000)

# Persist
config.save("config.json")

# Load later
config = PlatoConfig("config.json")
```

## Installation

```bash
pip install plato-config
```

## API

| Type | Description |
|------|-------------|
| `PlatoConfig` | Root config with agent, rooms, fleet URL, server port |
| `AgentConfig` | Agent identity: name, role, model, capabilities, deadband |
| `RoomConfig` | Room settings: name, domain, max_tiles, temperature, auto_prune |

## License

MIT
