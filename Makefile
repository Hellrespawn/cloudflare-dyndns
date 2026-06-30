PREFIX     = /usr/local
BINDIR     = $(PREFIX)/bin
SYSCONFDIR = /etc
UNITDIR    = /etc/systemd/system

NAME        = ryndns
BINARY      = target/release/$(NAME)
CONFIG_DIR  = $(SYSCONFDIR)/$(NAME)
CONFIG_FILE = $(CONFIG_DIR)/$(NAME).toml

.PHONY: all build clean check install uninstall

all: build

build:
	cargo build --release

clean:
	cargo clean

check:
	cargo test

install: build
	install -Dm755 $(BINARY) $(DESTDIR)$(BINDIR)/$(NAME)
	sed 's|ExecStart=.*|ExecStart=$(BINDIR)/$(NAME)|' $(NAME).service \
	    | install -Dm644 /dev/stdin $(DESTDIR)$(UNITDIR)/$(NAME).service
	install -Dm644 $(NAME).timer $(DESTDIR)$(UNITDIR)/$(NAME).timer
	@if [ ! -f "$(DESTDIR)$(CONFIG_FILE)" ]; then \
		install -Dm640 $(NAME).example.toml $(DESTDIR)$(CONFIG_FILE); \
		echo "Installed example config to $(DESTDIR)$(CONFIG_FILE)"; \
	else \
		echo "Config already exists at $(DESTDIR)$(CONFIG_FILE), skipping"; \
	fi

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(NAME)
	rm -f $(DESTDIR)$(UNITDIR)/$(NAME).service
	rm -f $(DESTDIR)$(UNITDIR)/$(NAME).timer
	@if [ -f "$(DESTDIR)$(CONFIG_FILE)" ]; then \
		mv $(DESTDIR)$(CONFIG_FILE) $(DESTDIR)$(CONFIG_FILE).save; \
		echo "Backed up config to $(DESTDIR)$(CONFIG_FILE).save"; \
	fi
