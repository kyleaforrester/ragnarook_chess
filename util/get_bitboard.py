#!/usr/bin/env python3

import sys
import math
import itertools
import random

def index_to_hex(index_list):
    if (len(index_list) == 0):
        return hex(0)
    args = [2**int(i) for i in index_list]
    return hex(sum(args))

def integer_index_list(index_list):
    if (len(index_list) == 0):
        return 0
    args = [2**int(i) for i in index_list]
    return sum(args)

def knight_collisions():
    square_list = []
    for i in range(64):
        square_list.append(index_to_hex(knight_moves(i)))
    return square_list

def knight_moves(i):
    #i is the index our knight occupies
    #index_list will contain all indexes of valid move_spaces
    index_list = []

    #(1,2)
    if ((i+17) <= 63 and ((i+17)%8) - (i%8) == 1):
        index_list.append(i+17)
    #(2,1)
    if ((i+10) <= 63 and ((i+10)%8) - (i%8) == 2):
        index_list.append(i+10)
    #(2,-1)
    if ((i-6) >= 0 and ((i-6)%8) - (i%8) == 2):
        index_list.append(i-6)
    #(1,-2)
    if ((i-15) >= 0 and ((i-15)%8) - (i%8) == 1):
        index_list.append(i-15)
    #(-1,-2)
    if ((i-17) >= 0 and (i%8) - ((i-17)%8) == 1):
        index_list.append(i-17)
    #(-2,-1)
    if ((i-10) >= 0 and (i%8) - ((i-10)%8) == 2):
        index_list.append(i-10)
    #(-2,1)
    if ((i+6) <= 63 and (i%8) - ((i+6)%8) == 2):
        index_list.append(i+6)
    #(-1,2)
    if ((i+15) <= 63 and (i%8) - ((i+15)%8) == 1):
        index_list.append(i+15)
    return index_list

def bishop_collisions():
    square_list = []
    for i in range(64):
        square_list.append(index_to_hex(bishop_moves(i)))
    return square_list

def bishop_moves(i):
    #i is the index our bishop occupies
    #index_list will contain all indexes of valid move_spaces
    index_list = []
    #Northeast
    x = 1
    y = 8
    while ((i+x)%8 > i%8 and (i+x)%8 <= 6 and math.floor((i+y)/8) <= 6):
        index_list.append(i+x+y)
        x += 1
        y += 8

    #Southeast
    x = 1
    y = -8
    while ((i+x)%8 > i%8 and (i+x)%8 <= 6 and math.floor((i+y)/8) >= 1):
        index_list.append(i+x+y)
        x += 1
        y -= 8

    #Southwest
    x = -1
    y = -8
    while ((i+x)%8 < i%8 and (i+x)%8 >= 1 and math.floor((i+y)/8) >= 1):
        index_list.append(i+x+y)
        x -= 1
        y -= 8

    #Northwest
    x = -1
    y = 8
    while ((i+x)%8 < i%8 and (i+x)%8 >= 1 and math.floor((i+y)/8) <= 6):
        index_list.append(i+x+y)
        x -= 1
        y += 8
    return index_list

def rook_collisions():
    square_list = []
    for i in range(64):
        square_list.append(index_to_hex(rook_moves(i)))
    return square_list

def rook_moves(i):
    #i is the index our rook occupies
    #index_list will contain all indexes of valid move_spaces
    index_list = []
    #North
    y = 8
    while (math.floor((i+y)/8) <= 6):
        index_list.append(i+y)
        y += 8
    #South
    y = -8
    while (math.floor((i+y)/8) >= 1):
        index_list.append(i+y)
        y -= 8
    #East
    x = 1
    while (((i+x)%8) > i%8 and ((i+x)%8) <= 6):
        index_list.append(i+x)
        x += 1
    #West
    x = -1
    while (((i+x)%8) < i%8 and ((i+x)%8) >= 1):
        index_list.append(i+x)
        x -= 1
    return index_list

def king_collisions():
    square_list = []
    for i in range(64):
        index_list = []
        #North
        if (math.floor((i+8)/8) <= 7):
            index_list.append(i+8)
        #Northeast
        if (math.floor((i+9)/8) <= 7 and i%8 < (i+9)%8):
            index_list.append(i+9)
        #East
        if (i%8 < (i+1)%8):
            index_list.append(i+1)
        #Southeast
        if (math.floor((i-7)/8) >= 0 and i%8 < (i-7)%8):
            index_list.append(i-7)
        #South
        if (math.floor((i-8)/8) >= 0):
            index_list.append(i-8)
        #Southwest
        if (math.floor((i-9)/8) >= 0 and i%8 > (i-9)%8):
            index_list.append(i-9)
        #West
        if (i%8 > (i-1)%8):
            index_list.append(i-1)
        #Northwest
        if (math.floor((i+7)/8) <= 7 and i%8 > (i+7)%8):
            index_list.append(i+7)
        square_list.append(index_to_hex(index_list))
    return square_list

def w_pawn_move_collisions():
    square_list = []
    for i in range(64):
        index_list = []
        if (i < 56):
            index_list.append(i+8)
        square_list.append(index_to_hex(index_list))
    return square_list

