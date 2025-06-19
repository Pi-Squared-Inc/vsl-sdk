# Quick Start (Docker Compose Using Pre-built Public Images)

This is the fastest way to get a local instance of VSL and the explorer running.

## Prerequisites

1. Docker + Docker Compose installed and running.

## Steps

This project's docker-compose file uses pre-built images for the `vsl-core` service and its
necessary components the `vsl-explorer` frontend and backend.

### Set environment variables

Create a `.env` file with master account (set `VSL_MASTER_ADDR` environment variable).


### Select whether you need persistent storage

The [docker-compose.public.yml](../docker-compose.public.yml) file has two configuration options for data persistence. If you want to start from scratch and delete all existing VSL data, keep the default uncommented command. To reuse existing data volumes, replace it with the commented command below.

```yml
command: # Fresh start with master account
      [
        "--master-account",
        '{"account": "$VSL_MASTER_ADDR", "initial_balance": "100000"}',
        "--claim-db-path",
        "/var/lib/vsl/vsl-db",
        "--tokens-db-path",
        "/var/lib/vsl/tokens.db",
      ]
    # command: # Use existing database
    #   [
    #     "--claim-db-path",
    #     "/var/lib/vsl/vsl-db",
    #     "--tokens-db-path",
    #     "/var/lib/vsl/tokens.db",
    #   ]
```

### Run the Service

Pull the relevant packages:
```bash
docker compose -f docker-compose.public.yml pull
```

Start the multi-container environment:
```bash
docker compose -f docker-compose.public.yml up
```
Or, to run it in background:
```bash
docker compose -f docker-compose.public.yml up -d
```

This runs the `vsl-core` service and the `vsl-explorer`. It maps:

- Container port 44444 (used by the JSON-RPC service) to host port 44444.
- Container port 4000 (used by the Explorer frontend) to host port 4000.

If deployed locally, the following services become available:
- VSL Core JSON-RPC: http://localhost:44444
- VSL Explorer UI: http://localhost:4000

To test accessibility to the RPC service, you may use something like:

```bash
curl -X POST http://localhost:44444 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "vsl_getHealth",
    "params": {}
  }'
```

You can view the explorer in the browser at `http://localhost:4000`. 
