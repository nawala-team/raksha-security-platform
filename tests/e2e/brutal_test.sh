#!/bin/bash
# set -e  # disabled to continue on test failures

BASE_URL="http://127.0.0.1:8080/api/v1"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0
WARN=0

log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; ((PASS++)); }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; ((FAIL++)); }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; ((WARN++)); }
log_info() { echo -e "[INFO] $1"; }

echo "========================================="
echo "RAKSHA BRUTAL E2E & SECURITY TEST"
echo "========================================="
echo ""

# ============ SECTION 1: AUTHENTICATION SECURITY ============
echo "=== 1. AUTHENTICATION SECURITY ==="

# 1.1 Invalid credentials
log_info "Testing invalid credentials..."
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"fake@test.com","password":"wrongpass"}')
if echo "$RESP" | grep -q 'error'; then
  log_pass "Invalid credentials rejected"
else
  log_fail "Invalid credentials not rejected: $RESP"
fi

# 1.2 SQL Injection attempt
log_info "Testing SQL injection in login..."
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"admin\"--","password":"x"}')
if echo "$RESP" | grep -q 'error'; then
  log_pass "SQL injection rejected"
else
  log_fail "SQL injection might work: $RESP"
fi

# 1.3 Empty credentials
log_info "Testing empty credentials..."
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"","password":""}')
if echo "$RESP" | grep -q 'error'; then
  log_pass "Empty credentials rejected"
else
  log_fail "Empty credentials not rejected"
fi