def w_pawn_attack_collisions():
    square_list = []
    for i in range(64):
        index_list = []
        #Northeast
        if (i%8 < (i+9)%8 and math.floor((i+9)/8) <= 7):
            index_list.append(i + 9)
        #Northwest
        if (i%8 > (i+7)%8 and math.floor((i+7)/8) <= 7):
            index_list.append(i + 7)
        square_list.append(index_to_hex(index_list))
    return square_list

def b_pawn_move_collisions():
    square_list = []
    for i in range(64):
        index_list = []
        if (i > 7):
            index_list.append(i-8)
        square_list.append(index_to_hex(index_list))
    return square_list

def b_pawn_attack_collisions():
    square_list = []
    for i in range(64):
        index_list = []
        #Southeast
        if (i%8 < (i-7)%8 and math.floor((i-7)/8) >= 0):
            index_list.append(i - 7)
        #Southwest
        if (i%8 > (i-9)%8 and math.floor((i-9)/8) >= 0):
            index_list.append(i - 9)
        square_list.append(index_to_hex(index_list))
    return square_list

def print_array(array):
    for a in array:
        print(a)

def rook_occupation_moves(i, occupations):
    index_list = []
    #North
    y = 8
    while (math.floor((i+y)/8) <= 7):
        index_list.append(i+y)
        if ((i+y) in occupations):
            break
        y += 8
    #South
    y = -8
    while (math.floor((i+y)/8) >= 0):
        index_list.append(i+y)
        if ((i+y) in occupations):
            break
        y -= 8
    #East
    x = 1
    while (((i+x)%8) > i%8 and ((i+x)%8) <= 7):
        index_list.append(i+x)
        if ((i+x) in occupations):
            break
        x += 1
    #West
    x = -1
    while (((i+x)%8) < i%8 and ((i+x)%8) >= 0):
        index_list.append(i+x)
        if ((i+x) in occupations):
            break
        x -= 1
    return index_list

def bishop_occupation_moves(i, occupations):
    #i is the index our bishop occupies
    #index_list will contain all indexes of valid move_spaces
    index_list = []
    #Northeast
    x = 1
    y = 8
    while ((i+x)%8 > i%8 and (i+x)%8 <= 7 and math.floor((i+y)/8) <= 7):
        index_list.append(i+x+y)
        if ((i+x+y) in occupations):
            break
        x += 1
        y += 8

    #Southeast
    x = 1
    y = -8
    while ((i+x)%8 > i%8 and (i+x)%8 <= 7 and math.floor((i+y)/8) >= 0):
        index_list.append(i+x+y)
        if ((i+x+y) in occupations):
            break
        x += 1
        y -= 8

    #Southwest
    x = -1
    y = -8
    while ((i+x)%8 < i%8 and (i+x)%8 >= 0 and math.floor((i+y)/8) >= 0):
        index_list.append(i+x+y)
        if ((i+x+y) in occupations):
            break
        x -= 1
        y -= 8

    #Northwest
    x = -1
    y = 8
    while ((i+x)%8 < i%8 and (i+x)%8 >= 0 and math.floor((i+y)/8) <= 7):
        index_list.append(i+x+y)
        if ((i+x+y) in occupations):
            break
        x -= 1
        y += 8
    return index_list


def rook_magic_numbers():
    bits = 0xffffffffffffffff
    magic_nums = []
    placed_moves = []
    for i in range(64):
        collisions = rook_moves(i)
        occupied_move_map = {}
        occupied_move_map[0] = integer_index_list(rook_occupation_moves(i, []))
        occupied_sets = []
        for j in range(1, len(collisions)+1):
            combos = list(itertools.combinations(collisions, j))
            for combo in combos:
                occupied_sets.append(combo)
        #Set the correct answers for each occupied_set
        for occupied_set in occupied_sets:
            occupied_move_map[integer_index_list(occupied_set)] = integer_index_list(rook_occupation_moves(i, occupied_set))
        
        hash_bit_size = 12
        attempts = 0
        perfect_hash = False
        new_rand = 100
        new_array = []
        while (not perfect_hash):
            new_rand = random.randint(1, bits) & random.randint(1, bits)
            attempts += 1
            new_array = [0]*(2**hash_bit_size)
            perfect_hash = True
            item_count = 0
            for item in occupied_move_map.items():
                item_count += 1
                index = ((item[0] * new_rand) & bits) >> (64-hash_bit_size)
                if (new_array[index] == 0 or new_array[index] == item[1]):
                    new_array[index] = item[1]
                else:
                    perfect_hash = False
                    if (attempts % 1000 == 0):
                        #print('Attempt {} failed on the {} item.'.format(attempts, item_count))
                        pass
                    break
        magic_nums.append(new_rand)
        placed_moves.append(new_array)

    return magic_nums, placed_moves

