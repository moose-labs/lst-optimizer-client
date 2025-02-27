# LST Optimizer Backtesting

## Overview

This repository provides tools for loading historical data, visualizing LST indicators, and running backtesting with customizable allocation logic. The core functionalities are implemented across three Jupyter notebooks.

## Notebooks

### 1. `historical_loader.ipynb`

- Loads historical data for analysis and backtesting.
- Fetches and preprocesses LST-related datasets.

### 2. `historical_ploter.ipynb`

- Plots historical LST indicators.
- Provides visual insights into trends and behaviors of LST allocations over time.

### 3. `backtesting.ipynb`

- Runs backtesting simulations.
- Allows customization of the allocator, which defines the business logic for LST allocation.

## Usage

1. Ensure you have the necessary dependencies installed.
2. Run `historical_loader.ipynb` to load the required historical data.
3. Use `historical_ploter.ipynb` to visualize key indicators.
4. Execute `backtesting.ipynb` to test allocation strategies with different parameters.

## Dependencies

- Python 3.x
- Jupyter Notebook
- Pandas, Matplotlib, and other required data science libraries
