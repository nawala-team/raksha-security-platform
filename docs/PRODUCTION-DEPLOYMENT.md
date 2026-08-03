# Production deployment

This deployment uses Docker Compose. The dashboard proxies browser requests from
`/api/*` to the internal `portal` service, so the database and portal do not
need to be exposed to browsers.

## Before deploying

1. Install Docker Engine 24+ and Docker Compose v2 on the server.
2. Clone this repository and copy the production environment template:

   ```sh
   cp .env.example .env
   ```

3. Set unique values for `POSTGRES_PASSWORD` and `JWT_SECRET`. Generate them
   with `openssl rand -base64 48`. Do not use the values from `.env.example`.
4. Set `RAKSHA__APP__ENVIRONMENT=production` and a production
   `GRAFANA_ADMIN_PASSWORD` if the monitoring profile will be enabled.
5. Restrict inbound traffic at the server firewall to the public web port
   (`WEB_PORT`, default `3000`) and your HTTPS reverse proxy port. Do not expose
   database, Redis, OpenSearch, or internal portal ports to the internet.

## Start and verify

```sh
docker compose up -d --build
docker compose ps
curl -f http://127.0.0.1:${WEB_PORT:-3000}/
docker compose logs --tail=100 web portal
```

Place a TLS reverse proxy (for example Nginx or Caddy) in front of the web
service and terminate HTTPS there. Configure its upstream to the web port; do
not point browsers directly at the `portal` service.

## Updates

```sh
git pull --ff-only
docker compose up -d --build
docker compose ps
```
