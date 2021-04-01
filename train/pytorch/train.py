#!/usr/bin/env python3

import net
import random
import sys
import os.path
import torch
import multiprocessing as mp
import gc

prompt = '''Select an option:
    1) Load net
    2) Save net
    3) New net
    4) Train net
    5) Evaluate position
    6) Change learning rate
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
    inputs, labels = (None, None)
    history = []

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
            inputs, labels = q.get()
            p.join()

        # Begin parsing the next file
        q = mp.Queue()
        p = mp.Process(target=read_file, args=(next_file_name, q))
        p.start()

        # Spend time training
        history.append(my_net.train_file(inputs, labels))

        # Remove old history
        history = history[-20:]

        # Check if we are not making progress over 10 epoch span
        if sum(history[:10]) < sum(history[-10:]):
            # Reduce learning rate
            new_rate = my_net.learning_rate / 2
            print('Reducing learning rate from {} to {}'.format(my_net.learning_rate, new_rate))
            my_net.learning_rate = new_rate
            for g in my_net.optimizer.param_groups:
                g['lr'] = new_rate


        # Free memory
        del inputs
        del labels
        gc.collect()

        # Pick up the next inputs and labels
        inputs, labels = q.get()
        p.join()



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
    labels = [[float(l[2])] for l in samples]
    q.put((inputs, labels))


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

    my_pawns = [[0.0]*8]*8
    my_knights = [[0.0]*8]*8
    my_bishops = [[0.0]*8]*8
    my_rooks = [[0.0]*8]*8
    my_queens = [[0.0]*8]*8
    my_king = [[0.0]*8]*8
    enemy_pawns = [[0.0]*8]*8
    enemy_knights = [[0.0]*8]*8
    enemy_bishops = [[0.0]*8]*8
    enemy_rooks = [[0.0]*8]*8
    enemy_queens = [[0.0]*8]*8
    enemy_king = [[0.0]*8]*8

    # Rotate the board if it is black's move
    if not is_w_move:
        position = position[::-1]

    for row in enumerate(position.split('/')):
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

    return [my_pawns, my_knights, my_bishops, my_rooks, my_queens, my_king, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens, enemy_king]


if __name__=='__main__':
    mp.set_start_method('spawn')
    my_net = None
    command = None

    while True:
        while True:
            command = input(prompt)
            if command in ('1', '2', '3', '4', '5', '6'):
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
        else:
            print('Unrecognized command: {}'.format(command))
