
JAVAC=javac
TEST_JAVA=$(wildcard tests/classes/Test*.java)
TEST_CLASSES=$(TEST_JAVA:.java=.class)

%.class: %.java
	$(JAVAC) $<

clean:
	rm -f $(TEST_CLASSES)

test: $(TEST_CLASSES)
	cargo test

lint:
	cargo clippy

fmt:
	cargo +nightly fmt

coverage: $(TEST_CLASSES)
	rm -rf *.profraw
	rm -rf ./target/debug/coverage/
	RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="coverage-%p-%m.profraw" cargo test
	grcov . -s . --binary-path ./target/debug/ --ignore "**/test*" -t html --branch --ignore-not-existing -o ./target/debug/coverage/

show-coverage:
	firefox ./target/debug/coverage/index.html

release:
	cargo build --release 

.PHONY: lint coverage show-coverage
