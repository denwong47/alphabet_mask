BAKE_OPTIONS=--no-input

help:
	@echo "bake 	generate project using defaults"
	@echo "watch 	generate project using defaults and watch for changes"
	@echo "replay 	replay last cookiecutter run and watch for changes"

bake:
	cookiecutter $(BAKE_OPTIONS) . --overwrite-if-exists

watch: bake
	watchmedo shell-command -p '*.*' -c 'make bake -e BAKE_OPTIONS=$(BAKE_OPTIONS)' -W -R -D \alphabet_mask/

replay: BAKE_OPTIONS=--replay
replay: watch
	;

docker_up:
	docker-compose up -d --build

docker_kill:
	docker-compose down

git_init:
	git init -b main
	-git remote add origin https://github.com/denwong47/alphabet_mask.git

# Internal use only.
git_init_commit:
	git add --all
	@-pre-commit
	git add --all
	git commit -a -m "Initial Commit from template."

pytest:
	pytest

precommit_init:
	pre-commit install

pip_install_dev:
	python3 -m pip install -e "./[dev]"

pip_reinstall:
	python3 -m pip install -e ./

pip_install: pip_reinstall

docs_rebuild_only:
	cd docs; make rebuild

docs_build:
	cd docs; make html; make text
	cargo doc --no-deps --target-dir docs/build/html/lib

docs_rebuild: docs_rebuild_only docs_build

full_rebuild: pip_reinstall docs_rebuild

setup: git_init precommit_init pip_install_dev docs_rebuild pytest git_init_commit

setup_git: git_init precommit_init git_init_commit
