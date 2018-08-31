# life-gl

[Conway's Game of Life](https://en.wikipedia.org/wiki/Conway's_Game_of_Life)
implemented in shaders. Supports a number of functional and aesthetic
configurations. Press space to restart.

## Examples

```
./life-gl -x640 -y480 --cell-size 2
```

![Screenshot 0](/images/screenshot0.png)

```
./life-gl  --alive-colour '#ff00ff' --dead-colour '#00ffff' -x640 -y480 --cell-size 2 --survive-min 4 --survive-max 8 --resurrect-min 5 --resurrect-max 5 --seed 42
```

![Screenshot 1](/images/screenshot1.png)

