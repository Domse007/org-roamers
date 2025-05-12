CARGO:=cargo
CP:=cp
STRIP:=strip
ECHO:=echo
RM:=rm

.PHONY: all
all: website native

.PHONY: website
website:
	$(MAKE) -C web all

.PHONY: format
format:
	$(CARGO) fmt
	$(MAKE) -C web format

.PHONY: native
native:
	@$(ECHO) Building org-roamers
	$(CARGO) build --release --bin server
	$(CARGO) build --release --bin cli

.PHONY: clean
clean:
	$(RM) *.$(LIB_EXTENSION)
	$(RM) -r target/

.PHONY: help
help:
	@$(ECHO) "Available targets:"
	@$(ECHO) "  all        Build the rust project"
	@$(ECHO) "  clean      Remove all build artefacts"
	@$(ECHO) "  help       Print this message"
