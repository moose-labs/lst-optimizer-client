import unittest

from allocator.allocator import Allocation, Allocations, Allocator, AllocationError
from allocator.ema import EmaAllocator, SortedEma, EmaHistoricalData
from datasource.types import HistoricalApy


class TestEmaAllocator(unittest.TestCase):

    def test_ema_allocator_sorted_emas(self):

        ema_period = 2
        ema_allocator = EmaAllocator()
        ema_allocator.with_data(
            [
                EmaHistoricalData(
                    "A",
                    [
                        # EMA(2) = 1, 1.667, 2.556, 3.519, 4.506
                        HistoricalApy("A", 1, 1),
                        HistoricalApy("A", 2, 2),
                        HistoricalApy("A", 3, 3),
                        HistoricalApy("A", 4, 4),
                        HistoricalApy("A", 5, 5),
                    ],
                ),
                EmaHistoricalData(
                    "B",
                    [
                        # EMA(2) = 5, 4.333, 3.444, 2.481, 1.494
                        HistoricalApy("B", 1, 5),
                        HistoricalApy("B", 2, 4),
                        HistoricalApy("B", 3, 3),
                        HistoricalApy("B", 4, 2),
                        HistoricalApy("B", 5, 1),
                    ],
                ),
            ]
        )

        expected_lastest_sorted_emas = [
            SortedEma("A", 4.506),
            SortedEma("B", 1.494),
        ]
        sorted_emas = ema_allocator.get_sorted_emas(ema_period)

        assertion_precision = 3
        for a, b in zip(sorted_emas, expected_lastest_sorted_emas):
            self.assertEqual(a.symbol, b.symbol)
            self.assertAlmostEqual(a.ema, b.ema, assertion_precision)

    def test_ema_allocator_calculate_ema(self):

        historical_apys = [
            HistoricalApy("SOL", 1, 1),
            HistoricalApy("SOL", 2, 2),
            HistoricalApy("SOL", 3, 3),
            HistoricalApy("SOL", 4, 4),
            HistoricalApy("SOL", 5, 5),
        ]
        expected_ema = [1, 1.667, 2.556, 3.519, 4.506]

        ema_allocator = EmaAllocator()
        ema_allocator.with_data(
            [
                EmaHistoricalData(
                    "SOL",
                    historical_apys,
                ),
            ]
        )

        assertion_precision = 3
        for a, b in zip(ema_allocator.calculate_ema(historical_apys, 2), expected_ema):
            self.assertAlmostEqual(a, b, assertion_precision)

    def test_ema_allocator_allocate_equal(self):
        sorted_emas = [
            SortedEma("E", 5),
            SortedEma("D", 4),
            SortedEma("C", 3),
            SortedEma("B", 2),
            SortedEma("A", 1),
        ]
        expected_allocations = [
            Allocation("E", 2000),
            Allocation("D", 2000),
            Allocation("C", 2000),
            Allocation("B", 2000),
            Allocation("A", 2000),
        ]
        allocations = EmaAllocator().allocate_equal(sorted_emas, 5)

        for a, b in zip(allocations, expected_allocations):
            self.assertEqual(a.symbol, b.symbol)
            self.assertEqual(a.ratio_bps, b.ratio_bps)
