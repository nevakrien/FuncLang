# FuncLang
my first ever "real" languge attempt. going with "everything is a function" similar to old javas "everyhing is a class" 

right now mostly a sketch of how things could look like. debating how to think of utf8 validation and loading into memory (right now leaning towards just load a u8 and validate utf8 at the end for names)

# perf debug
RUSTFLAGS="--emit asm -C llvm-args=-x86-asm-syntax=intel" cargo build --release