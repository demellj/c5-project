#!sh

# Clean before building via docker
(
	cd ./backend/
	cargo clean
)

docker-compose -f ./docker/docker-compose-build-phase1.yaml build --parallel && \
docker-compose -f ./docker/docker-compose-build-phase2.yaml build --parallel
