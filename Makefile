TAG = 0.4.1

all: dist

.PHONY: clean
clean:
	rm dist/*
	cargo clean

target/release/ktmpl:
	cargo build --release

target/x86_64-unknown-linux-musl/release/ktmpl:
	docker run \
		--rm \
		-v $(PWD):/volume \
		-v $(HOME)/.cargo/git:/root/.cargo/git \
		-v $(HOME)/.cargo/registry:/root/.cargo/registry \
		-w /volume \
		-t \
		clux/muslrust \
		cargo build --release

.PHONY: docker-build
docker-build:
	docker build -t inquicker/ktmpl -t inquicker/ktmpl:$(TAG) .

dist: clean dist/sha256sums.txt.sig docker-build

dist/ktmpl-$(TAG)-darwin.tar.gz: target/release/ktmpl
	tar -c -C target/release -zvf dist/ktmpl-$(TAG)-darwin.tar.gz ktmpl

dist/ktmpl-$(TAG)-linux.tar.gz: target/x86_64-unknown-linux-musl/release/ktmpl
	tar -c -C target/x86_64-unknown-linux-musl/release -zvf dist/ktmpl-$(TAG)-linux.tar.gz ktmpl

dist/sha256sums.txt: dist/ktmpl-$(TAG)-darwin.tar.gz dist/ktmpl-$(TAG)-linux.tar.gz
	cd dist && shasum -a 256 * > sha256sums.txt

dist/sha256sums.txt.sig: dist/sha256sums.txt
	cd dist && gpg2 --detach-sign sha256sums.txt
