def calculate_epoch_gain_from_apy(apy: float) -> float:
    """
    Calculate the epoch gain from the APY.

    To calculate the epoch gain from the APY, use the following formula:
    EpochGain = (1 + APY/100) ^ (1/182.5) - 1

    Reference:
    https://learn.sanctum.so/docs/sanctum-lsts/after-deploying-your-lst/increasing-the-apy-for-your-holders#calculation-methodology
    """
    return (1 + apy / 100) ** (1 / 182.5) - 1
