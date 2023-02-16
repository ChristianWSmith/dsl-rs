#!/bin/bash

for file in $(find src/ -type f -name "*.rs")
do
    rustfmt --edition 2021 $file
done
