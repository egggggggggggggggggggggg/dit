#!/usr/bin/env bash

# Terminal Emulator Test Script
# Run: chmod +x term_test.sh && ./term_test.sh

clear

echo "=== Basic Output Test ==="
echo "Hello, world!"
printf "Printf works: %d %s\n" 42 "test"
sleep 1

echo -e "\n=== ANSI Color Test ==="
for i in {0..7}; do
  for j in {0..7}; do
    code=$((30 + j))
    bg=$((40 + i))
    printf "\e[%d;%dm %3d/%-3d \e[0m" "$code" "$bg" "$code" "$bg"
  done
  echo
done
sleep 1

echo -e "\n=== 256 Color Test ==="
for i in {0..255}; do
  printf "\e[48;5;%sm%3d\e[0m " "$i" "$i"
  if (( (i + 1) % 16 == 0 )); then echo; fi
done
sleep 1

echo -e "\n=== Cursor Movement Test ==="
echo "Line 1"
echo "Line 2"
echo "Line 3"
sleep 1
echo -e "\e[2A"   # Move cursor up 2 lines
echo -e "\e[31mOverwritten Line 2\e[0m"
sleep 1

echo -e "\n=== Screen Clear Test ==="
sleep 1
clear
echo "Screen cleared."

echo -e "\n=== Unicode Test ==="
echo "Unicode: ✓ ✔ ✕ ✖ ★ ☆ → ← ↑ ↓ ☺ ☹ 🚀 🌍"
sleep 1

echo -e "\n=== Input Test (press keys, 'q' to quit) ==="
while true; do
  read -rsn1 key
  printf "You pressed: %q\n" "$key"
  [[ $key == "q" ]] && break
done

echo -e "\n=== Resize Test ==="
echo "Resize your terminal now. Waiting 5 seconds..."
for i in {5..1}; do
  echo "$i..."
  sleep 1
done

echo -e "\n=== Scroll Test ==="
for i in {1..50}; do
  echo "Scrolling line $i"
done
sleep 1

echo -e "\n=== Alternate Screen Buffer Test ==="
echo "Switching to alternate buffer..."
sleep 1
tput smcup   # switch to alternate screen
echo "This is the alternate screen. It should not affect main scrollback."
sleep 2
tput rmcup   # return to normal screen

echo -e "\n=== Bell Test ==="
echo -e "\a(You may hear a bell sound)"
sleep 1

echo -e "\n=== Done ==="
