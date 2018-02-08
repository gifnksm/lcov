WORKDIR=target
TARGET=$(WORKDIR)/main
OBJS=$(patsubst src/%.c,$(WORKDIR)/%.o,$(wildcard src/*.c))

CC=gcc
CFLAGS=-coverage -Wall -Wextra
LDFLAGS=-coverage
LCOVFLAGS=-d $(WORKDIR) --rc lcov_branch_coverage=1

all: $(TARGET)
.PHONY: all

clean:
	$(RM) -r $(WORKDIR)
.PHONY: clean

$(WORKDIR)/%.o: src/%.c | $(WORKDIR)/
	$(CC) $(CFLAGS) -c $< -o $@
$(WORKDIR)/main: $(OBJS) | $(WORKDIR)/
	$(CC) $(LDFLAGS) $^ -o $@

lcov: report.info report_checksum.info
report.info: LCOVFLAGS+=
report_checksum.info: LCOVFLAGS+= --checksum

%.info: LCOVINIT=$(patsubst %.info,%.init.info,$@)
%.info: LCOVRUN=$(patsubst %.info,%.run.info,$@)
%.info: $(TARGET)
	lcov $(LCOVFLAGS) -z
	lcov $(LCOVFLAGS) -c -i -o $(LCOVINIT)
	./$(TARGET)
	lcov $(LCOVFLAGS) -c -o $(LCOVRUN)
	lcov $(LCOVFLAGS) -a $(LCOVINIT) -a $(LCOVRUN) -o $@

%/:
	mkdir -p $@

