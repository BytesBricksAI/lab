from __future__ import annotations

from typing import Any

from .kernel import TagId

PLANT_TIME: str
EVENTS_PATH: str

class RerunRecorder:
    def __init__(self, stream: Any) -> None: ...
    @staticmethod
    def to_file(app_id: str, path: str) -> RerunRecorder: ...
    def flush(self) -> None: ...

def tag_entity_path(tag: TagId) -> str: ...
