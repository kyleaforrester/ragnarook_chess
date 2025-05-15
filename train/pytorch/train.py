#!/usr/bin/env python3

import net
import random
import sys
import os.path
import torch
import multiprocessing as mp
import gc

HISTORY_LEN = 100
PROMPT = '''Select an option:
    1) Load net
    2) Save net
    3) New net
    4) Train net
    5) Evaluate position
    6) Change learning rate
    7) Performance on validation data
Command: '''


def load_net():
    while True:
        file_name = input('What is the path to the net? ')
        if os.path.isfile(file_name):
            return torch.load(file_name).eval()
        else:
            print('File not found!')

def save_net(my_net):
    while True:
        file_name = input('What should the new file be named? ')
        if len(file_name) > 0:
            torch.save(my_net, file_name)
            return
        else:
            print('Could not read entered file name.')

def train_net():
    train_files = [f for f in os.listdir('./train_data') if f[-6:] == '.train']
    random.shuffle(train_files)
    inputs, wdl_labels, moves_labels = (None, None, None)
    history = None

    auto_tune = input('Autotune run (Y/N)? ')
    if auto_tune == 'Y':
        auto_tune = True
    elif auto_tune == 'N':
        auto_tune = False
    else:
        print('Invalid Autotune option')
        return

    epochs = int(input('Training epochs: '))
    for i in range(epochs):
        file_name = './train_data/' + train_files[i % len(train_files)]
        next_file_name = './train_data/' + train_files[(i + 1) % len(train_files)]
        print('Epoch {}: Using file {}'.format(i, file_name))

        # First iteration populate inputs and labels
        if i == 0:
            q = mp.Queue()
            p = mp.Process(target=read_file, args=(file_name, q))
            p.start()
            inputs, wdl_labels, moves_labels = q.get()
            p.join()

        # Begin parsing the next file
        q = mp.Queue()
        p = mp.Process(target=read_file, args=(next_file_name, q))
        p.start()

        # Spend time training
        my_net.train_file(inputs, wdl_labels, moves_labels)

        # Free memory
        del inputs
        del wdl_labels
        del moves_labels
        gc.collect()

        # Check if we are not making progress by checking validation data
        if auto_tune == True and i % HISTORY_LEN == 0:
            v_q = mp.Queue()
            read_file('validation.data', v_q)
            v_inputs, v_wdl_labels, v_moves_labels = v_q.get()
            loss = my_net.validation_loss(v_inputs, v_wdl_labels, v_moves_labels)
            if history is not None and history < loss:
                # Reduce learning rate
                new_rate = my_net.learning_rate / 2
                print('Reducing learning rate from {} to {}'.format(my_net.learning_rate, new_rate))
                my_net.learning_rate = new_rate
                for g in my_net.optimizer.param_groups:
                    g['lr'] = new_rate
            history = loss
            del v_inputs
            del v_wdl_labels
            del v_moves_labels

        # Pick up the next inputs and labels
        inputs, wdl_labels, moves_labels = q.get()
        p.join()
        gc.collect()

def validation_perf(my_net):
    q = mp.Queue()
    read_file('validation.data', q)
    inputs, wdl_labels, moves_labels = q.get()

    loss = my_net.validation_loss(inputs, wdl_labels, moves_labels)

    del inputs
    del wdl_labels
    del moves_labels


def eval_pos(my_net):
    fen = input('Fen to evaluate: ')
    tensor = torch.tensor([board_to_tensor(fen)], device=my_net.device)
    print('Evaluation: {}'.format(my_net(tensor)))

def change_rate(my_net):
    if my_net is None:
        print('Initialize network first!')
        return

    print('Current rate is {}'.format(my_net.learning_rate))
    my_net.learning_rate = float(input('New rate: '))

    for g in my_net.optimizer.param_groups:
        g['lr'] = my_net.learning_rate

def read_file(file_name, q):
    samples = open(file_name).readlines()
    random.shuffle(samples)
    samples = [l.strip().split(',') for l in samples]
    inputs = [board_to_tensor(l[0]) for l in samples]
    wdl_labels = [int(l[1]) for l in samples]
    moves_labels = [[float(l[2])] for l in samples]
    q.put((inputs, wdl_labels, moves_labels))


