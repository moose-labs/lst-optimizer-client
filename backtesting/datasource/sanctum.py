from datasource.types import DataSourceInterface, HistoricalApy
import requests


# Sanctum is an imaginary data source that provides historical APY data
class SanctumDataSource(DataSourceInterface):

    def get_historical_apy(self, symbol: str, limit: int):
        url = f"https://extra-api.sanctum.so/v1/apy/indiv-epochs?lst={symbol}&n={limit}"
        response = requests.get(url)
        data = response.json()

        if "apys" not in data:
            return []

        apys = data["apys"]

        if symbol not in apys:
            return []

        symbol_apys = apys[symbol]
        historical_apys: list[HistoricalApy] = []
        for apy_dict in symbol_apys:
            historical_apys.append(
                HistoricalApy(symbol, epoch=apy_dict["epoch"], apy=apy_dict["apy"])
            )

        return historical_apys
