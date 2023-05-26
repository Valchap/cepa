#!/bin/bash

# Create a new session and detach from it
tmux new-session -d -s mysession

# Split the window vertically
tmux split-window -v

# Split the both panes horizontally
tmux select-pane -t 0
tmux split-window -h -c '#{pane_current_path}'
tmux select-pane -t 2
tmux split-window -h -c '#{pane_current_path}'

# Start cepa
  # Node 1
tmux select-pane -t 0
tmux send-keys 'docker-compose run node1' C-m
tmux send-keys 'clear' C-m
sleep 2
tmux send-keys 'add 192.168.1.11' C-m

  # Node 2
tmux select-pane -t 1
tmux send-keys 'docker-compose run node2' C-m
tmux send-keys 'clear' C-m
sleep 2
tmux send-keys 'add 192.168.1.12' C-m

  # Node 3
tmux select-pane -t 2
tmux send-keys 'docker-compose run node3' C-m
tmux send-keys 'clear' C-m
sleep 2
tmux send-keys 'add 192.168.1.13' C-m

  # Node 4
tmux select-pane -t 3
tmux send-keys 'docker-compose run node4' C-m
tmux send-keys 'clear' C-m
sleep 2 
tmux send-keys 'add 192.168.1.14' C-m

tmux select-pane -t 0

# Attach to the new session
tmux attach -t mysession
