from dataclasses import dataclass


@dataclass
class Holding:
    symbol: str
    amount: float


class Portfolio:
    """
    The portfolio class to manage the holdings
    """

    def __init__(self):
        self.__holding_by_symbols: dict[str, Holding] = {}

    def add(self, symbol: str, amount: int):
        """
        Add a token to the portfolio

        Args:
            symbol (str): The symbol of the token
            amount (int): The amount of the token
        """
        holding = self.__holding_by_symbols.get(symbol, Holding(symbol, 0))
        holding.amount += amount
        self.__holding_by_symbols[symbol] = holding

    def get_holding(self, symbol: str) -> Holding:
        """
        Get the holding of the given symbol

        Args:
            symbol (str): The symbol of the token

        Returns:
            Holding: The holding of the given symbol
        """
        return self.__holding_by_symbols.get(symbol, Holding(symbol, 0))

    def get_holdings(self) -> list[Holding]:
        """
        Get all holdings

        Returns:
            list[Holding]: The list of holdings
        """
        return self.__holding_by_symbols.values()

    def get_total_token_amount(self) -> float:
        """
        Get the total token amount

        Returns:
            float: The total token amount
        """
        return sum([h.amount for h in self.__holding_by_symbols.values()])

    def breakdown(self):
        """
        Print the portfolio breakdown
        """
        print("=======================")
        print("Portfolio Breakdown")
        for s, h in self.__holding_by_symbols.items():
            if h.amount == 0:
                continue

            print(f" - {h.amount} {s.upper()}")
        print(f"Total token amount: {self.get_total_token_amount()}")
        print("----------------------")