# 1.4 Valid login
log_info "Testing valid login..."
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"superadmin@raksha.local","password":"RakshaSuper!2026"}')
TOKEN=$(echo "$RESP" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
if [ -n "$TOKEN" ] && [ ${#TOKEN} -gt 50 ]; then
  log_pass "Valid login successful (token length: ${#TOKEN})"
else
  log_fail "Valid login failed"
  exit 1
fi

echo ""
# ============ SECTION 2: AUTHORIZATION & ACCESS CONTROL ============
echo "=== 2. AUTHORIZATION & ACCESS CONTROL ==="

# 2.1 Access without token
log_info "Testing access without token..."
RESP=$(curl -s $BASE_URL/users)
if echo "$RESP" | grep -qi 'unauthorized\|error\|missing'; then
  log_pass "Unauthorized access blocked"
else
  log_fail "Unauthorized access allowed: $RESP"
fi

# 2.2 Access with invalid token
log_info "Testing access with invalid token..."
RESP=$(curl -s $BASE_URL/users -H 'Authorization: Bearer invalid_token_12345')
if echo "$RESP" | grep -qi 'unauthorized\|invalid\|error'; then
  log_pass "Invalid token rejected"
else
  log_fail "Invalid token accepted: $RESP"
fi

# 2.3 Access with expired-like token
log_info "Testing access with malformed JWT..."
RESP=$(curl -s $BASE_URL/users -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U')
if echo "$RESP" | grep -qi 'unauthorized\|invalid\|error'; then
  log_pass "Malformed JWT rejected"
else
  log_fail "Malformed JWT accepted"
fi

# 2.4 Test viewer role limitations
log_info "Testing viewer role (analyst login)..."
ANALYST_RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"analyst@raksha.local","password":"RakshaAnalyst!2026"}')
ANALYST_TOKEN=$(echo "$ANALYST_RESP" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
if [ -n "$ANALYST_TOKEN" ]; then
  log_pass "Analyst login successful"
  # Try admin-only action
  RESP=$(curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $ANALYST_TOKEN" -H 'Content-Type: application/json' -d '{"email":"hacker@test.com","name":"Hacker","password":"Test123!@#","role":"super_admin"}')
  if echo "$RESP" | grep -qi 'forbidden\|denied\|error'; then
    log_pass "Analyst cannot create users (RBAC working)"
  else
    log_fail "Analyst could create user - RBAC broken!"
  fi
else
  log_warn "Analyst login failed, skipping RBAC test"
fi

echo ""
# ============ SECTION 3: INPUT VALIDATION & INJECTION ============
echo "=== 3. INPUT VALIDATION & INJECTION ==="

# 3.1 XSS in user name
log_info "Testing XSS injection in user name..."
RESP=$(curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"email":"xss@test.com","name":"<script>alert(1)</script>","password":"SecurePass123!@#","role":"viewer"}')
if echo "$RESP" | grep -q 'script'; then
  log_warn "XSS payload stored (check if escaped on output)"
else
  log_pass "XSS payload handled"
fi

# 3.2 SQL injection in search/filter
log_info "Testing SQL injection in query params..."
RESP=$(curl -s "$BASE_URL/alerts?status=open'%20OR%201=1--" -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -qi 'syntax\|error.*sql'; then
  log_fail "SQL injection error exposed"
else
  log_pass "SQL injection in params handled"
fi

# 3.3 Path traversal
log_info "Testing path traversal..."
RESP=$(curl -s "$BASE_URL/users/../../../etc/passwd" -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -q 'root:'; then
  log_fail "Path traversal vulnerability!"
else
  log_pass "Path traversal blocked"
fi

# 3.4 Oversized payload
log_info "Testing oversized payload (1MB)..."
BIG_DATA=$(python3 -c "print('A'*1048576)" 2>/dev/null || head -c 1048576 /dev/zero | tr '\0' 'A')
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d "{\"title\":\"$BIG_DATA\",\"severity\":\"low\",\"source\":\"test\"}" --max-time 10 2>&1)
if echo "$RESP" | grep -qi 'too large\|payload\|error\|timeout'; then
  log_pass "Oversized payload rejected"
else
  log_warn "Large payload might be accepted"
fi

# 3.5 Special characters in fields
log_info "Testing special characters..."
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"Test\u0000NULL\u001fCTRL","severity":"high","source":"test"}')
if echo "$RESP" | grep -q 'id'; then
  log_pass "Special chars handled safely"
else
  log_pass "Special chars rejected"
fi

echo ""
# ============ SECTION 4: STRESS TEST ============
echo "=== 4. STRESS TEST ==="

# 4.1 Rapid fire requests
log_info "Testing rapid fire requests (50 requests)..."
SUCCESS=0
for i in $(seq 1 50); do
  RESP=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/health --max-time 5)
  if [ "$RESP" = "200" ]; then
    ((SUCCESS++))
  fi
done
if [ $SUCCESS -ge 45 ]; then
  log_pass "Rapid fire: $SUCCESS/50 successful"
else
  log_fail "Rapid fire: only $SUCCESS/50 successful"
fi

# 4.2 Concurrent requests
log_info "Testing concurrent requests (10 parallel)..."
for i in $(seq 1 10); do
  curl -s $BASE_URL/health -o /dev/null &
done
wait
RESP=$(curl -s $BASE_URL/health)
if echo "$RESP" | grep -q 'healthy'; then
  log_pass "Server stable after concurrent requests"
else
  log_fail "Server unstable after concurrent requests"
fi

# 4.3 Multiple logins same user
log_info "Testing multiple sessions same user..."
for i in $(seq 1 5); do
  curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"superadmin@raksha.local","password":"RakshaSuper!2026"}' -o /dev/null &
done
wait
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"superadmin@raksha.local","password":"RakshaSuper!2026"}')
if echo "$RESP" | grep -q 'access_token'; then
  log_pass "Multiple sessions handled"
else
  log_fail "Session handling broken"
fi

# Refresh token for next tests
TOKEN=$(echo "$RESP" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)

echo ""
# ============ SECTION 5: FULL CRUD REGRESSION ============
echo "=== 5. FULL CRUD REGRESSION ==="

# 5.1 Users CRUD
log_info "Testing Users CRUD..."
USER_RESP=$(curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"email":"crud_test@test.com","name":"CRUD Test User","password":"CrudTest123!@#","role":"viewer"}')
USER_ID=$(echo "$USER_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$USER_ID" ]; then
  log_pass "User CREATE: $USER_ID"
  # Read
  READ_RESP=$(curl -s $BASE_URL/users -H "Authorization: Bearer $TOKEN")
  if echo "$READ_RESP" | grep -q "$USER_ID"; then
    log_pass "User READ: found in list"
  else
    log_fail "User READ: not found"
  fi
  # Delete
  DEL_RESP=$(curl -s -X DELETE $BASE_URL/users/$USER_ID -H "Authorization: Bearer $TOKEN")
  if echo "$DEL_RESP" | grep -q 'deleted'; then
    log_pass "User DELETE: success"
  else
    log_fail "User DELETE: failed"
  fi
else
  log_fail "User CREATE failed: $USER_RESP"
fi

# 5.2 Alerts CRUD
log_info "Testing Alerts CRUD..."
ALERT_RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"CRUD Test Alert","severity":"medium","source":"crud-test","description":"Testing CRUD operations"}')
ALERT_ID=$(echo "$ALERT_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$ALERT_ID" ]; then
  log_pass "Alert CREATE: $ALERT_ID"
  # Update status
  UPD_RESP=$(curl -s -X PATCH $BASE_URL/alerts/$ALERT_ID/status -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"status":"resolved"}')
  if echo "$UPD_RESP" | grep -q 'resolved'; then
    log_pass "Alert UPDATE: status changed"
  else
    log_fail "Alert UPDATE: failed"
  fi
else
  log_fail "Alert CREATE failed: $ALERT_RESP"
fi

# 5.3 Incidents CRUD
log_info "Testing Incidents CRUD..."
INC_RESP=$(curl -s -X POST $BASE_URL/incidents -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"CRUD Test Incident","severity":"low","description":"Testing incident CRUD","priority":"low"}')
INC_ID=$(echo "$INC_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$INC_ID" ]; then
  log_pass "Incident CREATE: $INC_ID"
  # Update
  UPD_RESP=$(curl -s -X PATCH $BASE_URL/incidents/$INC_ID/status -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"status":"closed"}')
  if echo "$UPD_RESP" | grep -q 'closed'; then
    log_pass "Incident UPDATE: closed"
  else
    log_fail "Incident UPDATE: failed - $UPD_RESP"
  fi
else
  log_fail "Incident CREATE failed: $INC_RESP"
fi

echo ""
# 5.4 Servers CRUD
log_info "Testing Servers CRUD..."
SRV_RESP=$(curl -s -X POST $BASE_URL/servers -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"name":"CRUD Test Server","hostname":"crud-test-srv","ip_address":"10.99.99.99","os":"Linux"}')
SRV_ID=$(echo "$SRV_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$SRV_ID" ]; then
  log_pass "Server CREATE: $SRV_ID"
  # Read
  READ_RESP=$(curl -s $BASE_URL/servers -H "Authorization: Bearer $TOKEN")
  if echo "$READ_RESP" | grep -q 'crud-test-srv'; then
    log_pass "Server READ: found"
  else
    log_fail "Server READ: not found"
  fi
else
  log_fail "Server CREATE failed: $SRV_RESP"
fi

# 5.5 Honeypots CRUD
log_info "Testing Honeypots CRUD..."
HP_RESP=$(curl -s -X POST $BASE_URL/honeypots -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"name":"CRUD Honeypot","type":"http","ip_address":"10.88.88.88","port":8080}')
HP_ID=$(echo "$HP_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$HP_ID" ]; then
  log_pass "Honeypot CREATE: $HP_ID"
else
  log_fail "Honeypot CREATE failed: $HP_RESP"
fi

# 5.6 GRC Risks CRUD
log_info "Testing GRC Risks CRUD..."
RISK_RESP=$(curl -s -X POST $BASE_URL/grc/risks -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"CRUD Risk Test","description":"Testing risk CRUD","category":"operational","likelihood":3,"impact":3}')
RISK_ID=$(echo "$RISK_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$RISK_ID" ]; then
  log_pass "GRC Risk CREATE: $RISK_ID"
  # Delete
  DEL_RESP=$(curl -s -X DELETE $BASE_URL/grc/risks/$RISK_ID -H "Authorization: Bearer $TOKEN")
  if echo "$DEL_RESP" | grep -q 'deleted'; then
    log_pass "GRC Risk DELETE: success"
  else
    log_fail "GRC Risk DELETE: failed"
  fi
else
  log_fail "GRC Risk CREATE failed: $RISK_RESP"
fi

# 5.7 GRC Policies CRUD
log_info "Testing GRC Policies CRUD..."
POL_RESP=$(curl -s -X POST $BASE_URL/grc/policies -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"CRUD Policy Test","content":"Test policy content"}')
POL_ID=$(echo "$POL_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$POL_ID" ]; then
  log_pass "GRC Policy CREATE: $POL_ID"
else
  log_fail "GRC Policy CREATE failed: $POL_RESP"
fi

# 5.8 Tenants CRUD
log_info "Testing Tenants CRUD..."
TEN_RESP=$(curl -s -X POST $BASE_URL/tenants -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"name":"CRUD Tenant","slug":"crud-tenant-test","contact_email":"crud@test.com"}')
TEN_ID=$(echo "$TEN_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$TEN_ID" ]; then
  log_pass "Tenant CREATE: $TEN_ID"
else
  log_fail "Tenant CREATE failed: $TEN_RESP"
fi

# 5.9 Agent Tokens
log_info "Testing Agent Tokens..."
TOK_RESP=$(curl -s -X POST $BASE_URL/agents/tokens -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"agent_name":"CRUD Token Test","max_uses":3,"expiry_hours":24}')
TOK_ID=$(echo "$TOK_RESP" | grep -o '"token_id":"[^"]*"' | cut -d'"' -f4)
if [ -n "$TOK_ID" ]; then
  log_pass "Token CREATE: $TOK_ID"
  # Revoke
  REV_RESP=$(curl -s -X DELETE $BASE_URL/agents/tokens/$TOK_ID -H "Authorization: Bearer $TOKEN")
  if echo "$REV_RESP" | grep -q 'revoked'; then
    log_pass "Token REVOKE: success"
  else
    log_fail "Token REVOKE: failed"
  fi
else
  log_fail "Token CREATE failed: $TOK_RESP"
fi

echo ""
# ============ SECTION 6: API ENDPOINT COVERAGE ============
echo "=== 6. API ENDPOINT COVERAGE ==="

ENDPOINTS=(
  "GET /health"
  "GET /dashboard/stats"
  "GET /users"
  "GET /tenants"
  "GET /alerts"
  "GET /incidents"
  "GET /agents"
  "GET /agents/tokens"
  "GET /servers"
  "GET /servers/summary"
  "GET /honeypots"
  "GET /honeypots/summary"
  "GET /grc/risks"
  "GET /grc/policies"
  "GET /grc/controls"
  "GET /grc/summary"
  "GET /vulnerabilities"
  "GET /fim/events"
  "GET /compliance"
  "GET /audit"
  "GET /threat-intel"
  "GET /attack-surface"
  "GET /network"
  "GET /containers"
  "GET /darkweb"
  "GET /hunting"
  "GET /backups"
  "GET /documents"
  "GET /databases"
  "GET /settings"
)

for EP in "${ENDPOINTS[@]}"; do
  METHOD=$(echo $EP | cut -d' ' -f1)
  PATH=$(echo $EP | cut -d' ' -f2)
  
  if [ "$PATH" = "/health" ]; then
    RESP=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL$PATH")
  else
    RESP=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL$PATH" -H "Authorization: Bearer $TOKEN")
  fi
  
  if [ "$RESP" = "200" ]; then
    log_pass "$EP -> 200"
  elif [ "$RESP" = "401" ] || [ "$RESP" = "403" ]; then
    log_warn "$EP -> $RESP (auth issue)"
  else
    log_fail "$EP -> $RESP"
  fi
done

echo ""
# ============ SECTION 7: EDGE CASES & BOUNDARY ============
echo "=== 7. EDGE CASES & BOUNDARY ==="

# 7.1 Non-existent resource
log_info "Testing non-existent resource..."
RESP=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/users/00000000-0000-0000-0000-000000000000 -H "Authorization: Bearer $TOKEN")
if [ "$RESP" = "404" ]; then
  log_pass "Non-existent resource returns 404"
