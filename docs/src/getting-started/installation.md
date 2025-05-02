# Installation

## Prerequisites

- **Backend:** [Rust and Cargo](https://www.rust-lang.org/tools/install)
- **Frontend:** [NodeJS](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs) and [pnpm](https://pnpm.io/installation) (you can install both with [volta](https://volta.sh)!)

**Step 1:**

Clone the repository

```bash
git clone https://github.com/vandesm14/airwave
```

**Step 2:**

Build the client (frontend)

```bash
pnpm i
pnpm client-web:build
cargo build --release --bin serve
```

**Step 3:**

Build the server (backend)

```bash
cargo build --release --bin server
```

## Running (Self-Hosted)

**Step 0:**

In the base directory of the project (where the README is), create a `.env` file with the following contents:

```env
OPENAI_API_KEY=<your-openai-api-key>
```

This is a requirement for the server to be able to generate reply text for incoming commands and to perform speech recognition (for voice commands).

**Step 1:**

Launch the server

```bash
cargo run --release --bin server
```

**Step 2:**

Launch the client

```bash
cargo run --release --bin serve client-web/dist
```

**Step 3:**

Open your browser of choice and go to http://localhost:8080 to connect to the client

## Running (Connecting to Another Server)

**Step 1:**

Launch the client

```bash
cargo run --release --bin serve client-web/dist
```

**Step 2:**

Open your browser of choice and go to http://localhost:8080?api=url, where `url` is the full URL of the server you want to connect to.

Example: http://localhost:8080?ws=https://myserver.com:9001