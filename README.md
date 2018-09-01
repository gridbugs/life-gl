# life-gl

[Conway's Game of Life](https://en.wikipedia.org/wiki/Conway's_Game_of_Life)
implemented in shaders. By default, each pixel corresponds to a game cell, and
each frame rendered will progress the game by a tick. You can increase cell size
and add a delay between frames to actually see the game in progress. There are
also options for changing the survive/resurrect parameters of the game, as well
as the colours of cells.

Press space to restart with a new random (but deterministic if you specified a
seed) starting point.


## Examples

```
./life-gl -x640 -y480 --cell-size 2
```

![Screenshot 0](/images/screenshot0.png)

```
./life-gl --alive-colour '#ff00ff' --dead-colour '#00ffff' -x640 -y480 --cell-size 2 --survive-min 4 --survive-max 8 --resurrect-min 5 --resurrect-max 5 --seed 42
```

![Screenshot 1](/images/screenshot1.png)

