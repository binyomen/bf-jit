# BF JIT

A Rust implementation of the [Adventures in JIT compilation] series, where
interpreters and JITs are written for the [BF] esoteric programming language.
This language is very simple, containing only eight basic instructions, and so
is a good candidate for a basic JIT implementation.

Please see [Benchmarks](#benchmarks) for a performance comparison between all
implementations. This should demonstrate the advantages of following a JIT-like
approach when possible.

- [Supported platforms](#supported-platforms)
- [Implementations](#implementations)
  - [simpleinterp](#simpleinterp)
  - [opinterp](#opinterp)
  - [opinterp2](#opinterp2)
  - [opinterp3](#opinterp3)
  - [simplejit](#simplejit)
  - [opjit](#opjit)
- [Benchmarks](#benchmarks)
  - [Linux](#linux)
  - [Windows](#windows)
  - [macOS](#macos)
- [Docs](#docs)

## Supported platforms

| OS      | Architecture |
|---------|--------------|
| Linux   | x86-64       |
| Linux   | i686         |
| Linux   | aarch64      |
| Windows | x86-64       |
| macOS   | x86-64       |

## Benchmarking

We benchmark two [BF] programs: one which writes an image of a Mandelbrot set to
stdout, and another which factors a number passed to stdin. For the
factorization program we pass in the large prime 179424691.

We currently don't run benchmarks on aarch64 builds, even though those builds
are supported and tested in this repo. This is because the emulation we're
running them under is very slow and adds hours to the CI runtime. We can add
aarch64 benchmarking back once [#5631: Support Linux ARM64 build images] is
completed.

## Implementations

Code for each implementation can be found in the `vms` directory.

### simpleinterp

This loosely follows the implementation at [Adventures in JIT compilation § A
simple interpreter]. There are no optimizations here. We parse the input [BF]
program into a sequence of instructions (where each character either maps to an
instruction or a comment), and then interpret them in sequence. The destinations
of jumps are calculated at runtime as needed.

### opinterp

This loosely follows the implementation at [Adventures in JIT compilation
§ Optimized interpreter—take 1]. This adds the optimization over [simpleinterp]
that jump destinations are now calculated at compile-time. A jump table is
calculated that maps the position of a jump instruction to the position that it
jumps to.

We saw this provide around 35–50% speedups over [simpleinterp].

### opinterp2

This loosely follows the implementation at [Adventures in JIT compilation
§ Optimized interpreter—take 2]. This adds the optimization over [opinterp] that
repeated instructions are merged into a single instruction with a count. For
example, `Instruction::IncPtr` (represented in [BF] as `>`) now increments the
data pointer by the specified count; and `Instruction::DecData` (represented in
[BF] as `-`) now decrements the pointed-to data by the count. This way we don't
need a full iteration of the interpreter loop for each instance of a repeated
instruction. Instead, the multiple instructions are collapsed into a single
arithmetic operation.

We saw this provide around 50–70% speedups over [opinterp].

### opinterp3

This loosely follows the implementation at [Adventures in JIT compilation
§ Optimized interpreter—take 3]. This adds the optimization over [opinterp2]
that common sequences of [BF] (specifically `[>]`, `[<]`, `[+]`, `[-]`,
`[-<+>]`, `[->+<]`, and variations) are special-cased into new instructions.
`[>]` and `[<]` move the data pointer forward or backward by jumps of the count
of the instructions in the loop until a zero cell is reached. `[+]` and `[-]`
set the current cell to 0. `[-<+>]` and `[->+<]` add the current cell to the
cell as far forward or backward as the number of `>`/`<` characters as long as
the current cell isn't zero. The current cell is then set to zero. These again
reduce the number of times the interpreter loop must be run.

We saw this provide around 30–45% speedups over [opinterp2].

### simplejit

This loosely follows the implementation at [Adventures in JIT compilation
§ simpleasmjit—JIT with sane instruction encoding]. This is basically an
implementation of [opinterp], but rather than interpreting bytecode it compiles
the [BF] directly to machine code and runs it. It does this by translating each
[BF] instruction to an assembly instruction or sequence of instructions,
compiling the assembly to machine code with [dynasm-rs], writing the machine
code to a writable memory map, marking the memory map executable and read-only,
and calling the executable memory as if it were a function. The data pointer is
stored in a register and the [BF] memory is allocated by Rust.

This requires some `unsafe` Rust code, since we need to make sure the assembly
doesn't do anything unsound. For example, it must make sure to follow the
correct calling conventions for the platform it's compiled on. It's also worth
noting that this implementation (and that of [opjit] below) don't bounds-check
[BF] memory access. The [BF] spec does allow for undefined behavior when
accessing out-of-bounds memory, though, so this is OK for now.

We saw this provide around 20–50% speedups over [opinterp3].

### opjit

This loosely follows the implementation at [Adventures in JIT compilation
§ optasmjit—combining BF optimizations with a JIT]. This is basically an
implementation of [opinterp3] but following the same JIT strategy as
[simplejit]. As a result it has all of the [BF] optimizations of [opinterp3],
but also is compiled directly to machine code.

We saw this provide around 60–70% speedups over [simplejit].

## Benchmarks

### Linux

#### x86-64

##### Mandelbrot generator

![Linux x86-64 Mandelbrot generator perf graph](https://binyomen.github.io/bf-jit/img/linux-x86_64-mandelbrot.png)

##### Factorization

![Linux x86-64 factorization perf graph](https://binyomen.github.io/bf-jit/img/linux-x86_64-factor.png)

#### i686

##### Mandelbrot generator

![Linux i686 Mandelbrot generator perf graph](https://binyomen.github.io/bf-jit/img/linux-i686-mandelbrot.png)

##### Factorization

![Linux i686 factorization perf graph](https://binyomen.github.io/bf-jit/img/linux-i686-factor.png)

### Windows

#### x86-64

##### Mandelbrot generator

![Windows x86-64 Mandelbrot generator perf graph](https://binyomen.github.io/bf-jit/img/windows-x86_64-mandelbrot.png)

##### Factorization

![Windows x86-64 factorization perf graph](https://binyomen.github.io/bf-jit/img/windows-x86_64-factor.png)

### macOS

#### x86-64

##### Mandelbrot generator

![Linux x86-64 Mandelbrot generator perf graph](https://binyomen.github.io/bf-jit/img/macos-x86_64-mandelbrot.png)

##### Factorization

![Linux x86-64 factorization perf graph](https://binyomen.github.io/bf-jit/img/macos-x86_64-factor.png)

## Docs

- [bench docs]
- [simpleinterp docs]
- [opinterp docs]
- [opinterp2 docs]
- [opinterp3 docs]
- [simplejit docs]
- [opjit docs]

<!-- LINKS -->

<!-- GENERAL -->
[Adventures in JIT compilation]: https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-1-an-interpreter
[BF]: https://en.wikipedia.org/wiki/Brainfuck
[#5631: Support Linux ARM64 build images]: https://github.com/actions/runner-images/issues/5631

<!-- IMPLEMENTATIONS -->
[Adventures in JIT compilation § A simple interpreter]: https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-1-an-interpreter/#a-simple-interpreter
[Adventures in JIT compilation § Optimized interpreter—take 1]: https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-1-an-interpreter/#optimized-interpreter-take-1
[Adventures in JIT compilation § Optimized interpreter—take 2]: https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-1-an-interpreter/#optimized-interpreter-take-2
[Adventures in JIT compilation § Optimized interpreter—take 3]: https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-1-an-interpreter/#optimized-interpreter-take-3
[Adventures in JIT compilation § simpleasmjit—JIT with sane instruction encoding]: https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-2-an-x64-jit/#simpleasmjit-jit-with-sane-instruction-encoding
[Adventures in JIT compilation § optasmjit—combining BF optimizations with a JIT]: https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-2-an-x64-jit/#optasmjit-combining-bf-optimizations-with-a-jit

<!-- IMPLEMENTATION SECTIONS -->
[simpleinterp]: #simpleinterp
[opinterp]: #opinterp
[opinterp2]: #opinterp2
[opinterp3]: #opinterp3
[simplejit]: #simplejit
[opjit]: #opjit

<!-- DEPENDENCIES -->
[dynasm-rs]: https://github.com/CensoredUsername/dynasm-rs

<!-- DOCS -->
[bench docs]: https://binyomen.github.io/bf-jit/bench/
[simpleinterp docs]: https://binyomen.github.io/bf-jit/simpleinterp/
[opinterp docs]: https://binyomen.github.io/bf-jit/opinterp/
[opinterp2 docs]: https://binyomen.github.io/bf-jit/opinterp2/
[opinterp3 docs]: https://binyomen.github.io/bf-jit/opinterp3/
[simplejit docs]: https://binyomen.github.io/bf-jit/simplejit/
[opjit docs]: https://binyomen.github.io/bf-jit/opjit/

<!----------->
