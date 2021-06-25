#! /usr/bin/env python3

import argparse

parser = argparse.ArgumentParser()
parser.add_argument("file")
args = parser.parse_args()

with open(args.file, "rb") as f:
    data = f.read()

strs = []
for i in range(0, len(data), 2):
    lower, upper = int(data[i]), int(data[i+1])
    word = (upper << 8) | lower
    strs.append(f"0o{word:o}")

print(", ".join(strs))
