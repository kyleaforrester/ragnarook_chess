
c-chess-cli -each tc=10+0.2 option.Threads=1 \
    -engine cmd=stockfish_11 name=stockfish_11 \
    -engine cmd=stockfish_12 name=stockfish_12 \
    -engine cmd=stockfish_13 name=stockfish_13 \
    -engine cmd=stockfish_14 name=stockfish_14 \
    -engine cmd=stockfish_14.1 name=stockfish_14.1 \
    -engine cmd=stockfish_15 name=stockfish_15 \
    -engine cmd=stockfish_15.1 name=stockfish_15.1 \
    -engine cmd=stockfish_16 name=stockfish_16 \
    -engine cmd=stockfish_16.1 name=stockfish_16.1 \
    -engine cmd=stockfish_17 name=stockfish_17 \
    -engine cmd=stockfish_17.1 name=stockfish_17.1 \
    -engine cmd=stockfish name=stockfish \
    -games 50 -concurrency 5 -openings file=4moves_noob.epd order=random \
    -draw count=8 score=10 number=40 -pgn ./pgns/$(date +%d%h%Y_%H%M%S).pgn 1

