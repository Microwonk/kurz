build:
	cargo build --release

watch:
	watchexec -r -e rs,html,css 'cargo run --features livereload,disable_auth'
