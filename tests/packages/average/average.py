#!/usr/bin/env python3
# AVERAGE.py
#   by Lut99
#
# Created:
#   07 Mar 2024, 09:34:18
# Last edited:
#   07 Mar 2024, 13:00:38
# Auto updated?
#   Yes
#
# Description:
#   Small file computing the average of a line-separated list of integers in
#   a Brane-compatible way.
#

import json
import os


def average(dataset: str) -> float:
    # Dig until your find `numbers.txt`
    while os.path.isdir(dataset):
        entries = os.listdir(dataset)
        if len(entries) != 1:
            raise RuntimeError(f"No clue which file to continue with in dataset '{dataset}'")
        dataset = os.path.join(dataset, entries[0])

    # Once it's a file, read it
    with open(dataset, "r") as h:
        n = 0
        total = 0
        for line in h.readlines():
            val = int(line.strip())
            n += 1
            total += val
        return total / n


if __name__ == "__main__":
    dataset = json.loads(os.environ["DATASET"])

    # Call average
    avg = average(dataset)

    # Print the output
    print(f"output: {avg}")
