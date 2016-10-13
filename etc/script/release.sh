#!/usr/bin/env bash

CURDIR=`dirname $0`
. $CURDIR/includes.sh

VERSION=`cat $CURDIR/../../VERSION`

echo "Current version: $VERSION"
echo -n "Next version: "

read next_version

if [ "$next_version" == "$VERSION" ]; then
    echo "Aborted. Next version is same as current version"
fi

echo $next_version > $CURDIR/../../VERSION

echo "Version updated."

echo -n "Generate build.rs ? [y/n] "

read generate_build_rs

if [ "$generate_build_rs" == "y" ]; then
    make src/build.rs
    echo "build.rs updated."
fi

echo "done."