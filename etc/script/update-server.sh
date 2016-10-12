#!/usr/bin/env bash

CURDIR=`dirname $0`
. $CURDIR/includes.sh


cargo build --release

bash $CURDIR/deploy.sh
