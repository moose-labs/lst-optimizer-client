import unittest

from allocator.allocator import Allocation, Allocations, Allocator, AllocationError


class TestAllocator(unittest.TestCase):

    def test_allocator_ratio_validation(self):

        class RaiseAllocator(Allocator):

            def get_allocations(self, epoch: int) -> Allocations:
                return Allocations(
                    epoch,
                    [
                        Allocation("SOL", 1.0),
                        Allocation("JupSOL", 0.5),
                    ],
                )

        self.assertRaises(
            AllocationError,
            lambda: RaiseAllocator().get_allocations(1).must_validate(),
        )
