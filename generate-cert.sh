#!/usr/bin/env bash

# non-interactive and 10 years expiration
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 3650 -nodes -subj "/C=US/ST=Pennsylvania/L=Mongomeryville/O=Airwave/OU=Airwave/CN=localhost"
