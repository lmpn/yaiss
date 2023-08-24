prepare:
	cargo sqlx prepare --database-url sqlite:backend/sql/test.db --merged

backend-prepare:
	DB_URL=sqlite:backend/sql/test.db ./backend/sql/delete-db.sh 
	DB_URL=sqlite:backend/sql/test.db ./backend/sql/create-db.sh

backend-coverage: backend-prepare
	DB_URL=sqlite:sql/test.db INI_CONFIGURATION=resources/dev.configuration.ini \
	cargo llvm-cov --all-features --workspace --lcov --ignore-filename-regex "bin|services/images/ports/|web/mod.rs"  --output-path lcov.info

backend-tests: backend-prepare
	DB_URL=sqlite:sql/test.db INI_CONFIGURATION=resources/dev.configuration.ini cargo test -p yaiss-backend

clippy: prepare
	cargo clippy