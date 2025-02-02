# Chessticot
This is a TUI chess game written in rust.
My hope is to one day use the chess logic to build one or more chess engines "from first principles".  

## Current Advancement
- All chess rules are implemented, with partial test coverage and known bugs  
- Proof of concept engines are available, that play random moves and crash the program when no move is legal  
- The UI allows you to pick a color and the other color will be played by the latest generation engine (dumb)  
- The game is not yet able to detect a stalemate and will likely crash in that case figure  

## Controls
- In main menu use left and right arrow keys to select your color, enter to confirm  
- In the game use arrow keys or hjkl to move the cursor and space to select a piece/confirm a move, escape to cancel a move, tab to change which piece you want to promote to  
- On all screens hit 'q' to quit  

## Known bugs
- Cannot detect stalemate
- You can castle despite moving king/rook so long as they are in their starting location  
