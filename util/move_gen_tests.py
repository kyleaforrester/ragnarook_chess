import chess
import sys

fd = open(sys.argv[1]).readlines()

fd = [l.strip() for l in fd]

for l in fd:
    board = chess.Board(l)
    moves = []
    if board.legal_moves:
        print('        scenarios.push(("{}".to_string(), vec!['.format(l), end='');
        for m in board.legal_moves:
            board.push(m)
            moves.append('"{}".to_string()'.format(board.fen()));
            board.pop()
        print("{}]));".format(', '.join(moves)))
