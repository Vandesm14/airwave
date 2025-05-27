build-client:
  pnpm i
  rm -rf assets/client-web
  pnpm client-web:build

build-linux-gnu:
  cargo build --release --target x86_64-unknown-linux-gnu

build-linux-musl:
  cargo build --release --target x86_64-unknown-linux-musl

build-windows-gnu:
  cargo build --release --target x86_64-pc-windows-gnu

prepare-release: build-client
  mkdir -p dist/windows-gnu
  mkdir -p dist/linux-gnu
  mkdir -p dist/linux-musl
  mkdir -p dist/release
  cp -rL build_assets/shared/* dist/linux-gnu/
  cp -rL build_assets/shared/* dist/linux-musl/
  cp -rL build_assets/shared/* dist/windows/

release-linux-gnu: build-linux-gnu prepare-release
  cp -rL build_assets/linux/* dist/linux-gnu/
  cp target/x86_64-unknown-linux-gnu/release/airwave dist/linux-gnu/
  tar -czf dist/release/airwave-x86_64-unknown-linux-gnu.tar.gz dist/linux-gnu/

release-linux-musl: build-linux-musl prepare-release
  cp -rL build_assets/linux/* dist/linux-musl/
  cp target/x86_64-unknown-linux-musl/release/airwave dist/linux-musl/
  tar -czf dist/release/airwave-x86_64-unknown-linux-musl.tar.gz dist/linux-musl/

release-windows-gnu: build-windows-gnu prepare-release
  cp -rL build_assets/windows/* dist/windows-gnu/
  cp target/x86_64-pc-windows-gnu/release/airwave.exe dist/windows-gnu/
  zip -r dist/release/airwave-x86_64-pc-windows-gnu.zip dist/windows-gnu/

build: build-linux-gnu build-linux-musl build-windows-gnu
release: release-linux-gnu release-linux-musl release-windows-gnu
