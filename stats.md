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

# use fxhash

```
   195,290,306,005      cycles

      50.296644900 seconds time elapsed

      42.038223000 seconds user
       8.217397000 seconds sys
```

# stream join

```
   188,417,728,589      cycles

      49.339059871 seconds time elapsed

      42.746668000 seconds user
       6.573949000 seconds sys
```

# mmap files

```
   144,243,649,624      cycles

      39.624025050 seconds time elapsed

      33.628653000 seconds user
       5.944337000 seconds sys
```

# use arrays instead of vec

```
    95,035,983,128      cycles

      25.052302256 seconds time elapsed

      19.580007000 seconds user
       5.456657000 seconds sys
```

# smallvec

```
    86,108,837,399      cycles                                                             

      22.513780147 seconds time elapsed

      18.381462000 seconds user
       4.127430000 seconds sys
```

# smallvec size 4

```
    79,297,251,479      cycles                                                             

      20.754173519 seconds time elapsed

      18.158947000 seconds user
       2.587280000 seconds sys
```

