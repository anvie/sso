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

echo -n "Apply version ? [y/n] "

read apply_version

if [ "$apply_version" == "y" ]; then
    make version
    echo "version applied."
fi

echo -n "bump version ? [y/n] "
read to_commit

if [ "$to_commit" == "y" ]; then
    git commit -am "bump version $next_version"
    echo -n "tag version ? [y/n] "
    read tag_version
    if [ "$tag_version" == "y" ]; then
        git tag "v$next_version"
    fi
fi

echo "done."
