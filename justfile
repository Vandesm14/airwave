build-client:
  pnpm i
  pnpm client-web:build

build-linux: build-client
  cargo build --release --target x86_64-unknown-linux-gnu

build-windows: build-client
  cargo build --release --target x86_64-pc-windows-gnu

build: build-linux build-windows
