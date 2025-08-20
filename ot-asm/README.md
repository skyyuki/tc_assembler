# tc_ot_asm

The assembler for The OVERTURE Architecture in [Turing Complete](https://turingcomplete.game).

Thank you for making wonderful game!

## Usage

This is a command-line tool to assemble OVERTURE assembly files.

```sh
tc_ot_asm <input_file>
```

## Assembly Specifications

The label name is start with `@`.
And the label difinition is with end with `:`.

### Mnemonics
```
Register, I/O name for Copy Mode: REG0, REG1, REG2, REG3, REG4, REG5, IO(/OUT/IN)

Immediate Mode: LET [VALUE]
Caluclate Mode: OR, NAND, NOR, AND, ADD, SUB
Copy Mode     : COPY [DEST REGISTER/IO] [SOURCE REGISTER/IO]
Condition Mode: OFF, EQ, LS, LSEQ, ON, NEQ, GREQ, GR
```

OUT and IN is alias for IO.

### Example
```
let 1
copy reg2 reg0
@loop
copy reg1 in
add
copy out reg3
```

## Building

To build the project, you need to have Rust and Cargo installed.

```sh
cargo build --release
```

The executable will be located in `target/release/tc_ot_asm`.

## License

This project is dual-licensed under the Apache 2.0 and MIT licenses.
This is NOT applyed to document like README.md only for other source code files or setting files.
