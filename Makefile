

VERSION=$(shell cat VERSION)
GIT_BRANCH=$(shell git rev-parse --abbrev-ref HEAD)
GIT_REV=$(shell git rev-parse --short HEAD)

all: version

build.rs:
	@@echo "// this is auto-generated file, don't edit this by your dirty hands bitch!!" > src/build.rs
	@@echo "pub const GIT_REV:&'static str = \"$(GIT_REV) ($(GIT_BRANCH))\";" >> src/build.rs
	@@echo "pub const VERSION:&'static str =  \"$(VERSION)\";" >> src/build.rs

init-dev:
	@@if [ -f src/build.rs ]; then \
		echo "already initialized!"; \
	else \
		echo "initializing..."; \
		make build.rs; \
	fi;

version: build.rs


clean:

.PHONY: all version clean init-dev
