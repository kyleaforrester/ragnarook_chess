#!/bin/bash

# Parse the pgns into train data
ls pgns/*.pgn | parallel --max-args 1 ./parse_pgn.py {} ">" ./data/{/.}_$(date +%m%d%y).train
