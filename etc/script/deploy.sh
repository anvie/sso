#!/usr/bin/env bash

HOST=103.60.183.23
WEB_HOME=/home/web
NAME=${PWD##*/}
TARGET_PATH=$WEB_HOME/sso/

set -e

function run_on_remate {
    ssh root@$HOST $@
}

scp target/release/$NAME root@$HOST:$TARGET_PATH

rsync -avzrhcP static root@$HOST:$TARGET_PATH

rsync -avzrhcP tmpl root@$HOST:$TARGET_PATH

run_on_remate chown -R www-data:www-data $WEB_HOME
