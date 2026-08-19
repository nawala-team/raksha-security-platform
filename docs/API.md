#  Raksha API Documentation

> REST and GraphQL API reference for the Raksha Security Platform

## Base URL

```
Production:  https://your-domain.com/api/v1
Development: http://localhost:8080/api/v1
```

## Authentication

All API requests require authentication via JWT Bearer token.

```bash
curl -H "Authorization: Bearer <token>" https://your-domain.com/api/v1/...
```

### Obtain Token

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "your-password"
}
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 86400,
  "token_type": "Bearer"
}
```

### Refresh Token

```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

## API Endpoints

### Agent Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/agents` | List enrolled agents |
| GET | `/agents/:id` | Get agent details |
| GET | `/agents/:id/metrics` | Get agent metrics |
| POST | `/agents/enroll` | Enroll a new agent (public, requires token) |
| POST | `/agents/tokens` | Generate enrollment token (admin) |
| GET | `/agents/tokens` | List enrollment tokens (admin) |
| DELETE | `/agents/tokens/:id` | Revoke enrollment token (admin) |
| POST | `/agents/:id/rotate-certificate` | Rotate agent mTLS certificate |

### Server Monitor

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/servers` | List monitored servers |
| GET | `/servers/:id` | Get server details |
| POST | `/servers` | Register a new server |
| DELETE | `/servers/:id` | Remove a server |
| GET | `/servers/:id/metrics` | Get server metrics |
| GET | `/servers/:id/alerts` | Get server alerts |

### Network Scanner

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/scans` | List scan history |
| POST | `/scans` | Start a new scan |
| GET | `/scans/:id` | Get scan results |
| DELETE | `/scans/:id` | Delete scan results |
| GET | `/scans/:id/vulnerabilities` | Get discovered vulnerabilities |
| GET | `/network/topology` | Get network topology map |

### Database Guard

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/databases` | List monitored databases |
| POST | `/databases` | Register a database |
| GET | `/databases/:id/audit` | Get database audit log |
| GET | `/databases/:id/permissions` | Get permission analysis |
| POST | `/databases/:id/check` | Run security check |

### Compliance

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/compliance/standards` | List supported standards |
| POST | `/compliance/assess` | Start compliance assessment |
| GET | `/compliance/assessments/:id` | Get assessment results |
| GET | `/compliance/reports` | List compliance reports |
| GET | `/compliance/reports/:id/download` | Download report (PDF) |

### ML Threat Detection

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/ml/models` | List ML models |
| GET | `/ml/anomalies` | List detected anomalies |
| GET | `/ml/predictions` | Get threat predictions |
| POST | `/ml/train` | Trigger model retraining |

### Audit Trail

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/audit/events` | List audit events |
| GET | `/audit/events/:id` | Get event details |
| GET | `/audit/verify` | Verify audit chain integrity |
| GET | `/audit/export` | Export audit logs |

## Query Parameters

Common query parameters supported across list endpoints:

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | integer | Page number (default: 1) |
| `per_page` | integer | Items per page (default: 25, max: 100) |
| `sort` | string | Sort field (prefix with `-` for descending) |
| `filter` | string | Filter expression |
| `from` | datetime | Start date (ISO 8601) |
| `to` | datetime | End date (ISO 8601) |

## Error Responses

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid scan target",
    "details": [
      { "field": "target", "message": "must be a valid IP or CIDR range" }
    ]
  }
}
```

| Status Code | Meaning |
|-------------|---------|
| 400 | Bad Request — Invalid parameters |
| 401 | Unauthorized — Missing or invalid token |
| 403 | Forbidden — Insufficient permissions |
| 404 | Not Found — Resource does not exist |
| 429 | Too Many Requests — Rate limit exceeded |
| 500 | Internal Server Error |

## GraphQL

The GraphQL endpoint is available at `/api/v1/graphql`.

```graphql
query {
  servers(status: ONLINE) {
    id
    hostname
    metrics {
      cpuUsage
      memoryUsage
      diskUsage
    }
    alerts(severity: CRITICAL) {
      message
      timestamp
    }
  }
}
```

## WebSocket (Real-time)

Connect to `/api/v1/ws` for real-time updates:

```javascript
const ws = new WebSocket('wss://your-domain.com/api/v1/ws');
ws.send(JSON.stringify({ subscribe: ['alerts', 'metrics'] }));
```

## Rate Limits

| Tier | Requests/min | Burst |
|------|-------------|-------|
| Free | 60 | 10 |
| Pro | 600 | 50 |
| Enterprise | Unlimited | — |

---

*Part of the Nawala Ecosystem* 
