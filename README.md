# frothy

A postfix expression language loosely inspired by Forth.

Save the following example to `area.fy` and run it with `$ frothy area.fy`

```
# ./area.fy

# define a function to compute the area of a circle with radius `r`
area {
    r r * PI *
} fn =

# define the radius of the circle
r 5 =

# the print function expects a variable `print_arg` to exist
print_arg area call =
print call
```

