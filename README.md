### Sudoku TUI
This is a program running under command-line, with clean user interface.
It implement features:
  1. TODO! Solve sudoku
  2. TODO! Generate sudoku starting known numbers

## Basic keys
1. `hjkl` or arrow keys to move inside the sudoku grid
2. `q` for quit
3. number `1-9` to input num to current selected sudoku slot
4. number `0` to empty current selected sudoku slot
5. `s` for solve sudoku

## sudoku rules
Current sudoku use standard [3row*3col]*3row*3col grid

## for implement
Current implement a ratatui layout like this:
  1. upper is that sudoku grid
  2. a down bar to show all keys hint or error status.

And implement key events handle with crossterm, avoid windows OS repeat pressing bug(detect during keydown and keyup).
sudoku array is using array2d library as data structure
DO NOT implement  solve sudoku function, i will implement it.
DO NOT implement Generate sudoku feature.
over.
