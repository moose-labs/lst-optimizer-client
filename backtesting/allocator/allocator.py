from dataclasses import dataclass
from datasource.types import HistoricalApy


@dataclass
class Allocation:
    symbol: str
    ratio_bps: int


class AllocationError(Exception):
    def __init__(
        self,
        message=f"Allocation validation failed, total ratio must be 10000",
    ):
        self.message = message
        super().__init__(self.message)


@dataclass
class Allocations:
    MAX_RATIO_BPS = 10000
    epoch: int
    allocations: list[Allocation]

    def must_validate(self):
        """
        Validate the allocations and raise AllocationError if the validation fails

        Raises:
            AllocationError: If the allocations are invalid
        """
        if self.validate() is False:
            raise AllocationError()

    def validate(self) -> bool:
        """
        Validate the allocations

        Returns:
            bool: True if the allocations are valid, False otherwise
        """
        total_ratio = 0
        for allocation in self.allocations:
            total_ratio += allocation.ratio_bps

        slippage_acceptable = 1e-9
        assertion = abs(total_ratio - Allocations.MAX_RATIO_BPS) < slippage_acceptable
        if assertion is False:
            print(
                f"Assertion failed: expected 10000, got {total_ratio} (len={len(self.allocations)})"
            )
        return assertion


class Allocator:

    def __init__(self):
        pass

    def with_data(self, data) -> "Allocator":
        """
        Set the data for the allocator
        """
        raise NotImplementedError

    def assert_data_exists(self):
        """
        Assert the data is valid
        """
        assert hasattr(
            self, "data"
        ), "data must be set before calling get_allocations()"

    def get_allocations(self, epoch: int) -> Allocations:
        """
        Get the allocations for the given epoch

        Args:
            epoch (int): The epoch

        Returns:
            Allocations: The allocations for the given epoch
        """
        raise NotImplementedError
