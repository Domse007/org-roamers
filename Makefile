CARGO:=cargo
CP:=cp
STRIP:=strip
ECHO:=echo
RM:=rm

ifeq ($(OS),Windows_NT)
	LIB_EXTENSION:=dll
else
	LIB_EXTENSION:=so
endif

.PHONY: all
all:
	@$(ECHO) Building org-roam-rs
	$(CARGO) build
	@$(ECHO) Finishing up org-roam-rs.$(LIB_EXTENSION)
	$(CP) target/debug/liborg_roam_rs.$(LIB_EXTENSION) org-roam-rs.$(LIB_EXTENSION)
	# $(STRIP) org-roam-rs.$(LIB_EXTENSION)
	$(MAKE) -c web2
	$(RM) -r web/
	$(CP) web2/dist/ web/ -r

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
