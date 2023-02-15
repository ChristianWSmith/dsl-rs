#!/bin/bash

for file in $(find src/ -type f -name "*.rs")
do
    rustfmt $file
done
