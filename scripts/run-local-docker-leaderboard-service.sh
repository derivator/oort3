#!/bin/bash -eux
docker run --rm -it -p 127.0.0.1:8083:8080/tcp --env RUST_LOG=debug --env ENVIRONMENT=dev oort_leaderboard_service
