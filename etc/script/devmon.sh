#!/usr/bin/env bash
# this script require inotify
# if you're Debian based Linux, do: apt-get install inotify-tools

NAME=${PWD##*/}

function run_it {
    if [ -f ./target/debug/$NAME ]; then
        ./target/debug/$NAME & echo $! > run.pid
    fi
}

function kill_it {
    if [ -f run.pid ]; then
        kill `cat run.pid`
        rm -f run.pid
    fi
}

function compile_it {
    cargo build
}

function ctrl_c() {
    echo "quiting..."
    kill_it
}

trap ctrl_c INT

compile_it
run_it

while inotifywait -e modify -r tmpl/ src/; do
    kill_it
    compile_it
    run_it
done

kill_it
