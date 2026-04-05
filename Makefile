# Build configuration
CARGO ?= cargo
TARGET_DIR = release

# Installation paths
PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin

# Install commands
INSTALL := install
INSTALL_PROGRAM := $(INSTALL) -D -m 0755
INSTALL_DATA := $(INSTALL) -D -m 0644

# Binary names
BIN_DAEMON := cardwired
BIN_CLI := cardwire

# Asset files
SERVICE_NAME := cardwired.service
SERVICE_FILE := /usr/lib/systemd/system/$(SERVICE_NAME)
DBUS_CONFIG := /usr/share/dbus-1/system.d/com.github.luytan.cardwire.conf

# Build targets
TARGET_DAEMON := target/$(TARGET_DIR)/$(BIN_DAEMON)
TARGET_CLI := target/$(TARGET_DIR)/$(BIN_CLI)

.DEFAULT_GOAL := build

#=============================================================================
# Build targets
#=============================================================================

build:
	$(CARGO) build --release

check:
	$(CARGO) check --release
	$(CARGO) clippy --release -- -D warnings

#=============================================================================
# Installation targets
#=============================================================================

install:
	@echo "Installing binaries..."
	$(INSTALL_PROGRAM) "$(TARGET_DAEMON)" "$(DESTDIR)$(BINDIR)/$(BIN_DAEMON)"
	$(INSTALL_PROGRAM) "$(TARGET_CLI)" "$(DESTDIR)$(BINDIR)/$(BIN_CLI)"
	@echo "Installing systemd service..."
	$(INSTALL_DATA) "assets/cardwired.service" "$(DESTDIR)$(SERVICE_FILE)"
	@echo "Installing D-Bus config..."
	$(INSTALL_DATA) "assets/com.github.luytan.cardwire.conf" "$(DESTDIR)$(DBUS_CONFIG)"
ifeq ($(DESTDIR),)
	@echo "Reloading systemd daemon..."
	systemctl daemon-reload
	@if systemctl is-enabled --quiet $(SERVICE_NAME) 2>/dev/null; then \
		echo "Service already enabled."; \
	else \
		echo "Enabling service..."; \
		systemctl enable $(SERVICE_NAME); \
	fi
	@echo "Installation complete. Please reboot or start the service manually."
endif

.PHONY: build check install
