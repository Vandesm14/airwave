# Installation

## Building from source

- **Backend:** [Rust and Cargo](https://www.rust-lang.org/tools/install)
- **Frontend:** [NodeJS](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs) and [pnpm](https://pnpm.io/installation) (you can install both with [volta](https://volta.sh)!)
- **Build:** [Just](https://just.systems)

**Step 1:**

Clone the repository.

```bash
git clone https://github.com/airwavegame/airwave
```

**Step 2:**

Build the release for your system.

```bash
# For Linux
just release-linux

# For Windows
just release-windows
```

**Step 3:**

Enter the directory of the release, such as `dist/linux` or `dist/windows`. This will contain all necessary files for Airwave to run, including all game assets and configuration.

## Set-Up and Configuration

### AI Features (requires OpenAI API key)

To use AI features that allow you to speak to aircraft and have your commands be interpreted via an LLM, it is required for you to bring your own [OpenAI](https://openai.com) [API key](https://platform.openai.com/api-keys).

*Note: In our testing of games with two players at an hour each, we've found that Airwave uses less than $0.30 per hour of gameplay, so you don't need to worry about high usage, even if you are constantly communicating with aircraft!*

Once you have obtained your API key, you will need to add it to an `.env` as shown in the `.env.example`:
```ini
# .env
OPENAI_API_KEY="<YOUR API KEY HERE>"
```

### Game-specific Configuration

Airwave provides a config spec to change things like the main airport, default frequencies, and other options for startup and gameplay. We include a `config.toml` file in the release directory, with the default options.

Airwave will read the `config.toml` on startup, changes to the config won't be read at runtime.

## Running a Singleplayer instance

To start a singleplayer (client and server) instance of Airwave, run the `client-server` script in the release directory (e.g. `dist/windows/client-server.exe` or `dist/linux/client-server.sh`).

Now, to access the game, go to `http://localhost:8080` in your browser of choice.

## Running a Multiplayer instance

Airwave always starts a multiplayer server under the hood, just like Minecraft. If you run the singleplayer instance, other players can connect, as long as you have set the server address to bind to `0.0.0.0` and you have forwarded port `8080` in your router.

### Connecting to a server

You can connect to an Airwave server by running the `client` script which will launch Airwave without the background game and server, relying on an external host to provide the game data.

Once you run the client, you can add the URL or IP of the server you are connecting to to the connection string of your client, like so: `localhost:8080?api=12.34.56.78:8080`.

### Running a client-less server

The Airwave game client is simply an extension to the server that serves static files (HTML, CSS, JS) and does not introduce any extra performance whether enabled or disabled.

Though not common and definitely not expected, you can run an Airwave server without serving the client by running the `server` script.
