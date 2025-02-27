import os


def ensure_dir(filepath: str):
    path = os.path.dirname(filepath)
    if not os.path.exists(path):
        os.makedirs(path)
