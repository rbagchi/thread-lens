# Thread Analyzer Project Status

This document outlines the current state of the Thread Analyzer project, detailing what has been accomplished and what remains to be done.

## Current State (What Works)

### 1. Java Lab Application (`java-lab-app`)
*   **Framework**: Converted from Spring Boot to **Spark Java** for a lighter-weight footprint and broader JDK compatibility.
*   **Endpoints**:
    *   `/`: A simple "Hello World" endpoint.
    *   `/block`: Simulates a blocked thread for 60 seconds.
    *   `/deadlock`: Initiates a classic two-thread deadlock scenario.
*   **Containerization**: Dockerfiles are available for various JDKs, allowing the application to run on:
    *   OpenJDK 8 (`Dockerfile.openjdk8`)
    *   OpenJDK 11 (`Dockerfile.openjdk11`)
    *   OpenJDK 17 (`Dockerfile.openjdk17`)
    *   OpenJDK 21 (`Dockerfile.openjdk21`)
    *   IBM Semeru Runtime 17 (`Dockerfile.ibm17`)
*   **Jstack Dumping (Shared Volume Mode)**:
    *   `entrypoint.sh`: Starts the Java application and periodically runs `dump_jstack.sh` in the background.
    *   `dump_jstack.sh`: Executes `jstack` on the running Java process, saves the output to a timestamped file on a shared volume (`/shared-jstack-dumps`), and performs cleanup to keep only the latest 5 `jstack` files for each container.

### 2. Rust Sidecar (`thread-lens-sidecar`)
*   **Name**: Renamed to `thread-lens-sidecar` for clarity and adherence to naming conventions.
*   **Core Functionality**:
    *   **Thread Dump Collection (Shared Volume Mode)**: Reads `jstack` output directly from files on a shared volume, eliminating the need for `hostPID` access.
    *   **Thread State Parsing**: Parses `jstack` output to classify and count threads into states like `RUNNABLE`, `BLOCKED`, `WAITING`, and `TIMED_WAITING`.
    *   **Modular Parsing**: Includes logic to detect the JVM vendor (OpenJDK/HotSpot or IBM OpenJ9) from the `jstack` output and apply the appropriate parsing strategy.
*   **API Endpoints**:
    *   `/threads/{filename}`: Returns the raw `jstack` output from the specified file on the shared volume.
    *   `/stats/{filename}`: Provides a JSON summary of thread counts by state from the specified file.
    *   `/metrics/{filename}`: Exposes thread state metrics in a Prometheus-compatible format from the specified file.
    *   `/discover_jstack_files`: Lists the names of available `jstack` files (which serve as identifiers for the Java applications) on the shared volume.
*   **Containerization**: The `thread-lens-sidecar` Docker image includes its own OpenJDK 17 installation, making it self-sufficient for executing `jstack` (though this is now less critical in shared volume mode, it ensures `jstack` is available if needed for other purposes).

### 3. Docker Compose Setup
*   **`docker-compose.yml`**: Configured to orchestrate the `thread-lens-sidecar` and multiple instances of the `java-lab-app` (OpenJDK 8, 11, 17, 21, IBM 17).
*   **PID Namespace**: `pid: "host"` has been removed from all services, enhancing security and Kubernetes compatibility.
*   **Shared Volume**: A named Docker volume (`jstack-data`) is defined and mounted to `/shared-jstack-dumps` in all Java application containers (for writing `jstack` dumps) and the `thread-lens-sidecar` (for reading them).
*   **Port Mapping**: Each `java-lab-app` instance is mapped to a unique port (8080, 8082, 8083, 8084, 8085). The `thread-lens-sidecar` runs on port 8081.
*   **Environment Variables**: `CONTAINER_NAME` environment variable is passed to each Java application container to identify its `jstack` dump file.

## What is Left to be Done

### 1. Rust Sidecar - Parsing Refinement
*   **IBM OpenJ9 Parsing**: The parsing logic for IBM OpenJ9 `jstack` output is currently a placeholder. Full implementation is required to accurately extract thread statistics from IBM JVMs.
*   **OpenJDK Version Nuances**: While basic thread states are covered, further research and refinement might be needed to capture subtle differences or additional information present in `jstack` output across different OpenJDK versions (e.g., new thread states, specific diagnostic information).

### 2. Support for Both Modes (Direct PID vs. Shared Volume)
*   Implement a mechanism (e.g., environment variable configuration) to allow the `thread-lens-sidecar` to operate in either the `direct_pid` mode (using `hostPID` and direct `jstack` execution) or the `shared_volume` mode (current implementation).

### 3. Verification and Testing
*   **Comprehensive Testing**: Thorough testing of the `thread-lens-sidecar` against all configured Java application instances (OpenJDK 8, 11, 17, 21, IBM 17) in the new shared volume mode is needed to ensure accurate thread dump collection and parsing for all scenarios (normal, blocked, deadlocked).
*   **Prometheus Metrics Validation**: Verify that the `/metrics` endpoint provides correctly formatted and accurate data for Prometheus integration.

### 4. Growth Path (Future Enhancements)
*   **Reporting**: Implement functionality to generate Markdown or PDF reports with detailed thread analysis.
*   **Integrations**: Develop integrations with communication platforms like Slack or Microsoft Teams for alerts and notifications.
*   **Historical Analysis**: Add capabilities for storing and analyzing historical thread dump data.
*   **Advanced Analysis**: Explore AI-powered analysis, fix suggestions, and enterprise-specific integrations.
*   **Alternative JVMs**: Investigate and implement parsing for other JVMs if their `jstack` output significantly deviates from OpenJDK/HotSpot or IBM OpenJ9 (e.g., Azul Platform Prime/Zing).