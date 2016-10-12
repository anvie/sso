#!/usr/bin/env bash

CURDIR=`dirname $0`
. $CURDIR/includes.sh



echo -n "stop running service... "
run_on_remate svc -d /etc/service/sso
echo "[done]"

echo -n "updating... "
scp target/release/$NAME root@$HOST:$TARGET_PATH
rsync -avzrhcP static root@$HOST:$TARGET_PATH
rsync -avzrhcP tmpl root@$HOST:$TARGET_PATH
echo "[done]"

echo -n "setting permissions... "
run_on_remate chown -R www-data:www-data $WEB_HOME
echo "[done]"

echo -n "starting service..."
run_on_remate svc -u /etc/service/sso
echo "[done]"
