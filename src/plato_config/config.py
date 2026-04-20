"""PLATO configuration management."""

import json, os
from dataclasses import dataclass, field
from typing import Optional

@dataclass
class AgentConfig:
    name: str = "agent"
    role: str = "general"
    model: str = ""
    capabilities: list[str] = field(default_factory=list)
    max_context: int = 4096
    deadband_threshold: float = 0.7

@dataclass
class RoomConfig:
    name: str = "default"
    domain: str = "general"
    max_tiles: int = 10000
    temperature: float = 0.5
    auto_prune: bool = True

class PlatoConfig:
    def __init__(self, config_path: str = ""):
        self.agent = AgentConfig()
        self.rooms: dict[str, RoomConfig] = {}
        self.fleet_url: str = ""
        self.server_port: int = 8847
        self.log_level: str = "info"
        self._extra: dict = {}
        if config_path and os.path.exists(config_path):
            self.load(config_path)

    def load(self, path: str):
        with open(path) as f:
            data = json.load(f)
        if "agent" in data:
            for k, v in data["agent"].items():
                if hasattr(self.agent, k):
                    setattr(self.agent, k, v)
        for name, cfg in data.get("rooms", {}).items():
            self.rooms[name] = RoomConfig(**{k: v for k, v in cfg.items() if hasattr(RoomConfig(), k)})
        self.fleet_url = data.get("fleet_url", "")
        self.server_port = data.get("server_port", 8847)
        self.log_level = data.get("log_level", "info")
        self._extra = {k: v for k, v in data.items() if k not in ("agent", "rooms", "fleet_url", "server_port", "log_level")}

    def save(self, path: str):
        data = {"agent": vars(self.agent), "rooms": {n: vars(r) for n, r in self.rooms.items()},
                "fleet_url": self.fleet_url, "server_port": self.server_port, "log_level": self.log_level}
        data.update(self._extra)
        with open(path, 'w') as f:
            json.dump(data, f, indent=2)

    def get(self, key: str, default=None):
        return self._extra.get(key, default)

    def set(self, key: str, value):
        self._extra[key] = value

    def add_room(self, name: str, domain: str = "general", **kwargs) -> RoomConfig:
        cfg = RoomConfig(name=name, domain=domain, **kwargs)
        self.rooms[name] = cfg
        return cfg

    def to_dict(self) -> dict:
        return {"agent": vars(self.agent), "rooms": {n: vars(r) for n, r in self.rooms.items()},
                "fleet_url": self.fleet_url, "server_port": self.server_port, **self._extra}
