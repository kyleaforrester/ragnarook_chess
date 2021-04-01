#!/usr/bin/env python3

import os
import random
import subprocess as sp
from datetime import date

MAX_LINES = 50000000
FILE_SIZE = 500000
HEX_CHARS = '0123456789abcdef'

def split_to_pytorch(lines):
    lines = [lines[i:i+FILE_SIZE] for i in range(0, len(lines), FILE_SIZE)]
    for line in lines:
        today = date.today().strftime("%d%m%y")
        file_name = './pytorch/train_data/' + ''.join(random.choices(HEX_CHARS, k=10)) + '_' + today + '.train'
        fd = open(file_name, mode='w')
        fd.write('\n'.join(line))

train_files = [f for f in os.listdir('./data') if f[-6:] == '.train']
random.shuffle(train_files)

# Keep looping until all files consumed
# Group files up for a shuffle up to MAX_LINES
lines = []
for f in train_files:
    fd = open('./data/' + f).readlines()
    fd = [l.strip() for l in fd]
    lines += fd
    if len(lines) > MAX_LINES:
        random.shuffle(lines)
        split_to_pytorch(lines)
        lines = []
if len(lines) > 0:
    random.shuffle(lines)
    split_to_pytorch(lines)
    lines = []