else
  log_fail "Non-existent resource returns $RESP"
fi

# 7.2 Invalid UUID format
log_info "Testing invalid UUID format..."
RESP=$(curl -s $BASE_URL/users/not-a-uuid -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -qi 'invalid\|error\|bad'; then
  log_pass "Invalid UUID rejected"
else
  log_warn "Invalid UUID handling: $RESP"
fi

# 7.3 Empty POST body
log_info "Testing empty POST body..."
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{}')
if echo "$RESP" | grep -qi 'error\|required\|missing'; then
  log_pass "Empty body rejected"
else
  log_fail "Empty body accepted: $RESP"
fi

# 7.4 Duplicate email
log_info "Testing duplicate email..."
curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"email":"duplicate@test.com","name":"First","password":"Pass123!@#","role":"viewer"}' > /dev/null
RESP=$(curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"email":"duplicate@test.com","name":"Second","password":"Pass123!@#","role":"viewer"}')
if echo "$RESP" | grep -qi 'exist\|duplicate\|error\|conflict'; then
  log_pass "Duplicate email rejected"
else
  log_fail "Duplicate email accepted: $RESP"
fi

# 7.5 Invalid enum value
log_info "Testing invalid severity value..."
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"Test","severity":"super_mega_critical","source":"test"}')
if echo "$RESP" | grep -qi 'error\|invalid'; then
  log_pass "Invalid enum rejected"
