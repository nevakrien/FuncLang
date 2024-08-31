# FuncLang
my first ever "real" languge attempt. going with "everything is a function" similar to old javas "everyhing is a class" 
so arrays are just functions... that return %out_of_bounds for anything out of bounds. hopefully the compiler can optimize it away.

the languge is safe and GCed. and it probably wants to be JIT compiled with an automatically parallel runtime.
I am considering making the runtime start runing before full parsing of the text is done. this is fine to do because we are pure functional.

# Dev Log

by deafualt everything is implemented 100% safe. I had unsafe mode but there wasnt a real performance gain and it does not work well with the projects goals. the refcell on diagnostics apears to be fine since every lock on it is a print anyway. tested with unsafe to be sure. 

right now this is just the lexer going to work up to the parser.
there is an obvious optimization of spliting things to lines and then parse with an atomic work stealing queue.
spliting based on parenthesis would work nicely as well. also can be put into a work stealing queue.

since this is going to be an interpeter to a pure functional languge parallalizing things can work well.
so we can run almost everything in parallel which is probably the first optimization worth looking into.

would look at the array stuff first mostly for the fun

# perf debug
RUSTFLAGS="--emit asm -C llvm-args=-x86-asm-syntax=intel" cargo build --release

cargo run --no-default-features --features unsafe_mode
