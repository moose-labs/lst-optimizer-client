from dataclasses import dataclass
from portfolio.portfolio import Portfolio
from allocator.allocator import Allocation, Allocations, Allocator
from helper.sanctum import calculate_epoch_gain_from_apy


@dataclass
class WeightedSymbol:
    """
    The target LST to allocate with a weight

    Attributes:
        symbol (str): The symbol
        weight (float): The target weight will be multiplied by the allocation ratio to determine the final weight, increasing exposure to the target LST while reducing exposure to sensitive or high-risk LSTs.
    """

    symbol: str
    weight: float = 1.0


@dataclass
class EpochInfo:
    epoch: int
    apy: int
    exchange_rate: float


@dataclass
class EpochSymbolData:
    symbol: str
    epoch_infos: list[EpochInfo]


@dataclass
class PortfolioRebalanceChange:
    symbol: str
    amount: float


class Simulation:
    """
    The simulation class to simulate the portfolio

    Args:
        portfolio (Portfolio): The portfolio
        epoch_symbol_datas (list[EpochSymbolData]): The epoch symbol datas
        start_epoch (int): The start epoch
        end_epoch (int): The end epoch
    """

    def __init__(
        self,
        portfolio: Portfolio,
        epoch_symbol_datas: list[EpochSymbolData],
        start_epoch: int,
        end_epoch: int,
    ):
        self.portfolio = portfolio
        self.epoch_datas_by_symbol = {data.symbol: data for data in epoch_symbol_datas}
        self.start_epoch = start_epoch
        self.end_epoch = end_epoch
        self.__current_epoch = start_epoch

    def __iter__(self):
        """
        Make the simulation an iterator
        """
        return self

    def __next__(self):
        """
        Move to the next epoch
        """
        if self.__current_epoch >= self.end_epoch:
            raise StopIteration

        self.__current_epoch += 1

    # ? Getters

    def get_current_epoch(self) -> int:
        """
        Get the current epoch

        Returns:
            int: The current epoch
        """
        return self.__current_epoch

    def get_epoch_info(self, target_symbol: str, epoch: int) -> EpochInfo:
        """
        Get the epoch info for the given symbol and epoch

        Args:
            target_symbol (str): The symbol of the asset
            epoch (int): The current epoch

        Returns:
            EpochInfo: The epoch info for the given symbol and epoch

        Raises:
            ValueError: If the data for the target symbol is not found in the epoch
        """
        target_symbol = target_symbol.lower()
        for symbol, data in self.epoch_datas_by_symbol.items():
            if target_symbol == symbol.lower():
                for epoch_info in data.epoch_infos:
                    if epoch_info.epoch == epoch:
                        return epoch_info

        raise ValueError(f"Data for {target_symbol} not found in epoch {epoch}")

    def get_total_underlying_holding(self, epoch: int, portfolio: Portfolio):
        """
        Calculate the total underlying holding of the portfolio

        Args:
            epoch (int): The current epoch

        Returns:
            float: The total underlying holding of the portfolio
        """
        current_holdings = portfolio.get_holdings()
        total_underlying_holding = 0
        for holding in current_holdings:
            symbol = holding.symbol
            amount = holding.amount
            lst_epoch_info = self.get_epoch_info(symbol, epoch)
            total_underlying_holding += amount * lst_epoch_info.exchange_rate
        return total_underlying_holding

    # ? Adjust symbol weights to match the target allocations

    def adjust_symbol_weights(
        self, symbol_weights: list[WeightedSymbol], allocations: Allocations
    ) -> Allocations:
        """
        Adjust the symbol weights to match the target allocations

        Args:
            weights (list[WeightedSymbol]): The target weights
            allocations (Allocations): The target allocations

        Returns:
            None
        """
        weights_by_symbol = {
            weighted_symbol.symbol.lower(): weighted_symbol.weight
            for weighted_symbol in symbol_weights
        }

        total_allocation_weight = Allocations.MAX_RATIO_BPS
        total_symbol_weight = sum(
            [weights_by_symbol[s.symbol.lower()] for s in allocations.allocations]
        )

        for allocation in allocations.allocations:
            symbol = allocation.symbol.lower()
            symbol_weight = weights_by_symbol[symbol]
            allocation.ratio_bps = (
                symbol_weight / total_symbol_weight * total_allocation_weight
            )

        return allocations

    # ? Assert

    def assert_portfolio_changes(
        self, epoch: int, portfolio: Portfolio, rebalance_func
    ):
        """
        Assert that the portfolio changes are valid

        Args:
            epoch (int): The current epoch
            portfolio (Portfolio): The current portfolio
            rebalance_func (function): The function to rebalance the portfolio

        Returns:
            None

        Raises:
            AssertionError: If the rebalance failed
        """
        before_rebalance_holding = self.get_total_underlying_holding(epoch, portfolio)
        rebalance_func()
        after_rebalance_holding = self.get_total_underlying_holding(epoch, portfolio)

        # we also accept the floating point error at least 1e-9 (1 lamport)
        slippage_acceptable = -1e-9
        assert (
            after_rebalance_holding > before_rebalance_holding
            or after_rebalance_holding - before_rebalance_holding > slippage_acceptable
        ), f"Rebalance failed, underlying holding should not decrease\nBefore:\n{before_rebalance_holding}\nAfter:\n{after_rebalance_holding}"

    # ? Simulate

    def rebalance(self, epoch: int, allocations: Allocations):
        """
        Rebalance the portfolio based on the target allocations
        for the given epoch

        Args:
            epoch (int): The current epoch
            allocations (Allocations): The target allocations

        Returns:
            None
        """
        portfolio = self.portfolio
        epoch = self.get_current_epoch()

        portfolio_changes = self.calculate_portfolio_rebalance_changes(
            epoch, portfolio, allocations
        )

        # assert rebalance changes are valid
        self.assert_portfolio_changes(
            epoch,
            portfolio,
            lambda: [
                portfolio.add(portfolio_change.symbol, portfolio_change.amount)
                for portfolio_change in portfolio_changes
            ],
        )

        # simulate gain from apy after rebalancing
        for holding in portfolio.get_holdings():
            symbol = holding.symbol
            amount = holding.amount
            gain = self.calculate_gain_from_apy(epoch, symbol, amount)
            portfolio.add(symbol, gain)

    def calculate_gain_from_apy(self, epoch: int, symbol: str, amount: float) -> float:
        """
        Calculate the gain from the given apy and amount

        Args:
            epoch (int): The current epoch
            symbol (str): The symbol of the asset
            amount (float): The amount of the asset

        Returns:
            float: The gain from the given apy and amount
        """
        epoch = self.get_current_epoch()
        epoch_info = self.get_epoch_info(symbol, epoch)
        gain_ratio = calculate_epoch_gain_from_apy(epoch_info.apy)
        return amount * gain_ratio

    def calculate_portfolio_rebalance_changes(
        self, epoch: int, portfolio: Portfolio, allocations: Allocations
    ) -> list[PortfolioRebalanceChange]:
        """
        Calculate the changes needed to rebalance the portfolio

        Args:
            epoch (int): The current epoch
            portfolio (Portfolio): The current portfolio
            allocations (Allocations): The target allocations

        Returns:
            list[PortfolioRebalanceChange]: The changes needed to rebalance the portfolio
        """

        allocations.must_validate()

        # 1. Calculate underlying of current holdings
        total_underlying_holding = self.get_total_underlying_holding(epoch, portfolio)

        # 2. Calculate expected underlying holdings
        expected_underlying_holdings: dict[str, float] = {}
        for allocation in allocations.allocations:
            symbol = allocation.symbol
            ratio = allocation.ratio_bps / Allocations.MAX_RATIO_BPS
            expected_underlying_holdings[symbol] = total_underlying_holding * ratio

        # 3. Calculate expected lst holdings
        expected_lst_holdings: dict[str, float] = {}
        for symbol, expected_underlying_holding in expected_underlying_holdings.items():
            lst_epoch_info = self.get_epoch_info(symbol, epoch)
            exchange_rate = lst_epoch_info.exchange_rate
            expected_lst_holding = expected_underlying_holding / exchange_rate
            expected_lst_holdings[symbol] = expected_lst_holding

        # 4. Calculate changes from allocations
        changes: dict[str, float] = {}
        for symbol, expected_lst_holding in expected_lst_holdings.items():
            current_lst_holding = portfolio.get_holding(symbol).amount
            changes[symbol] = expected_lst_holding - current_lst_holding

        # 5. Culculate changes from current holdings
        current_holdings = portfolio.get_holdings()
        for holding in current_holdings:
            symbol = holding.symbol
            if symbol not in changes:
                changes[symbol] = -holding.amount

        return [
            PortfolioRebalanceChange(symbol, amount)
            for symbol, amount in changes.items()
        ]
