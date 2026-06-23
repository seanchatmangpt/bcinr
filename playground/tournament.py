import subprocess
import time
import chess

def start_engine(cmd):
    return subprocess.Popen(
        cmd,
        universal_newlines=True,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=1
    )

def send_command(process, cmd):
    process.stdin.write(cmd + "\n")
    process.stdin.flush()

def read_bestmove(process):
    while True:
        line = process.stdout.readline().strip()
        if line.startswith("bestmove"):
            return line.split(" ")[1]

def play_game(engine1_cmd, engine2_cmd, movetime=1000):
    e1 = start_engine(engine1_cmd)
    e2 = start_engine(engine2_cmd)
    
    send_command(e1, "uci")
    send_command(e2, "uci")
    time.sleep(0.5)
    
    board = chess.Board()
    moves = []
    
    while not board.is_game_over():
        current_engine = e1 if board.turn == chess.WHITE else e2
        
        pos_cmd = "position startpos"
        if moves:
            pos_cmd += " moves " + " ".join(moves)
            
        send_command(current_engine, pos_cmd)
        send_command(current_engine, f"go movetime {movetime}")
        
        bestmove = read_bestmove(current_engine)
        if bestmove == "(none)" or bestmove == "0000":
            break
            
        moves.append(bestmove)
        board.push_uci(bestmove)
        print(f"{'White' if board.turn == chess.BLACK else 'Black'} plays: {bestmove}")
        
    e1.terminate()
    e2.terminate()
    
    result = board.result()
    print(f"Game over! Result: {result}")
    return result

if __name__ == "__main__":
    print("Starting BCINR (White) vs Stockfish (Black)")
    play_game(["/Users/sac/bcinr/target/release/bcinr_uci"], ["/opt/homebrew/bin/stockfish"], movetime=100)
    
    print("Starting Stockfish (White) vs BCINR (Black)")
    play_game(["/opt/homebrew/bin/stockfish"], ["/Users/sac/bcinr/target/release/bcinr_uci"], movetime=100)
