## RJAVA

`RJAVA` is JVM made for fun on Rust. 

**Very. Limited. JVM** 

However, it may do some tricks other JVMs cannot do.

### How to use

```
cd examples
cargo build --release
javac YourClass.java
cargo run --release YourClass
```


### Tail Recursion optimization

See `examples/TailRecursion.java`

```
cd examples
javac TailRecursion.java
cargo run --release TailRecursion
```

Check timings and `rjava.log` - it will contain dump of stack and heap.

Annotation `@RVM.TailRecursion` is specially processed by RJAVA. It turns recursive call into 'loop'.


### Memoization
See `examples/Memorize.java`

```
cd examples
javac Memorize.java
cargo run --release Memorize
```

Check timings and `rjava.log`.

Annotation `@RVM.Mem` leads to caching of method call results using
`io.github.rvm.MemEntry` class. Instances of that class are created internally
when needed, you may see it in heap.

### AutoFree

See `examples/AutoFree.java`

```
cd examples
javac AutoFree.java
cargo run --release AutoFree
```

Check output and `rjava.log`.

`RJAVA` does not have GC. But when method is annotated with `@RVM.AutoFree`, then all objects
instantiated with `new` in this stack frame and all deeper frames will be automatically removed from memory.

It means that references to objects/arrays could be passed "down" through stack, but not "up" (as they became invalid).


### What is the day today?

```
  ______________________________
 '                              ' 
( It's Wednesday, my Java dudes! )
 '                              ' 
  ______________________________
      \
        oO)-.
        /__  _\
        \  \(  |
        \__|\ {
        '  '--'
```