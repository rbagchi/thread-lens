#!/bin/bash

SOURCE_DIR="/Users/ranjan/project/thread-analyzer/jstack_dumps"

# Function to copy dumps for a specific JVM
copy_dumps() {
    JVM_NAME=$1
    TARGET_DIR=$2

    echo "Copying dumps for $JVM_NAME to $TARGET_DIR"

    # Delete existing dumps in the target directory
    rm -f "${TARGET_DIR}"/*

    # Get the 20 latest dumps from the source directory, filtered by JVM name
    LATEST_DUMPS=$(ls -t "${SOURCE_DIR}/${JVM_NAME}_"*.jstack 2>/dev/null | head -n 20)

    # Copy them to the target directory
    for dump_file in $LATEST_DUMPS; do
        cp "$dump_file" "${TARGET_DIR}/"
    done
    echo "Finished copying dumps for $JVM_NAME."
}

# Call the function with arguments passed to the script
copy_dumps "$1" "$2"

echo "Script finished."
