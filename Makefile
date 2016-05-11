all: cargo-build docker-build

cargo-build:
	docker run \
		--rm \
		-v ${PWD}:/volume \
		-v ${HOME}/.cargo/git:/root/.cargo/git \
		-v ${HOME}/.cargo/registry:/root/.cargo/registry \
		-w /volume \
		-t \
		clux/muslrust \
		cargo build --release

docker-build:
	docker build -t inquicker/ktmpl .
