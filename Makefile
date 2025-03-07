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
all: website native

.PHONY: website
website:
	$(MAKE) -C web2
	$(RM) -r web/
	$(CP) web2/dist/ web/ -r

.PHONY: native
native:
	@$(ECHO) Building org-roamers
	$(CARGO) build
	@$(ECHO) Finishing up org-roamers.$(LIB_EXTENSION)
	$(CP) target/debug/liborg_roamers.$(LIB_EXTENSION) org-roamers-utils.$(LIB_EXTENSION)

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
