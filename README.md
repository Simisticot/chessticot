# Chessticot
This is a TUI chess game written in rust.
I am using the chess logic to build (for now rudimentary) chess engines "from first principles".  

## Current Advancement
- All chess rules are implemented, with very partial test coverage and likely bugs  
- Proof of concept engines are available, that play random moves and crash the program when no move is legal  
- The UI allows you to pick a color and what engine to play against  

## Controls
- In main menu use left and right arrow keys to select your color, enter to confirm  
- In the game use arrow keys or hjkl to move the cursor and space to select a piece/confirm a move, escape to cancel a move, tab to change which piece you want to promote to  
- On all screens hit 'q' to quit  


## Plans 
I plan to keep adding smarter and smarter engines iteratively while still having earlier ones accessible, I like the idea of being able to contrast very simple heuristics to complex ones.  
I plan to split this project into the tui chess game and a library containing just the chess logic and engines. I hope to then implement a different, web-based ui using the library to make the game playable in the browser.  
