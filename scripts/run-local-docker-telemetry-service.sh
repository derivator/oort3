#!/bin/bash -eux
exec docker run --rm -it -p 127.0.0.1:8082:8080/tcp --env RUST_LOG=debug --env ENVIRONMENT=dev oort_telemetry_service
