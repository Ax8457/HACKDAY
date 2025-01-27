#!/bin/bash

chattr -R +i /app
/sbin/iptables -A INPUT -p tcp --dport 14456 -j DROP
knockd -D &
python app.py &
python flag-app_adfthrieydndsjlsfnl.py &

wait
