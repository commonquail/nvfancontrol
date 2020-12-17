.SUFFIXES:

include driver_version.mk

ifndef NV_DRIVER_VERSION
    $(error set NV_DRIVER_VERSION in driver_version.mk)
endif

LIBRARY_PATH ?= nvidia-settings-$(NV_DRIVER_VERSION)/src/libXNVCtrl/_out/Linux_x86_64
RUSTFLAGS ?= "-C target-cpu=native"

CARGO ?= RUSTFLAGS=$(RUSTFLAGS) LIBRARY_PATH=$(LIBRARY_PATH) cargo

sources = $(wildcard src/*.rs)

bin = nvfancontrol
debug_bin = target/debug/$(bin)
release_bin = target/release/$(bin)

nvidia_settings_dir = nvidia-settings-$(NV_DRIVER_VERSION)
libxnvctrl = $(LIBRARY_PATH)/libXNVCtrl.a

alldeps = $(libxnvctrl) $(sources)

systemd_unit_user = $(HOME)/.config/systemd/user/$(bin).service

.PHONY: build
build: $(debug_bin)

.PHONY: test
test: build
	$(CARGO) test

.PHONY: release-build
release-build: $(release_bin)

$(debug_bin): $(alldeps)
	$(CARGO) build

$(release_bin): $(alldeps)
	$(CARGO) build --release

# If user overrides LIBRARY_PATH this target recipe won't work -- but in that
# case the target will (should) already exist, else the user wouldn't be
# overriding LIBRARY_PATH.
$(libxnvctrl): $(nvidia_settings_dir)
	$(MAKE) -C $</src/libXNVCtrl

$(nvidia_settings_dir).tar.bz2:
	wget https://download.nvidia.com/XFree86/nvidia-settings/$@

%: %.tar.bz2
	tar xf $<

.PHONY: clean
clean:
	cargo clean

.PHONY: clobber
clobber:
	cargo clean
	$(RM) -r $(nvidia_settings_dir)*

.PHONY: install
install: $(release_bin)
	install -m 0775 $(release_bin) $(HOME)/.local/bin/

.PHONY: daemon-install
daemon-install: $(systemd_unit_user)

.PHONY: daemon-status
daemon-status:
	systemctl --user status $(bin)

.PHONY: daemon-restart
daemon-restart:
	systemctl --user restart $(bin)

.PHONY: daemon-start
daemon-start:
	systemctl --user start $(bin)

.PHONY: daemon-stop
daemon-stop:
	systemctl --user stop $(bin)

.PHONY: daemon-reload
daemon-reload:
	systemctl --user daemon-reload

.PHONY: daemon-tail
daemon-tail:
	journalctl --user --utc --pager-end --follow --unit $(bin) --output short-iso --since today

.PHONY: daemon-edit
daemon-edit: daemon-install
	$(EDITOR) $(systemd_unit_user)

$(systemd_unit_user):
	cp $(@F) $@
