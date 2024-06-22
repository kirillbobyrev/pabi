# Supporting builds through `make` command is a requirement for OpenBench:
# https://github.com/AndyGrant/OpenBench/wiki/Requirements-For-Public-Engines#basic-requirements

# Variable for the output binary name, defaults to 'pabi' if not provided.
EXE ?= pabi

ifeq ($(OS),Windows_NT)
    EXE_SUFFIX := .exe
else
    EXE_SUFFIX :=
endif

# Compile flags for the fastest possible build.
COMPILE_FLAGS := RUSTFLAGS='-C target-feature=+avx2,+fma,+bmi1,+bmi2'

# Compile the target and add a link to the binary for OpenBench to pick up.
openbench:
	$(COMPILE_FLAGS) cargo rustc --profile=fast --bin=pabi -- --emit link=$(EXE)$(EXE_SUFFIX)

.PHONY: openbench
