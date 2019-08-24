# Sokoban Solver
```
 .M"""bgd           `7MM                *MM
,MI    "Y             MM                 MM
`MMb.      ,pW"Wq.    MM  ,MP' ,pW"Wq.   MM,dMMb.   ,6"Yb.  `7MMpMMMb.
  `YMMNq. 6W'   `Wb   MM ;Y   6W'   `Wb  MM    `Mb 8)   MM    MM    MM
.     `MM 8M     M8   MM;Mm   8M     M8  MM     M8  ,pm9MM    MM    MM
Mb     dM YA.   ,A9   MM `Mb. YA.   ,A9  MM.   ,M9 8M   MM    MM    MM
P"Ybmmd"   `Ybmd9'  .JMML. YA.,,Ybmd9'   P^YbmdP'  `Moo9^Yo..JMML  JMML.
         .M"""bgd           `7MM
        ,MI    "Y             MM
        `MMb.      ,pW"Wq.    MM  `7M'   `MF' .gP"Ya  `7Mb,od8
          `YMMNq. 6W'   `Wb   MM    VA   ,V  ,M'   Yb   MM' "'
        .     `MM 8M     M8   MM     VA ,V   8M""""""   MM
        Mb     dM YA.   ,A9   MM      VVV    YM.    ,   MM
        P"Ybmmd"   `Ybmd9'  .JMML.     W      `Mbmmd' .JMML.
```

# Solver Approach

The approach to solve sokoban is to reverse the game, we pull the boxes instead of pushing.
This will fix some typical states where setting a box in a specific place will make it impossible to solve the level, i.e. when a box is in a corner.

Although, this is great, it also adds other complexities:

* The player may not have a place to start if it starts over a whole (when we swap wholes with boxes, the player can't be over a box)
* We need to try all player zones since when we swap boxes with wholes, we don't know where the player should be, potentially needing to try to solve the level on all player zones. i.e.:

```
  # # # # # #
  # .   # @ # <- Zone 1
  #     $   #
  # # # # # #

  # # # # # #
  # . @ #   # <- Zone 2
  #     $   #
  # # # # # #
```

## Keep moving the same box

If we successfully move a box, we keep moving the same box in the next steps. We will still try all other steps but give priority to the current one.

## States

A state is a specific moment in the sokoban level solution, it's given by:

#### Box current positions
  Where the boxes are in the given state

#### Player Zone

The player current zone. A player can be in a situation where it can't move to a different zone of a game. 

```
  # # # # # #
  #     # @ #
  #     $   #
  # # # # # #
```

We have 2 zones in this example. Because of these zones, a state will be different with the same box positions but a different player zone.

#### Depth

Depth is the number of boxes that have been pushed to reach the current state. If we reach a given state in less steps than before,
we consider it a better way to reach that state and keep going. If we reach that state with more steps, we stop pursuing that tree.

# Run with logs

Add log level environment variable before running

```
RUST_LOG=debug cargo run --release 0709000001110111111411100002501100010001111113101000010001000011111
RUST_LOG=info cargo run --release 0709000001110111111411100002501100010001111113101000010001000011111
```

# Run all levels asynchronously

```
cat levels/microban_num.soko | xargs -n1 -P $(sysctl -n hw.physicalcpu) cargo run --release
```

# TODO

- [ ] If a box can't be moved to any direction, stop trying to move it to different goals
- [ ] If there is a box that can't be moved because of walls (not boxes that may be moved later on), we should stop trying that tree
- [ ] Make the solver procedural
- [ ] Make it possible to resolve sokoban without reversing, instead, just adding constraints to stop trying an option tree when a box is stuck.
- [ ] Implement method to find optimal solution
- [ ] Implement recorder to count certain events (i.e. box swaps, box pushes, etc) 
- [ ] Implement exporter of solution
