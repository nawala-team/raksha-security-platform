/**
 * k6 performance test configuration for Raksha Security Platform.
 *
 * Defines shared options, thresholds, and utility functions
 * used across performance test scenarios.
 *
 * Usage: k6 run --config k6-config.js scenarios/api-load.js
 */

export const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
export const API_BASE = `${BASE_URL}/api/v1`;

/**
 * Default test options with conservative thresholds.
 * Individual scenarios can override these.
 */
export const defaultOptions = {
  thresholds: {
    // 95th percentile response time should be under 500ms
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    // Less than 1% of requests should fail
    http_req_failed: ['rate<0.01'],
    // Custom metrics
    'http_req_duration{type:api}': ['p(95)<300'],
    'http_req_duration{type:auth}': ['p(95)<200'],
  },
  // Don't send metrics to cloud by default
  noConnectionReuse: false,
  userAgent: 'RakshaPerformanceTest/1.0',
};

/**
 * Stage definitions for common load patterns.
 */
export const loadPatterns = {
  // Smoke test: minimal load to verify system works
  smoke: [
    { duration: '1m', target: 1 },
  ],

  // Average load: normal traffic pattern
  average: [
    { duration: '2m', target: 10 },   // Ramp up
    { duration: '5m', target: 10 },   // Steady state
    { duration: '2m', target: 0 },    // Ramp down
  ],

  // Stress test: find breaking point
  stress: [
    { duration: '2m', target: 10 },
    { duration: '5m', target: 50 },
    { duration: '5m', target: 100 },
    { duration: '5m', target: 200 },
    { duration: '5m', target: 0 },
  ],

  // Spike test: sudden traffic surge
  spike: [
    { duration: '1m', target: 5 },
    { duration: '10s', target: 200 },  // Sudden spike
    { duration: '3m', target: 200 },   // Hold at spike
    { duration: '10s', target: 5 },    // Drop back
    { duration: '2m', target: 5 },     // Recovery
    { duration: '1m', target: 0 },
  ],

  // Soak test: extended duration for memory leaks
  soak: [
    { duration: '5m', target: 20 },
    { duration: '60m', target: 20 },
    { duration: '5m', target: 0 },
  ],
};

/**
 * Authenticate and return an access token.
 */
export function getAuthToken(http, email = 'perf-test@raksha.local', password = 'PerfTest123!') {
  const loginRes = http.post(
    `${API_BASE}/auth/login`,
    JSON.stringify({ email, password }),
    { headers: { 'Content-Type': 'application/json' }, tags: { type: 'auth' } }
  );

  if (loginRes.status !== 200) {
    console.error(`Auth failed: ${loginRes.status} - ${loginRes.body}`);
    return null;
  }

  const body = JSON.parse(loginRes.body);
  return body.access_token;
}

/**
 * Create authorized request headers.
 */
export function authHeaders(token) {
  return {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
  };
}

/**
 * Generate a random IP address for test data.
 */
export function randomIP() {
  return `${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`;
}

/**
 * Generate a random alert payload for testing.
 */
export function randomAlert() {
  const severities = ['critical', 'high', 'medium', 'low', 'info'];
  const types = ['network', 'process', 'file', 'auth', 'registry'];

  return {
    title: `Test alert ${Date.now()}`,
    severity: severities[Math.floor(Math.random() * severities.length)],
    type: types[Math.floor(Math.random() * types.length)],
    source_ip: randomIP(),
    timestamp: new Date().toISOString(),
  };
}
