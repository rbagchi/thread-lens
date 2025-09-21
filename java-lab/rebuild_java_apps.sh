#!/bin/bash

set -e

echo "--- Starting rebuild of all Java application Docker images ---"

# List of all services to rebuild
SERVICES_TO_REBUILD=(
    "java-app-openjdk8"
    "java-app-openjdk11"
    "java-app-openjdk17"
    "java-app-openjdk21"
    "java-app-ibm17"
)

for service in "${SERVICES_TO_REBUILD[@]}"; do
    echo "\nBuilding Docker image for service: ${service}..."
    docker-compose build --no-cache "${service}"
    echo "Successfully built ${service}."
done

echo "\n--- All specified Docker images rebuilt successfully ---"
