from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass
class HistoricalApy:
    symbol: str
    epoch: int
    apy: float


class DataSourceInterface(ABC):

    @abstractmethod
    def get_historical_apy(self, symbol: str, limit: int) -> float:
        pass
