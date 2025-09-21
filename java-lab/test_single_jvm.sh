#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
BASE_JSTACK_DUMPS_DIR="./jstack_dumps"
TIMEOUT_SECONDS=150 # Increased timeout for dump generation and copy
MIN_DUMPS=10 # Minimum number of dumps to wait for

# Define block types to test
BLOCK_TYPES=("block" "deadlock") # Add "hold-lock" if needed

# --- Functions ---

function cleanup_containers() {
    echo "Stopping and removing all test-related Docker containers..."
    # Get a list of all containers (running and exited) that match the name pattern
    CONTAINER_IDS=$(docker ps -a --filter "name=java-app-" --format "{{.ID}}")
    if [ -n "${CONTAINER_IDS}" ]; then
        echo "Stopping containers: ${CONTAINER_IDS}"
        docker stop -t 0 ${CONTAINER_IDS} || true
        # Wait for stop to complete before attempting to remove
        sleep 2
        echo "Removing containers: ${CONTAINER_IDS}"
        docker rm ${CONTAINER_IDS} || true
    fi

    CONTAINER_IDS=$(docker ps -a --filter "name=my-java-app" --format "{{.ID}}")
    if [ -n "${CONTAINER_IDS}" ]; then
        echo "Stopping containers: ${CONTAINER_IDS}"
        docker stop -t 0 ${CONTAINER_IDS} || true
        # Wait for stop to complete before attempting to remove
        sleep 2
        echo "Removing containers: ${CONTAINER_IDS}"
        docker rm ${CONTAINER_IDS} || true
    fi
    echo "Cleanup complete."
}

function wait_for_app_ready() {
    local host_port=$1
    local timeout=120
    local start_time=$(date +%s)
    echo "Waiting for Java application on port ${host_port} to be ready..."

    while true; do
        current_time=$(date +%s)
        elapsed_time=$((current_time - start_time))

        if [ "${elapsed_time}" -ge "${timeout}" ]; then
            echo "Error: Java application did not become ready within ${timeout} seconds."
            return 1
        fi

        CURL_APP_READY_OUTPUT=$(curl -s http://localhost:${host_port}/)
        HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:${host_port}/)
        if [ "${HTTP_CODE}" -eq 200 ]; then
            echo "Java application on port ${host_port} is ready."
            return 0
        fi

        echo "App not ready yet (status: ${HTTP_CODE}). Output: ${CURL_APP_READY_OUTPUT}. Waiting..."
        sleep 2
    done
}

function get_host_port() {
    local java_container_name=$1
    case "${java_container_name}" in
        "java-app-openjdk8") echo "8080" ;; 
        "java-app-openjdk11") echo "8082" ;; 
        "java-app-openjdk17") echo "8083" ;; 
        "java-app-openjdk21") echo "8084" ;; 
        "java-app-ibm17") echo "8085" ;; 
        *) echo "Error: Unknown Java container name: ${java_container_name}" >&2; exit 1 ;; 
    esac
}

function wait_for_dumps_in_container() {
    local container_id=$1
    local java_container_name=$2
    local container_jstack_path_base="/shared-jstack-dumps"
    local container_jstack_path_full="${container_jstack_path_base}/${java_container_name}_${TIMESTAMP}_${BLOCK_TYPE}" # Use global TIMESTAMP and BLOCK_TYPE
    local start_time=$(date +%s)
    local dump_count=0

    echo "Waiting for at least ${MIN_DUMPS} jstack dumps for ${java_container_name} (${BLOCK_TYPE}) in container ${container_id}:${container_jstack_path_full}..."

    while true; do
        current_time=$(date +%s)
        elapsed_time=$((current_time - start_time))

        if [ "${elapsed_time}" -ge "${TIMEOUT_SECONDS}" ]; then
            echo "Timeout reached. Only found ${dump_count} dumps in container."
            return 1 # Indicate failure
        fi

        # Check if container is still running
        if [ "$(docker inspect -f '{{.State.Running}}' ${container_id})" != "true" ]; then
            echo "Error: Java container ${container_id} exited prematurely while waiting for dumps."
            docker logs ${container_id} # Print logs for debugging
            return 1
        fi

        # Check for dumps inside the container
        # Use -maxdepth 2 to only count files directly in the target directory
        dump_count=$(docker exec "${container_id}" find "${container_jstack_path_full}" -maxdepth 1 -name "${java_container_name}*.jstack" | wc -l | tr -d ' ')

        if [ "${dump_count}" -ge "${MIN_DUMPS}" ]; then
            echo "Found ${dump_count} jstack dumps in container. Proceeding to copy."
            return 0 # Indicate success
        fi

        echo "Found ${dump_count} dumps so far in container. Waiting..."
        sleep 5
    done
}

function run_analyzer() {
    local run_output_dir=$1
    echo "Running Rust analyzer..."
    # The CLI is now expected to be run from the project root
    ../target/release/thread-lens-cli analyze --path "${run_output_dir}"
}

function perform_sanity_check() {
    local analysis_output=$1
    echo "Performing sanity check on analysis output..."

    if echo "${analysis_output}" | grep -q "No persistently blocked threads found." && \
       echo "${analysis_output}" | grep -q "No common blocked stack frames found."; then
        echo "WARNING: Analyzer reported no persistently blocked threads and no common blocked stack frames."
        echo "This might indicate an issue with dump generation or the Java application's blocking behavior."
        return 1 # Indicate potential issue
    else
        echo "Sanity check passed: Analyzer found some blocked threads or common stack frames."
        return 0 # Indicate success
    fi
}

# --- Main Script Logic ---

if [ -z "$1" ]; then
    echo "Usage: $0 <java_container_name>"
    echo "Example: $0 java-app-openjdk17"
    exit 1
