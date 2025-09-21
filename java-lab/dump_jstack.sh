# Dummy comment to force rebuild
#!/bin/bash

JAVA_PID=$(pgrep -f "java -jar /app/app.jar")

if [ -z "$JAVA_PID" ]; then
  echo "Error: Java process not found." >&2
  exit 1
fi

# The first argument is the output directory
OUTPUT_DIR="$1"

if [ -z "$OUTPUT_DIR" ]; then
  echo "Error: Output directory not provided." >&2
  exit 1
fi

# CONTAINER_NAME is passed as an environment variable from docker-compose
if [ -z "$CONTAINER_NAME" ]; then
  echo "Error: CONTAINER_NAME environment variable not set." >&2
  exit 1
fi

TIMESTAMP=$(date +%Y%m%d%H%M%S%3N) # Add milliseconds for more uniqueness
JSTACK_FILE="${CONTAINER_NAME}_${TIMESTAMP}.jstack"
FULL_PATH="${OUTPUT_DIR}/${JSTACK_FILE}"

# Dump jstack output to a file
jstack "$JAVA_PID" > "$FULL_PATH" 2>/dev/null

echo "jstack dump for PID $JAVA_PID saved to $FULL_PATH"