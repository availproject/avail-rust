#!/bin/bash
set -e

cd ./examples

OP="$1"
for file in `ls -d */`
do
    if [ "$file" == "code_doc/" ]; then
        continue
    fi
    
    if [ "$OP" == "check" ]; then
        cd "$file" && cargo check && cd ./../.
    fi
    
    if [ "$OP" == "run" ]; then
        cd "$file" && cargo run && cd ./../.
    fi
    
    if [ "$OP" == "clean" ]; then
        cd "$file" && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
    fi
    
    if [ "$OP" == "fmt" ]; then
        cd "$file" && cargo +nightly fmt && cd ./../.
    fi
done