def board_to_tensor(fen):
    # Format:
    # 1: my_pawns
    # 2: my_knights
    # 3: my_bishops
    # 4: my_rooks
    # 5: my_queens
    # 6: my_king
    # 7: enemy_pawns
    # 8: enemy_knights
    # 9: enemy_bishops
    # 10: enemy_rooks
    # 11: enemy_queens
    # 12: enemy_king
    # Board is arranged to it is my move
    # First row is board positions 56 - 63
    # Last row is board positions 0 - 7

    elems = fen.split(' ')
    position = elems[0]
    is_w_move = elems[1] == 'w'
    castling = elems[2]
    en_passent = elems[3] != '-'

    my_pawns = [[0.0]*8 for _ in range(8)]
    my_knights = [[0.0]*8 for _ in range(8)]
    my_bishops = [[0.0]*8 for _ in range(8)]
    my_rooks = [[0.0]*8 for _ in range(8)]
    my_queens = [[0.0]*8 for _ in range(8)]
    my_king = [[0.0]*8 for _ in range(8)]
    enemy_pawns = [[0.0]*8 for _ in range(8)]
    enemy_knights = [[0.0]*8 for _ in range(8)]
    enemy_bishops = [[0.0]*8 for _ in range(8)]
    enemy_rooks = [[0.0]*8 for _ in range(8)]
    enemy_queens = [[0.0]*8 for _ in range(8)]
    enemy_king = [[0.0]*8 for _ in range(8)]

    # Rotate the board if it is black's move
    # Flip the board so both white and black will short castle to the right of the screen. Training data is indistinguishable who is white or black, they will look the same.
    if is_w_move:
        position = position.split('/')
    else:
        position = position.split('/')[::-1]

    for row in enumerate(position):
        col = 0
        for char in row[1]:
            if char == 'p':
                if is_w_move:
                    enemy_pawns[row[0]][col] = 1.0
                else:
                    my_pawns[row[0]][col] = 1.0
            elif char == 'n':
                if is_w_move:
                    enemy_knights[row[0]][col] = 1.0
                else:
                    my_knights[row[0]][col] = 1.0
            elif char == 'b':
                if is_w_move:
                    enemy_bishops[row[0]][col] = 1.0
                else:
                    my_bishops[row[0]][col] = 1.0
            elif char == 'r':
                if is_w_move:
                    enemy_rooks[row[0]][col] = 1.0
                else:
                    my_rooks[row[0]][col] = 1.0
            elif char == 'q':
                if is_w_move:
                    enemy_queens[row[0]][col] = 1.0
                else:
                    my_queens[row[0]][col] = 1.0
            elif char == 'k':
                if is_w_move:
                    enemy_king[row[0]][col] = 1.0
                else:
                    my_king[row[0]][col] = 1.0
            elif char == 'P':
                if not is_w_move:
                    enemy_pawns[row[0]][col] = 1.0
                else:
                    my_pawns[row[0]][col] = 1.0
            elif char == 'N':
                if not is_w_move:
                    enemy_knights[row[0]][col] = 1.0
                else:
                    my_knights[row[0]][col] = 1.0
            elif char == 'B':
                if not is_w_move:
                    enemy_bishops[row[0]][col] = 1.0
                else:
                    my_bishops[row[0]][col] = 1.0
            elif char == 'R':
                if not is_w_move:
                    enemy_rooks[row[0]][col] = 1.0
                else:
                    my_rooks[row[0]][col] = 1.0
            elif char == 'Q':
                if not is_w_move:
                    enemy_queens[row[0]][col] = 1.0
                else:
                    my_queens[row[0]][col] = 1.0
            elif char == 'K':
                if not is_w_move:
                    enemy_king[row[0]][col] = 1.0
                else:
                    my_king[row[0]][col] = 1.0
            else:
                # The char must be a number designating empty spaces
                col += int(char) - 1

            col += 1

    tensor = [my_pawns, my_knights, my_bishops, my_rooks, my_queens, my_king, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens, enemy_king]
    return tensor


if __name__=='__main__':
    mp.set_start_method('spawn')
    my_net = None
    command = None

    while True:
        while True:
            command = input(PROMPT)
            if command in ('1', '2', '3', '4', '5', '6', '7'):
                break

        if command == '1':
            my_net = load_net()
        elif command == '2':
            save_net(my_net)
        elif command == '3':
            my_net = net.Net().cuda()
        elif command == '4':
            train_net()
        elif command == '5':
            eval_pos(my_net)
        elif command == '6':
            change_rate(my_net)
        elif command == '7':
            validation_perf(my_net)
        else:
            print('Unrecognized command: {}'.format(command))
