parsort
=======

### Usage
```bash
$ parsort [cores] [input]
```

This is a Rust implementation of a parallel mergesort utility, splits
the given input into `cores` threads, each thread sorts a chunk of the array.
After all threads have finished, the main thread merges the sorted arrays into one.

To build, run `build.sh` on the `rack-mad-01` machine.
This script uses `udocker` to compile the code using the official `rust:1.50-slim` image.
