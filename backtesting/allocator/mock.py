from allocator.allocator import Allocator, Allocations, Allocation


class MockAllocator(Allocator):

    def get_allocations(self, epoch: int) -> Allocations:
        return Allocations(
            epoch,
            [
                Allocation("SOL", 0.5),
                Allocation("mSOL", 0.5),
            ],
        )
