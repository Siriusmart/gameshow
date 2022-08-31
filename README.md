To get install, run the following command

(Rust and Cargo is required)

```bash
git clone https://github.com/siriusmart/gameshow && cd gameshow && cargo install --path . && cd ..
```

Help message can be displayed with `gameshow help`, but here is an example anyways

```bash
gameshow ./easy.txt leaderboard.txt 10 3 -1 player1 player2
```

Asked questions are marked with an x to prevent repeating the same questions, leaderboard.txt contains the leaderboard updated every time a question is answered.