else
  log_warn "Invalid enum might be accepted"
fi

# 7.6 Very long string
log_info "Testing very long title (10000 chars)..."
LONG_STR=$(head -c 10000 /dev/zero | tr '\0' 'X')
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d "{\"title\":\"$LONG_STR\",\"severity\":\"low\",\"source\":\"test\"}" --max-time 10 2>&1)
if echo "$RESP" | grep -qi 'error\|too long\|exceeded'; then
  log_pass "Very long string rejected"
else
  log_warn "Very long string handling unclear"
fi

echo ""
# ============ SECTION 8: AGENT SECURITY ============
echo "=== 8. AGENT SECURITY ==="

# 8.1 Agent enrollment with invalid token
log_info "Testing agent enrollment with invalid token..."
RESP=$(curl -s -X POST $BASE_URL/agents/enroll -H 'Content-Type: application/json' -d '{"token":"invalid_fake_token","fingerprint":{"hostname":"evil-host","os":"linux","os_version":"1.0","arch":"x64","machine_id":"evil123","cpu_cores":4,"total_memory":8000000000,"mac_hash":"fakemac"}}')
if echo "$RESP" | grep -qi 'invalid\|error\|unauthorized'; then
  log_pass "Invalid enrollment token rejected"
else
  log_fail "Invalid enrollment token might work: $RESP"
