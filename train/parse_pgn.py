#!/usr/bin/env python3
import chess.pgn
import sys

def convert_ply_to_float(ply):
    return (1.015)**(-ply - 46.56) + 0.5

pgn = open(sys.argv[1])

game = chess.pgn.read_game(pgn)
while game is not None:
    total_plies = int(game.headers["PlyCount"])
    is_draw = False
    if game.headers["Result"] == "1/2-1/2":
        is_draw = True
    for enum_node in enumerate(game.mainline()):
        board = enum_node[1].board()
        plies_to_end = total_plies - enum_node[0] - 1
        if not is_draw and plies_to_end % 2 == 0:
            #Loser
            print("{},{},{}".format(board.fen(), -plies_to_end, 1-convert_ply_to_float(plies_to_end)))
        elif not is_draw and plies_to_end % 2 != 0:
            #Winner
            print("{},{},{}".format(board.fen(), plies_to_end, convert_ply_to_float(plies_to_end)))
        else:
            #Draw
            print("{},{},{}".format(board.fen(), plies_to_end, 0.5))
    game = chess.pgn.read_game(pgn)
