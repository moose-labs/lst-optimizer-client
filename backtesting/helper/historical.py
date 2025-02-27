from datasource.types import HistoricalApy
from helper.path import ensure_dir
from constant.cache import CACHED_PATH
import os


def get_apy_at_epoch(lst: str, epoch: int) -> float:
    lst = lst.lower()
    for historical_apy in load_cached_historical_apys(lst):
        if historical_apy.epoch == epoch:
            return historical_apy.apy

    raise Exception(f"{lst.upper()} APY not found for epoch {epoch}")


def load_historical_apys_until_epoch(lst: str, epoch: int) -> list[HistoricalApy]:
    """
    Load historical apys until the given epoch (eg. Provide 10, it will load until epoch 9)
    """
    lst = lst.lower()
    historical_apys = load_cached_historical_apys(lst)
    return [x for x in historical_apys if x.epoch < epoch]


def get_cached_path(lst: str) -> str:
    return f"{CACHED_PATH}/{lst.lower()}.csv"


def historical_apy_writer(symbol: str, historical_apys: list[HistoricalApy]):
    cached_path = get_cached_path(symbol)

    ensure_dir(cached_path)

    with open(cached_path, "w") as f:
        f.write("epoch,apy\n")
        for apy in historical_apys:
            f.write(f"{apy.epoch},{apy.apy}\n")

        f.close()


def is_historical_apy_file_exists(symbol: str) -> bool:
    return os.path.exists(get_cached_path(symbol))


def load_cached_historical_apys(symbol: str) -> list[HistoricalApy]:
    lines: list[str] = []

    cached_path = get_cached_path(symbol)
    with open(cached_path, "r") as f:
        lines = f.readlines()
        f.close()

    historical_apys: list[HistoricalApy] = []
    for line in lines[1:]:
        epoch, apy = line.strip().split(",")
        historical_apys.append(HistoricalApy(symbol, epoch=int(epoch), apy=float(apy)))

    return historical_apys
