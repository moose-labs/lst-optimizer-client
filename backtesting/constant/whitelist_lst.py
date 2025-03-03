from dataclasses import dataclass
from simulation.simulation import WeightedSymbol


WEIGHTED_LST = [
    # > 1m SOL
    WeightedSymbol("jitosol", 1.0),
    WeightedSymbol("bnsol", 1.0),
    WeightedSymbol("msol", 1.0),
    WeightedSymbol("jupsol", 1.0),
    WeightedSymbol("vsol", 1.0),
    WeightedSymbol("bsol", 1.0),
    WeightedSymbol("jsol", 1.0),
    # 1m - 500k SOL
    WeightedSymbol("edgesol", 1.0),
    # WeightedSymbol("bbsol", 1.0),  # ! outlier
    WeightedSymbol("inf", 1.0),
    WeightedSymbol("dsol", 1.0),
    WeightedSymbol("hsol", 1.0),
    WeightedSymbol("aerosol", 1.0),
    # 500k - 100k SOL
    WeightedSymbol("lst", 0.5),
    WeightedSymbol("picosol", 0.5),
    WeightedSymbol("jucysol", 0.5),
    # 100k - 50k SOL
    WeightedSymbol("mangosol", 0.3),
    WeightedSymbol("strongsol", 0.3),
    WeightedSymbol("bonksol", 0.3),
    WeightedSymbol("lainesol", 0.3),
    WeightedSymbol("pumpkinsol", 0.3),
    WeightedSymbol("lakesol", 0.3),
    WeightedSymbol("polarsol", 0.3),
    # 50k - 10k SOL
    # WeightedSymbol("eonsol", 0.1),
    # WeightedSymbol("lanternsol", 0.1),
    # WeightedSymbol("hubsol", 0.1),
    # # WeightedSymbol("hasol", 0.1),  # ! outlier
    # WeightedSymbol("xandsol", 0.1),
    # WeightedSymbol("digitsol", 0.1),
    # WeightedSymbol("lumisol", 0.1),
    # WeightedSymbol("soulsol", 0.1),# ! outlier
    # WeightedSymbol("pawsol", 0.1), # ! outlier
]

AVG_LST = [
    # > 1m SOL
    WeightedSymbol("jitosol", 1.0),
    WeightedSymbol("msol", 1.0),
    WeightedSymbol("jupsol", 1.0),
    WeightedSymbol("vsol", 1.0),
    WeightedSymbol("bsol", 1.0),
    WeightedSymbol("jsol", 1.0),
]

LST = [WeightedSymbol(w.symbol, 1.0) for w in WEIGHTED_LST]
