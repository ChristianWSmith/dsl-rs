#!/bin/bash

if ! which footclient > /dev/null || ! which fish > /dev/null || ! which figlet > /dev/null || ! which lolcat > /dev/null || ! swaymsg > /dev/null
then
    exit 1
fi

workspace=$(swaymsg -t get_workspaces | jq '.[] | select(.focused==true) | .name' | sed "s/\"//g")

swaymsg workspace demo

footclient fish -c "figlet A | lolcat -F 0.4; read -p ''" & pidA=$!
footclient fish -c "figlet B | lolcat -F 0.4; read -p ''" & pidB=$!
footclient fish -c "figlet C | lolcat -F 0.4; read -p ''" & pidC=$!
footclient fish -c "figlet D | lolcat -F 0.4; read -p ''" & picD=$!
footclient fish -c "figlet E | lolcat -F 0.4; read -p ''" & picE=$!

wait $pidA
wait $pidB
wait $pidC
wait $pidD
wait $pidE

swaymsg workspace $workspace
