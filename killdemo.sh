#!/bin/bash

curl -s https://cepa.ech0.ch/reset > /dev/null
docker stop $(docker ps -aq)
docker rm $(docker ps -aq)
tmux kill-server
