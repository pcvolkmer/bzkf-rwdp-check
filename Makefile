ifndef VERBOSE
.SILENT:
endif

PROG_NAME = "bzkf-rwdp-check"

TAG = `git describe --tag 2>/dev/null`

REV = git`git rev-parse HEAD | cut -c1-7`

package-all: win-package linux-package

.PHONY: win-package
win-package: win-binary-x86_64
	mkdir $(PROG_NAME) 2>/dev/null || true
	cp target/x86_64-pc-windows-gnu/release/$(PROG_NAME).exe $(PROG_NAME)/
	cp README.md $(PROG_NAME)/
	cp LICENSE $(PROG_NAME)/
	zip $(PROG_NAME)-$(TAG)_win64.zip $(PROG_NAME)/*
	rm -rf $(PROG_NAME) || true

.PHONY: linux-package
linux-package: linux-binary-x86_64
	mkdir $(PROG_NAME) 2>/dev/null || true
	cp target/x86_64-unknown-linux-gnu/release/$(PROG_NAME) $(PROG_NAME)/
	cp README.md $(PROG_NAME)/
	cp LICENSE $(PROG_NAME)/
	tar -czvf $(PROG_NAME)-$(TAG)_linux.tar.gz $(PROG_NAME)/
	rm -rf $(PROG_NAME) || true

binary-all: win-binary-x86_64 linux-binary-x86_64

.PHONY: win-binary-x86_64
win-binary-x86_64:
	cargo build --release --target=x86_64-pc-windows-gnu

.PHONY: linux-binary-x86_64
linux-binary-x86_64:
	cargo build --release --target=x86_64-unknown-linux-gnu

.PHONY: install
install:
	cargo install --path .

.PHONY: clean
clean:
	cargo clean
	rm -rf $(PROG_NAME) 2>/dev/null || true
	rm *_win64.zip 2>/dev/null || true
	rm *_linux.tar.gz 2>/dev/null || true
