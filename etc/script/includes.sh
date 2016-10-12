#!/usr/bin/env bash


set -e

HOST=103.60.183.23
WEB_HOME=/home/web
NAME=${PWD##*/}
TARGET_PATH=$WEB_HOME/sso/

function run_on_remate {
    ssh root@$HOST $@
}
