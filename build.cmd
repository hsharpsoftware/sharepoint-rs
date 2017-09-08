set RUST_BACKTRACE=1

cargo build
if errorlevel 1 (
  exit /b %errorlevel%
)
cargo test
if errorlevel 1 (
  exit /b %errorlevel%
)
exit 0
