from __future__ import annotations

from typing import Any

from . import modbus as modbus
from . import replay as replay
from .asset_model import AssetCatalog
from .kernel import TagId
from .recording import RerunRecorder

class TagBinding:
    def __init__(self, tag: TagId, address: str) -> None: ...
    def tag(self) -> TagId: ...
    def address(self) -> str: ...

class SamplingPolicy:
    def __init__(self, period_ms: int, *, deadband: float | None = None) -> None: ...
    def period_ms(self) -> int: ...
    def deadband(self) -> float | None: ...

class SessionState:
    Created: SessionState
    Running: SessionState
    Stopped: SessionState

class AcquisitionSession:
    @staticmethod
    def create(
        id: str,
        bindings: list[TagBinding],
        policy: SamplingPolicy,
        catalog: AssetCatalog,
    ) -> AcquisitionSession: ...
    def start(self) -> None: ...
    def stop(self, batches_recorded: int) -> None: ...
    def id(self) -> str: ...
    def bindings(self) -> list[TagBinding]: ...
    def policy(self) -> SamplingPolicy: ...
    def state(self) -> SessionState: ...

def run_session(
    session: AcquisitionSession,
    *,
    catalog: AssetCatalog,
    source: Any,
    recorder: RerunRecorder,
) -> int: ...
