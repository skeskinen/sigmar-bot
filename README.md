Bot that plays Sigmar's Garden minigame from Zachtronics's Opus Magnum.

Mostly written just to learn Rust.

Basic steps:

1. Take screenshot of main display
2. Try to find game board
3. OCR to determine what marble is where
4. Solve with basic dfs, use zobrist table to make perf less terrible
5. Input solution with Win32 mouse API
