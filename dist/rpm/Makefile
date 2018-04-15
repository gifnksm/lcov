define query-cargo-metadata
$(shell rq -tJ "$(2)" < $(1) --format compact | jq -r '.')
endef

PKG_MANIFEST=../../Cargo.toml
PKG_NAME=$(call query-cargo-metadata,$(PKG_MANIFEST),map(m => m.package.name))
PKG_VERSION=$(call query-cargo-metadata,$(PKG_MANIFEST),map(m => m.package.version))
PKG_RELEASE=0
PKG_SUMMARY=$(call query-cargo-metadata,$(PKG_MANIFEST),map(m => m.package.description))
PKG_DESCRIPTION=$(call query-cargo-metadata,$(PKG_MANIFEST),map(m => m.package.description))
PKG_LICENSE=$(call query-cargo-metadata,$(PKG_MANIFEST),map(m => m.package.license))
PKG_VENDOR=$(call query-cargo-metadata,$(PKG_MANIFEST),map(m => m.package.authors[0]))
PKG_GROUP=Applications/Utilities
PKG_URL=$(call query-cargo-metadata,$(PKG_MANIFEST),map(m => m.package.homepage || m.package.repository || m.package.documentation))

PKG=$(PKG_NAME)-$(PKG_VERSION)

WORKDIR:=target
RPMBUILD:=$(WORKDIR)/rpmbuild
REPOROOT:=$(shell git rev-parse --show-toplevel)

ORIG_HOME:=$(HOME)

all: build
.PHONY: all

clean:
	$(RM) -r $(WORKDIR)
.PHONY: clean

build: export RUSTUP_HOME=$(ORIG_HOME)/.rustup
build: export CARGO_HOME=$(ORIG_HOME)/.cargo
build: export HOME=$(shell readlink -f $(WORKDIR))
build: $(RPMBUILD)/SPECS/$(PKG).spec $(RPMBUILD)/SOURCES/master.tar.gz
	rpmbuild -bb \
	  $(RPMBUILD)/SPECS/$(PKG).spec \
	  --define "_pkg_name $(PKG_NAME)" \
	  --define "_pkg_version $(PKG_VERSION)" \
	  --define "_pkg_release $(PKG_RELEASE)" \
	  --define "_pkg_summary $(PKG_SUMMARY)" \
	  --define "_pkg_description $(PKG_DESCRIPTION)" \
	  --define "_pkg_license $(PKG_LICENSE)" \
	  --define "_pkg_vendor $(PKG_VENDOR)" \
	  --define "_pkg_group $(PKG_GROUP)" \
	  --define "_pkg_url $(PKG_URL)"
.PHONY: build

rpmbuild-tree: export HOME=$(shell readlink -f $(WORKDIR))
rpmbuild-tree: | $(WORKDIR)/
	rpmdev-setuptree
.PHONY: rpmbuild-tree

$(RPMBUILD)/SPECS/%.spec: package.spec rpmbuild-tree
	cp $< $@
$(RPMBUILD)/SOURCES/master.tar.gz: rpmbuild-tree
	git -C "$(REPOROOT)" archive HEAD --prefix "master/" --output=$(PWD)/$@

%/:
	mkdir -p $@

