build-client:
  pnpm i
  rm -rf assets/client-web
  pnpm client-web:build

build-linux: build-client
  cargo build --release --target x86_64-unknown-linux-gnu

build-windows: build-client
  cargo build --release --target x86_64-pc-windows-gnu

prepare-release:
  rm -rf dist/
  mkdir -p dist/windows
  mkdir -p dist/linux
  cp -rL build_assets/linux/* dist/linux/
  cp -rL build_assets/windows/* dist/windows/
  cp -rL build_assets/shared/* dist/linux/
  cp -rL build_assets/shared/* dist/windows/

release-linux: build-linux prepare-release
  cp target/x86_64-unknown-linux-gnu/release/airwave dist/linux/
  tar -czf dist/x86_64-unknown-linux-gnu.tar.gz dist/linux/

release-windows: build-windows prepare-release
  cp target/x86_64-pc-windows-gnu/release/airwave.exe dist/windows/
  zip dist/x86_64-pc-windows-gnu.zip dist/windows/

build: build-linux build-windows
release: release-linux release-windows
