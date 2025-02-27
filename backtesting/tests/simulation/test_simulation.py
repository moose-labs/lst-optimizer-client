import unittest

from simulation.simulation import (
    Simulation,
    EpochSymbolData,
    EpochInfo,
    PortfolioRebalanceChange,
    WeightedSymbol,
)
from portfolio.portfolio import Portfolio
from allocator.allocator import Allocation, Allocations, Allocator


class MockAllocator(Allocator):

    def get_symbol_allocation(self, epoch: int) -> Allocations:
        return Allocations(
            epoch,
            [
                Allocation("SOL", 0),
                Allocation("JupSOL", 5000),
                Allocation("dSOL", 5000),
            ],
        )


class TestSimulation(unittest.TestCase):

    def test_simulation_portfolio_assertion(self):
        epoch_symbol_datas = [
            EpochSymbolData(
                "SOL",
                [
                    EpochInfo(1, 0, 1.0),
                    EpochInfo(2, 0, 1.0),
                ],
            ),
        ]
        portfolio = Portfolio()
        portfolio.add("SOL", 100)
        s = Simulation(
            portfolio,
            epoch_symbol_datas,
            1,
            2,
        )

        self.subTest("should success when after rebalance GTE")
        s.assert_portfolio_changes(1, portfolio, lambda: portfolio.add("SOL", 1))

        self.subTest("should success when after rebalance (sigthly loss)")
        s.assert_portfolio_changes(
            1, portfolio, lambda: portfolio.add("SOL", -0.000_000_000_1)
        )

        self.subTest("should fail when after rebalance (sigthly loss)")
        self.assertRaises(
            AssertionError,
            lambda: s.assert_portfolio_changes(
                1, portfolio, lambda: portfolio.add("SOL", -0.000_000_001)
            ),
        )

        self.subTest("should fail when after rebalance LT")
        self.assertRaises(
            AssertionError,
            lambda: s.assert_portfolio_changes(
                1, portfolio, lambda: portfolio.add("SOL", -1)
            ),
        )

    def test_simulation_calculate_portfolio_chages(self):

        portfolio = Portfolio()
        portfolio.add("SOL", 100)

        allocator = MockAllocator()
        simulation_datas = [
            EpochSymbolData(
                "SOL",
                [
                    EpochInfo(1, 0, 1.0),
                    EpochInfo(2, 0, 1.0),
                ],
            ),
            EpochSymbolData(
                "JupSOL",
                [
                    EpochInfo(1, 0, 2.0),
                    EpochInfo(2, 0, 4.0),
                ],
            ),
            EpochSymbolData(
                "dSOL",
                [
                    EpochInfo(1, 0, 2.0),
                    EpochInfo(2, 0, 4.0),
                ],
            ),
        ]

        s = Simulation(portfolio, simulation_datas, 1, 2)

        epoch = s.get_current_epoch()
        self.assertEqual(
            # 50, 50 Each divided by its exchange rate (2.0 @ epoch 1)
            [
                PortfolioRebalanceChange("SOL", -100.0),
                PortfolioRebalanceChange("JupSOL", 25.0),
                PortfolioRebalanceChange("dSOL", 25.0),
            ],
            s.calculate_portfolio_rebalance_changes(
                epoch,
                portfolio,
                allocator.get_symbol_allocation(epoch),
            ),
        )

        s.__next__()

        epoch = s.get_current_epoch()
        self.assertEqual(
            # 50, 50 Each divided by its exchange rate (4.0 @ epoch 2)
            [
                PortfolioRebalanceChange("SOL", -100.0),
                PortfolioRebalanceChange("JupSOL", 12.5),
                PortfolioRebalanceChange("dSOL", 12.5),
            ],
            s.calculate_portfolio_rebalance_changes(
                epoch,
                portfolio,
                allocator.get_symbol_allocation(epoch),
            ),
        )

    def test_simulation_adjust_symbol_weights(self):
        symbol_weights = [WeightedSymbol("A", 0.8), WeightedSymbol("B", 0.2)]
        allocations = Allocations(
            0,
            [
                Allocation("A", 5000),
                Allocation("B", 5000),
            ],
        )
        expected_new_allcations = [
            Allocation("A", 8000),
            Allocation("B", 2000),
        ]

        s = Simulation(Portfolio(), [], 0, 0)
        new_allcations = s.adjust_symbol_weights(symbol_weights, allocations)
        self.assertEqual(
            expected_new_allcations,
            new_allcations.allocations,
        )
