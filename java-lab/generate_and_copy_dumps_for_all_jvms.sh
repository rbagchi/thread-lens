#!/bin/bash

# Define the list of JVM services and their ports
# Using a simple string and then splitting it
JVM_SERVICES_LIST="java-app-openjdk8:8080 java-app-openjdk11:8082 java-app-openjdk17:8083 java-app-openjdk21:8084 java-app-ibm17:8085"

# Source and target directories for dumps
SOURCE_DIR="/Users/ranjan/project/thread-analyzer/jstack_dumps"
TARGET_BASE_DIR="thread-lens/src/test_data"

# Function to wait for the application to be ready
wait_for_app() {
    APP_PORT=$1
    echo "Waiting for app on port $APP_PORT to be ready..."
    for i in $(seq 1 30); do # Try for 30 seconds
        curl -s "http://localhost:$APP_PORT/" > /dev/null
        if [ $? -eq 0 ]; then
            echo "App on port $APP_PORT is ready."
            return 0
        fi
        sleep 1
    done
    echo "Error: App on port $APP_PORT did not become ready in time."
    return 1
}

# Loop through each JVM service
for SERVICE_ENTRY in $JVM_SERVICES_LIST; do
    SERVICE_NAME=$(echo "$SERVICE_ENTRY" | cut -d':' -f1)
    PORT=$(echo "$SERVICE_ENTRY" | cut -d':' -f2)
    TARGET_DIR="${TARGET_BASE_DIR}/$(echo "$SERVICE_NAME" | sed 's/java-app-//g')" # e.g., openjdk/openjdk8

    echo "--- Processing JVM: $SERVICE_NAME (Port: $PORT) ---"

    # 1. Bring down the service and delete associated volumes
    echo "Bringing down $SERVICE_NAME and deleting volumes..."
    docker-compose down -v "$SERVICE_NAME"

    # 2. Bring up the service
    echo "Bringing up $SERVICE_NAME..."
    docker-compose up -d "$SERVICE_NAME"

    # 3. Wait for the application to be ready
    wait_for_app "$PORT"
    if [ $? -ne 0 ]; then
        echo "Skipping $SERVICE_NAME due to app not ready."
        docker-compose down "$SERVICE_NAME" # Try to clean up
        continue
    fi

    # 4. Trigger the /hold-lock endpoint
    echo "Triggering /hold-lock endpoint on $SERVICE_NAME..."
    curl "http://localhost:$PORT/hold-lock"

    # 5. Wait for 20 seconds to allow dumps to be generated
    echo "Waiting for 20 seconds for dumps to generate..."
    sleep 20

    # 6. Create target directory if it doesn't exist
    mkdir -p "$TARGET_DIR"

    # 7. Run copy_dumps.sh to collect the generated dumps
    echo "Collecting dumps for $SERVICE_NAME..."
    java-lab/copy_dumps.sh "$SERVICE_NAME" "$TARGET_DIR"

    # 8. Bring down the service
    echo "Bringing down $SERVICE_NAME..."
    docker-compose down "$SERVICE_NAME"

    echo "--- Finished processing $SERVICE_NAME ---"
    echo ""
done

echo "All JVM dump generation and copying complete."
ll JVM dump generation and copying complete."