def bishop_magic_numbers():
    bits = 0xffffffffffffffff
    magic_nums = []
    placed_moves = []
    for i in range(64):
        collisions = bishop_moves(i)
        occupied_move_map = {}
        occupied_move_map[0] = integer_index_list(bishop_occupation_moves(i, []))
        occupied_sets = []
        for j in range(1, len(collisions)+1):
            combos = list(itertools.combinations(collisions, j))
            for combo in combos:
                occupied_sets.append(combo)
        #Set the correct answers for each occupied_set
        for occupied_set in occupied_sets:
            occupied_move_map[integer_index_list(occupied_set)] = integer_index_list(bishop_occupation_moves(i, occupied_set))
        
        hash_bit_size = 9 
        attempts = 0
        perfect_hash = False
        new_rand = 100
        new_array = []
        while (not perfect_hash):
            new_rand = random.randint(1, bits) & random.randint(1, bits)
            attempts += 1
            new_array = [0]*(2**hash_bit_size)
            perfect_hash = True
            item_count = 0
            for item in occupied_move_map.items():
                item_count += 1
                index = ((item[0] * new_rand) & bits) >> (64-hash_bit_size)
                if (new_array[index] == 0 or new_array[index] == item[1]):
                    new_array[index] = item[1]
                else:
                    perfect_hash = False
                    if (attempts % 1000 == 0):
                        #print('Attempt {} failed on the {} item.'.format(attempts, item_count))
                        pass
                    break
        magic_nums.append(new_rand)
        placed_moves.append(new_array)

    return magic_nums, placed_moves

def print_rook_magic_numbers():
    
    magic_nums, placed_moves = rook_magic_numbers()

    magic_num_string = ','.join([str(num) for num in magic_nums])

    print(magic_num_string)
    print('')

    placed_moves_strings = ['{ ' + ','.join([hex(move) for move in moves]) + ' }' for moves in placed_moves]

    for move in placed_moves_strings:
        print(move)

def print_bishop_magic_numbers():
    
    magic_nums, placed_moves = bishop_magic_numbers()

    magic_num_string = ','.join([str(num) for num in magic_nums])

    print(magic_num_string)
    print('')

    placed_moves_strings = ['{ ' + ','.join([hex(move) for move in moves]) + ' }' for moves in placed_moves]

    for move in placed_moves_strings:
        print(move)

def print_bitboard(integer):
    for y in range(7, -1, -1):
        for x in range(8):
            index = 8*y + x
            if (integer & (2**index)):
                print('1', end='')
            else:
                print('0', end='')
        print('')

def print_square_lookup_table():
    for col in '12345678':
        for i in range(8):
            print(col, end=',')
    print()
    for i in range(8):
        for row in 'abcdefgh':
            print(row, end=',')

def basic_eval_pawn_tables(is_white):
    
    edge_penalty = 0.8
    if (is_white):
        for i in range(64):
            row = math.floor(i/8)
            value = 1.2**((row - 1)/5)
            if (i%8 == 0 or i%8 == 7):
                value = value * edge_penalty
            if (row == 6):
                value += 1
            print('{0:.2f}'.format(value), end=',')
    else:
        for i in range(64):
            row = math.floor(i/8)
            value = 1.2**((6 - row)/5)
            if (i%8 == 0 or i%8 == 7):
                value = value * edge_penalty
            if (row == 1):
                value += 1
            print('{0:.2f}'.format(value), end=',')

def basic_eval_mid_king_tables():
    for i in range(64):
        row = math.floor(i/8)
        col = i%8
        value = 0

        if (row == 0 or row == 7):
            value += 2
        elif (row == 1 or row == 6):
            value += 1.7
        elif (row == 2 or row == 5):
            value += 0.3
        elif (row == 3 or row == 4):
            pass

        if (col == 0 or col == 7):
            value += 2
        elif (col == 1 or col == 6):
            value += 1.7
        elif (col == 2 or col == 5):
            value += 0.3
        elif (col == 3 or col == 4):
            pass

        print('{0:.2f}'.format(value), end=',')

def basic_eval_end_king_tables():
    for i in range(64):
        row = math.floor(i/8)
        col = i%8
        value = 0

        if (col == 3 or col == 4):
            value += 0.4
        elif (col == 2 or col == 5):
            value += 0.3
        elif (col == 1 or col == 6):
            value += 0.1

        print('{0:.2f}'.format(value), end=',')

def table_translate_lsb_index_my_index():
    for y in range(8):
        for x in range(7, -1, -1):
            print('{},'.format(8*y + x), end='')

def eval_to_cp(win_percent):
    if (win_percent > 0.5):
        cp = round(math.sqrt((20000*win_percent - 10000)/(1 - win_percent)))
    elif (win_percent < 0.5):
        temp_eval = 1 - win_percent
        cp = round(math.sqrt((20000*temp_eval - 10000)/(1 - temp_eval)))
    else:
        cp = 0

    return cp

def cp_to_eval(cp):
    if (cp > 0):
        win_percent = (cp**2 + 10000)/(cp**2 + 20000)
    elif (cp < 0):
        win_percent = 1 - (cp**2 + 10000)/(cp**2 + 20000)
    else:
        win_percent = 0.5

    return win_percent

