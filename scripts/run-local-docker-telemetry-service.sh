#!/bin/bash -eux
docker run -it -p 127.0.0.1:8082:8080/tcp --env RUST_LOG=debug oort_telemetry_service