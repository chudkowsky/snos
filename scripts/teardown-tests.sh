#!/bin/bash

shopt -s extglob

echo -e "\ncleaning cairo submodule...\n"
git submodule deinit -f .
git submodule update --init && cd cairo-lang && git checkout feat/sharding && cd ..

# remove compiled contracts
echo -e "\nremoving compiled contracts/programs...\n"
rm starkware
rm -rf build/!(os_latest.json)