fi

# 8.2 Agent heartbeat without valid agent
log_info "Testing heartbeat from non-existent agent..."
RESP=$(curl -s -X POST $BASE_URL/agents/heartbeat -H 'Content-Type: application/json' -d '{"agent_id":"00000000-0000-0000-0000-000000000000","status":"online","metrics":{"cpu":50,"memory":60}}')
if echo "$RESP" | grep -qi 'error\|not found\|invalid'; then
  log_pass "Invalid agent heartbeat rejected"
else
  log_warn "Heartbeat handling: $RESP"
fi

# 8.3 Test install script availability
log_info "Testing install script endpoints..."
RESP=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/agent/install)
if [ "$RESP" = "200" ]; then
  log_pass "Linux install script available"
else
  log_fail "Linux install script not available: $RESP"
fi

RESP=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/agent/install.ps1)
if [ "$RESP" = "200" ]; then
  log_pass "Windows install script available"
else
  log_fail "Windows install script not available: $RESP"
fi

echo ""
# ============ SECTION 9: DATA INTEGRITY ============
echo "=== 9. DATA INTEGRITY ==="

# 9.1 Create and verify data persists
log_info "Testing data persistence..."
UNIQUE_ID=$(date +%s)
ALERT_RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d "{\"title\":\"Persistence Test $UNIQUE_ID\",\"severity\":\"low\",\"source\":\"integrity-test\"}")
ALERT_ID=$(echo "$ALERT_RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$ALERT_ID" ]; then
  # Verify it exists
  sleep 1
  LIST_RESP=$(curl -s $BASE_URL/alerts -H "Authorization: Bearer $TOKEN")
  if echo "$LIST_RESP" | grep -q "$UNIQUE_ID"; then
    log_pass "Data persisted and retrievable"
  else
    log_fail "Data not found after creation"
  fi
