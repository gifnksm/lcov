TARGET_DIR = target
LCOV_DIR = $(shell readlink -f lcov)
PROFILE_DIR = $(LCOV_DIR)/test/profiles

TRACEFILE_NAME=full.info part1.info part2.info target.info zero.info
TRACEFILE_small=$(patsubst %,$(TARGET_DIR)/tracefiles/small/%,$(TRACEFILE_NAME))
TRACEFILE_medium=$(patsubst %,$(TARGET_DIR)/tracefiles/medium/%,$(TRACEFILE_NAME))
TRACEFILE_large=$(patsubst %,$(TARGET_DIR)/tracefiles/large/%,$(TRACEFILE_NAME))

PROFILE=

all:
.PHONY: all

clean:
	$(RM) -r $(TARGET_DIR)
.PHONY: clean

run-%: PROFILE=$(patsubst run-%,%,$@)
run-%:
	cargo build --release
	$(MAKE) measure-lcov-master PROFILE=$(PROFILE)
	$(MAKE) measure-lcov-installed PROFILE=$(PROFILE)
	$(MAKE) measure-lcov-merge PROFILE=$(PROFILE)

gen-tracefile: $(TARGET_DIR)/tracefiles/$(PROFILE)/.generated
.PHONY: gen-tracefile
$(TARGET_DIR)/tracefiles/%/.generated: $(PROFILE_DIR)/% | $(TARGET_DIR)/tracefiles/%/
	if [ -z "$(PROFILE)" ]; then echo "PROFILE is not specified"  ; exit 1; fi
	cd $(TARGET_DIR)/tracefiles/$(PROFILE) && $(LCOV_DIR)/test/bin/mkinfo "$(PROFILE_DIR)/$(PROFILE)" -o src
	touch $@

measure-lcov-master: $(TARGET_DIR)/results/$(PROFILE)/lcov-master
$(TARGET_DIR)/results/%/lcov-master: $(TARGET_DIR)/tracefiles/%/.generated | $(TARGET_DIR)/results/%/
	/usr/bin/time -p $(LCOV_DIR)/bin/lcov --rc lcov_branch_coverage=1 $(patsubst %,-a %,$(TRACEFILE_$(PROFILE))) -o $@.info 2>$@
	$(LCOV_DIR)/bin/lcov --version > $@.version

measure-lcov-installed: $(TARGET_DIR)/results/$(PROFILE)/lcov-installed
$(TARGET_DIR)/results/%/lcov-installed: $(TARGET_DIR)/tracefiles/%/.generated | $(TARGET_DIR)/results/%/
	/usr/bin/time -p lcov --rc lcov_branch_coverage=1 $(patsubst %,-a %,$(TRACEFILE_$(PROFILE))) -o $@.info 2>$@
	lcov --version > $@.version

measure-lcov-merge: $(TARGET_DIR)/results/$(PROFILE)/lcov-merge
$(TARGET_DIR)/results/%/lcov-merge: $(TARGET_DIR)/tracefiles/%/.generated | $(TARGET_DIR)/results/%/
	/usr/bin/time -p ../target/release/lcov-merge $(TRACEFILE_$(PROFILE)) > $@.info 2>$@
	../target/release/lcov-merge --version > $@.version

%/:
	mkdir -p $@

