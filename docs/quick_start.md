# Quick Start (Docker Compose Using Pre-built Public Images)

This is the fastest way to get a local instance of VSL and the explorer running.

### Prerequisites

1. Docker + Docker Compose installed and running.

## Steps

This project's docker-compose file uses pre-built images for the `vsl-core` service and its
necessary components the `vsl-explorer` frontend and backend.

### Set environment variables

Copy the sample environment file:

```bash
cp test.env 
```
And change variables if desired.

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