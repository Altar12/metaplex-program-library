#!/bin/bash

function exists_in_list() {
    LIST=$1
    DELIMITER=$2
    VALUE=$3
    echo $LIST | tr "$DELIMITER" '\n' | grep -F -q -x "$VALUE"
}

input=$1

programs="auction-house auctioneer bubblegum candy-machine candy-machine-core fixed-price-sale gumdrop hydra nft-packs token-entangler token-metadata trifle"

mkdir -p test-programs

if exists_in_list "$programs" " " $input; then
    echo "building $input"
    cd $input/program
    cargo test-bpf --bpf-out-dir ../../test-programs/
    cd ../../

elif [[ $input = "all" ]]
then
    echo "building all programs"
    for program in ${programs}; do
        echo "building $program"
        cd $program/program
        cargo test-bpf --bpf-out-dir ../../test-programs/
        cd ../../
    done
else
    echo "Invalid program name: $input"
    exit 1
fi