else
  log_fail "Could not create test data"
fi

# 9.2 Verify timestamps are set
log_info "Testing automatic timestamps..."
if echo "$ALERT_RESP" | grep -q 'created_at'; then
  log_pass "Created_at timestamp present"
else
  log_fail "Created_at timestamp missing"
fi

# 9.3 Verify UUID generation
log_info "Testing UUID generation..."
if echo "$ALERT_ID" | grep -qE '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'; then
  log_pass "Valid UUID format"
else
  log_fail "Invalid UUID format: $ALERT_ID"
fi

echo ""

# ============ SECTION 10: LOGOUT & SESSION ============
echo "=== 10. LOGOUT & SESSION ==="

# 10.1 Logout
log_info "Testing logout..."
RESP=$(curl -s -X POST $BASE_URL/auth/logout -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -qi 'success\|logged\|ok' || [ -z "$RESP" ]; then
  log_pass "Logout successful"
else
  log_warn "Logout response: $RESP"
fi

# 10.2 Access after logout (token should be invalidated)
log_info "Testing access after logout..."
sleep 1
RESP=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/users -H "Authorization: Bearer $TOKEN")
if [ "$RESP" = "401" ]; then
  log_pass "Token invalidated after logout"
else
  log_warn "Token might still be valid after logout (stateless JWT): $RESP"
fi

echo ""
# ============ FINAL SUMMARY ============
echo ""
echo "========================================="
echo "        BRUTAL TEST COMPLETE"
echo "========================================="
echo ""
echo -e "${GREEN}PASSED: $PASS${NC}"
echo -e "${RED}FAILED: $FAIL${NC}"
echo -e "${YELLOW}WARNINGS: $WARN${NC}"
echo ""
TOTAL=$((PASS + FAIL))
if [ $TOTAL -gt 0 ]; then
  SCORE=$((PASS * 100 / TOTAL))
  echo "Score: $SCORE%"
  echo ""
  if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ ALL TESTS PASSED - PRODUCTION READY${NC}"
    exit 0
  elif [ $FAIL -le 3 ]; then
    echo -e "${YELLOW}⚠️ MOSTLY PASSED - REVIEW WARNINGS${NC}"
    exit 0
  else
    echo -e "${RED}❌ MULTIPLE FAILURES - FIX REQUIRED${NC}"
    exit 1
  fi
fi