fi

JAVA_CONTAINER_NAME=$1

# Perform cleanup before starting tests
cleanup_containers

# Generate a unique timestamp for this test run (once per JVM test)
TIMESTAMP=$(date +"%Y%m%d%H%M%S")

for BLOCK_TYPE in "${BLOCK_TYPES[@]}"; do
    echo "\n--- Starting test for ${JAVA_CONTAINER_NAME} with block type: ${BLOCK_TYPE} ---"

    # Generate a unique directory name for this specific block type test
    RUN_OUTPUT_SUBDIR="${JAVA_CONTAINER_NAME}_${TIMESTAMP}_${BLOCK_TYPE}"
    RUN_OUTPUT_DIR="${BASE_JSTACK_DUMPS_DIR}/${RUN_OUTPUT_SUBDIR}" # Full host path

    mkdir -p "${RUN_OUTPUT_DIR}"

    # Step 1: Start Java container directly
    echo "Starting Java container: ${JAVA_CONTAINER_NAME} (ID will be dynamic)..."
    # Pass JSTACK_OUTPUT_SUBDIR as an environment variable to the container
    # The container will create its own local directory for dumps
    HOST_PORT=$(get_host_port "${JAVA_CONTAINER_NAME}") # Moved this line up
    CONTAINER_ID=$(docker run -d --rm \
        --name "${JAVA_CONTAINER_NAME}_${TIMESTAMP}_${BLOCK_TYPE}" \
        -p ${HOST_PORT}:8080 \
        -e CONTAINER_NAME="${JAVA_CONTAINER_NAME}" \
        -e JSTACK_OUTPUT_SUBDIR="${RUN_OUTPUT_SUBDIR}" \
        "thread-analyzer-${JAVA_CONTAINER_NAME}:latest" \
        /app/entrypoint.sh)

    echo "Java container started with ID: ${CONTAINER_ID}"

    # Verify container is running
    sleep 30 # Give it a moment to start
    if [ "$(docker inspect -f '{{.State.Running}}' ${CONTAINER_ID})" != "true" ]; then
        echo "Error: Java container ${CONTAINER_ID} is not running after startup."
        docker logs ${CONTAINER_ID} # Print logs for debugging
        exit 1
    fi

    # Step 2: Start Rust sidecar (Temporarily commented out for JVM-only testing)
    # echo "Starting Rust sidecar..."
    # docker-compose up -d rust-sidecar

    # Ensure services are stopped even if script fails
    trap "docker stop ${CONTAINER_ID}" EXIT

    # Step 3: Trigger blocked conditions in Java app
    HOST_PORT=$(get_host_port "${JAVA_CONTAINER_NAME}")
    if [ -n "${HOST_PORT}" ]; then
        # Wait for the application to be ready
        if ! wait_for_app_ready "${HOST_PORT}"; then
            echo "Error: Java application did not become ready. Exiting."
            exit 1
        fi

        echo "Triggering blocked condition '${BLOCK_TYPE}' on ${JAVA_CONTAINER_NAME} via port ${HOST_PORT}..."
        HTTP_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${HOST_PORT}/${BLOCK_TYPE}")
        CURL_OUTPUT=$(curl -s "http://localhost:${HOST_PORT}/${BLOCK_TYPE}")
        echo "Curl HTTP Status: ${HTTP_RESPONSE}"
        echo "Curl Output: ${CURL_OUTPUT}"

        if [ "${HTTP_RESPONSE}" -ne 200 ]; then
            echo "Error: Curl request failed with status ${HTTP_RESPONSE}."
            exit 1
        fi
        sleep 15 # Give more time for conditions to manifest
    else
        echo "Warning: Could not determine host port for ${JAVA_CONTAINER_NAME}. Skipping blocked condition trigger."
    fi

    # Step 4: Wait for dumps to be generated inside the container
    if ! wait_for_dumps_in_container "${CONTAINER_ID}" "${JAVA_CONTAINER_NAME}"; then
        echo "Error: Failed to generate enough jstack dumps inside the container within the timeout."
        exit 1
    fi

    # Step 5: Copy dumps from container to host
    CONTAINER_JSTACK_SOURCE_PATH="/shared-jstack-dumps/${RUN_OUTPUT_SUBDIR}"
    echo "Copying dumps from container ${CONTAINER_ID}:${CONTAINER_JSTACK_SOURCE_PATH} to host ${RUN_OUTPUT_DIR}"
    docker cp "${CONTAINER_ID}:${CONTAINER_JSTACK_SOURCE_PATH}/." "${RUN_OUTPUT_DIR}"
    echo "Dumps copied."

    # Step 6: Run analyzer on the copied dumps (Temporarily commented out for JVM-only testing)
    # ANALYSIS_RESULT=$(run_analyzer "${RUN_OUTPUT_DIR}")
    # echo "${ANALYSIS_RESULT}"

    # Step 7: Perform sanity check (Temporarily commented out for JVM-only testing)
    # if ! perform_sanity_check "${ANALYSIS_RESULT}"; then
    #     echo "Sanity check failed. Review the output above."
    # fi

    # Step 8: Save output (Temporarily commented out for JVM-only testing)
    # REPORT_FILE="analysis_report_${JAVA_CONTAINER_NAME}_${TIMESTAMP}_${BLOCK_TYPE}.txt"
    # echo "${ANALYSIS_RESULT}" > "${REPORT_FILE}"
    # echo "Analysis report saved to ${REPORT_FILE}"

    echo "--- Test for ${JAVA_CONTAINER_NAME} with block type: ${BLOCK_TYPE} completed ---"

    # Stop the Java container after each block type test
    docker stop ${CONTAINER_ID}

done
