# Baseline

```
   440,280,395,251      cycles

     120.942325499 seconds time elapsed

     109.870761000 seconds user
      10.984278000 seconds sys
```

# Compiler optimizations

```
   429,701,405,581      cycles

     119.400103359 seconds time elapsed

     108.286886000 seconds user
      11.028146000 seconds sys
```

# join file by file

```
   434,880,577,583      cycles

     116.514198733 seconds time elapsed

     107.258709000 seconds user
       9.169734000 seconds sys
```

# use byte vectors instead of strings

```
   426,439,974,509      cycles

     114.846540260 seconds time elapsed

     105.708734000 seconds user
       9.062233000 seconds sys
```

# switch to hash join

```
   352,024,313,086      cycles

      97.007805235 seconds time elapsed

      86.545248000 seconds user
      10.397422000 seconds sys
```

# simplify

```
   329,846,796,589      cycles

      88.845377556 seconds time elapsed

      77.898300000 seconds user
      10.889052000 seconds sys
```

# use refs instead of cloning

```
   234,790,780,618      cycles

      62.512477349 seconds time elapsed

      54.184855000 seconds user
       8.291234000 seconds sys
```
