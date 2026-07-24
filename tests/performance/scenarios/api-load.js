/**
 * API Load Test Scenario for Raksha Security Platform.
 *
 * Simulates realistic API usage patterns including:
 * - Authentication
 * - Alert queries and creation
 * - Agent status checks
 * - Dashboard data fetching
 *
 * Run: k6 run tests/performance/scenarios/api-load.js
 */

import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';
import {
  API_BASE,
  defaultOptions,
  loadPatterns,
  getAuthToken,
  authHeaders,
  randomAlert,
} from '../k6-config.js';

// Custom metrics
const alertsCreated = new Counter('alerts_created');
const alertsFetched = new Counter('alerts_fetched');
const apiErrors = new Rate('api_errors');
const alertLatency = new Trend('alert_query_latency');

// Test configuration
export const options = {
  ...defaultOptions,
  stages: loadPatterns.average,
  scenarios: {
    api_load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: loadPatterns.average,
      gracefulRampDown: '30s',
    },
  },
};

// Setup: runs once before all VUs
export function setup() {
  const token = getAuthToken(http);
  if (!token) {
    throw new Error('Failed to authenticate during setup');
  }
  return { token };
}

// Main test function: each VU executes this repeatedly
export default function (data) {
  const { token } = data;
  const headers = authHeaders(token);

  group('Dashboard Data', () => {
    // Fetch alert statistics
    const statsRes = http.get(`${API_BASE}/alerts/stats`, {
      ...headers,
      tags: { type: 'api', endpoint: 'alert_stats' },
    });

    check(statsRes, {
      'alert stats status 200': (r) => r.status === 200,
      'alert stats has total': (r) => {
        const body = JSON.parse(r.body);
        return body.total !== undefined;
      },
    });

    apiErrors.add(statsRes.status !== 200);

    // Fetch agent status summary
    const agentsRes = http.get(`${API_BASE}/agents/status`, {
      ...headers,
      tags: { type: 'api', endpoint: 'agent_status' },
    });

    check(agentsRes, {
      'agents status 200': (r) => r.status === 200,
    });

    apiErrors.add(agentsRes.status !== 200);
  });

  sleep(1);

  group('Alert Operations', () => {
    // Fetch recent alerts with pagination
    const startTime = Date.now();
    const alertsRes = http.get(`${API_BASE}/alerts?page=1&per_page=20&status=open`, {
      ...headers,
      tags: { type: 'api', endpoint: 'list_alerts' },
    });

    alertLatency.add(Date.now() - startTime);

    check(alertsRes, {
      'list alerts status 200': (r) => r.status === 200,
      'list alerts returns array': (r) => {
        const body = JSON.parse(r.body);
        return Array.isArray(body.data || body.alerts || body);
      },
    });

    alertsFetched.add(1);
    apiErrors.add(alertsRes.status !== 200);

    // Create a new alert (simulating agent reporting)
    const alertPayload = randomAlert();
    const createRes = http.post(
      `${API_BASE}/alerts`,
      JSON.stringify(alertPayload),
      {
        ...headers,
        tags: { type: 'api', endpoint: 'create_alert' },
      }
    );

    check(createRes, {
      'create alert status 201': (r) => r.status === 201,
      'create alert returns id': (r) => {
        const body = JSON.parse(r.body);
        return body.id !== undefined;
      },
    });

    if (createRes.status === 201) {
      alertsCreated.add(1);
    }
    apiErrors.add(createRes.status !== 201);
  });

  sleep(1);

  group('Search and Filter', () => {
    // Search alerts by severity
    const searchRes = http.get(
      `${API_BASE}/alerts?severity=critical&page=1&per_page=10`,
      {
        ...headers,
        tags: { type: 'api', endpoint: 'search_alerts' },
      }
    );

    check(searchRes, {
      'search alerts status 200': (r) => r.status === 200,
    });

    apiErrors.add(searchRes.status !== 200);

    // Fetch specific alert detail
    const listRes = http.get(`${API_BASE}/alerts?page=1&per_page=1`, {
      ...headers,
      tags: { type: 'api', endpoint: 'list_one_alert' },
    });

    if (listRes.status === 200) {
      const body = JSON.parse(listRes.body);
      const alerts = body.data || body.alerts || body;
      if (Array.isArray(alerts) && alerts.length > 0) {
        const alertId = alerts[0].id;
        const detailRes = http.get(`${API_BASE}/alerts/${alertId}`, {
          ...headers,
          tags: { type: 'api', endpoint: 'alert_detail' },
        });

        check(detailRes, {
          'alert detail status 200': (r) => r.status === 200,
          'alert detail has id': (r) => {
            const detail = JSON.parse(r.body);
            return detail.id === alertId;
          },
        });
      }
    }
  });

  sleep(2);

  group('Agent Endpoints', () => {
    // List agents
    const agentsRes = http.get(`${API_BASE}/agents`, {
      ...headers,
      tags: { type: 'api', endpoint: 'list_agents' },
    });

    check(agentsRes, {
      'list agents status 200': (r) => r.status === 200,
    });

    apiErrors.add(agentsRes.status !== 200);

    // Simulate agent heartbeat
    const heartbeatRes = http.post(
      `${API_BASE}/agents/heartbeat`,
      JSON.stringify({
        agent_id: `perf-agent-${__VU}`,
        hostname: `perf-host-${__VU}`,
        status: 'online',
        metrics: {
          cpu_percent: Math.random() * 100,
          memory_percent: Math.random() * 100,
          events_per_second: Math.floor(Math.random() * 500),
        },
      }),
      {
        ...headers,
        tags: { type: 'api', endpoint: 'agent_heartbeat' },
      }
    );

    check(heartbeatRes, {
      'heartbeat status 200 or 201': (r) => r.status === 200 || r.status === 201,
    });
  });

  sleep(1);
}

// Teardown: runs once after all VUs finish
export function teardown(data) {
  console.log(`Performance test completed. Token used: ${data.token ? 'yes' : 'no'}`);
}
