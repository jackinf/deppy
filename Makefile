DEPPY_COMMAND=cargo run --package deppy-rust --bin deppy-rust

#
# Foo-Web
#

# Return all commits not deployed to Sign web production
foo-web-prod:
	$(DEPPY_COMMAND) to-deploy -p foo-web -e prod

# Return all commits not deployed to Sign web staging
foo-web-staging:
	$(DEPPY_COMMAND) to-deploy -p foo-web -e staging

#
# Bar-Web
#

# Return all commits not deployed to Docgen web Production
bar-web-prod:
	$(DEPPY_COMMAND) to-deploy -p bar-web -e prod

# Return all commits not deployed to Docgen web staging
bar-web-staging:
	$(DEPPY_COMMAND) to-deploy -p bar-web -e staging
