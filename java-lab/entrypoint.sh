#!/bin/bash

# Start the Java application in the background
java -jar /app/app.jar &

# Get the PID of the background Java process
JAVA_APP_PID=$!

echo "Java application started with PID: $JAVA_APP_PID"

# Define the output directory for jstack dumps
# JSTACK_OUTPUT_SUBDIR is passed as an environment variable from the test script
OUTPUT_DIR="/shared-jstack-dumps/${JSTACK_OUTPUT_SUBDIR}"
mkdir -p "${OUTPUT_DIR}"

echo "Jstack dumps will be saved to: ${OUTPUT_DIR}"

# Loop to periodically dump jstack
while true; do
  /app/dump_jstack.sh "${OUTPUT_DIR}"
  sleep 5
done