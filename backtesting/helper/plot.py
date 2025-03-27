import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from dataclasses import dataclass


@dataclass
class Datapoint:
    epoch: int
    value: float
    description: str = ""


@dataclass
class PlotData:
    name: str
    datapoints: list[Datapoint]
    line: dict = None


def plots(title: str, plot_datas: list[PlotData]):
    """
    Plots the given data

    :param title: Title of the plot
    :param plot_datas: List of PlotData
    """
    fig = make_subplots()

    for plot_data in plot_datas:
        epochs = [x.epoch for x in plot_data.datapoints]
        y = [x.value for x in plot_data.datapoints]
        descriptions = [x.description for x in plot_data.datapoints]
        fig.add_trace(
            go.Scatter(
                x=epochs,
                y=y,
                mode="lines",
                line=plot_data.line,
                name=plot_data.name,
                text=descriptions,
            ),
        )

    fig.update_layout(
        title=title,
        xaxis_title="Epoch",
        template="plotly_dark",
        legend=dict(bordercolor="Black", borderwidth=1),
        width=1200,
        height=600,
    )
    fig.show()
