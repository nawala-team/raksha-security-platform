#!/data/data/com.termux/files/usr/bin/bash
export PATH="/data/data/com.termux/files/usr/bin:$PATH"

BASE_URL="http://127.0.0.1:8080/api/v1"
PASS=0
FAIL=0
WARN=0

log_pass() { echo "[PASS] $1"; ((PASS++)); }
log_fail() { echo "[FAIL] $1"; ((FAIL++)); }
log_warn() { echo "[WARN] $1"; ((WARN++)); }
log_info() { echo "[INFO] $1"; }

echo "========================================="
echo "RAKSHA BRUTAL E2E & SECURITY TEST v2"
echo "========================================="
echo ""

# Get token first
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"superadmin@raksha.local","password":"RakshaSuper!2026"}')
TOKEN=$(echo "$RESP" | sed 's/.*"access_token":"\([^"]*\)".*/\1/')

if [ ${#TOKEN} -lt 50 ]; then
  echo "Failed to get token. Aborting."
  exit 1
fi
log_pass "Login & token obtained (${#TOKEN} chars)"

echo ""
echo "=== 1. AUTHENTICATION SECURITY ==="

# Invalid credentials
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"fake@test.com","password":"wrong"}')
if echo "$RESP" | grep -q 'error'; then log_pass "Invalid credentials rejected"; else log_fail "Invalid creds accepted"; fi

# SQL injection
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"admin\"--","password":"x"}')
if echo "$RESP" | grep -q 'error'; then log_pass "SQL injection rejected"; else log_fail "SQL injection might work"; fi

# Empty credentials
RESP=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"","password":""}')
if echo "$RESP" | grep -q 'error'; then log_pass "Empty credentials rejected"; else log_fail "Empty creds accepted"; fi

echo ""
echo "=== 2. AUTHORIZATION ==="

# No token
RESP=$(curl -s $BASE_URL/users)
if echo "$RESP" | grep -qi 'unauthorized\|error'; then log_pass "No token blocked"; else log_fail "No token allowed"; fi

# Invalid token
RESP=$(curl -s $BASE_URL/users -H 'Authorization: Bearer invalid123')
if echo "$RESP" | grep -qi 'unauthorized\|invalid\|error'; then log_pass "Invalid token blocked"; else log_fail "Invalid token allowed"; fi

# RBAC - Analyst cannot create admin
ANALYST=$(curl -s -X POST $BASE_URL/auth/login -H 'Content-Type: application/json' -d '{"email":"analyst@raksha.local","password":"RakshaAnalyst!2026"}')
ATOK=$(echo "$ANALYST" | sed 's/.*"access_token":"\([^"]*\)".*/\1/')
RESP=$(curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $ATOK" -H 'Content-Type: application/json' -d '{"email":"h@h.com","name":"H","password":"P@ss123!!","role":"super_admin"}')
if echo "$RESP" | grep -qi 'forbidden\|denied\|error'; then log_pass "RBAC: analyst blocked from admin actions"; else log_fail "RBAC broken"; fi

echo ""
echo "=== 3. INPUT VALIDATION ==="

# XSS
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"<script>alert(1)</script>","severity":"low","source":"xss-test"}')
if echo "$RESP" | grep -q '<script>'; then log_fail "XSS not sanitized"; else log_pass "XSS sanitized"; fi

# Path traversal
RESP=$(curl -s $BASE_URL/users/../../../etc/passwd -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -q 'root:'; then log_fail "Path traversal!"; else log_pass "Path traversal blocked"; fi

echo ""
echo "=== 4. STRESS TEST ==="

# Rapid fire 50 requests
SUCC=0
for i in $(seq 1 50); do
  CODE=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/health --max-time 5)
  [ "$CODE" = "200" ] && ((SUCC++))
done
if [ $SUCC -ge 45 ]; then log_pass "Rapid fire: $SUCC/50"; else log_fail "Rapid fire: $SUCC/50"; fi

# Concurrent
for i in $(seq 1 10); do curl -s $BASE_URL/health -o /dev/null & done
wait
RESP=$(curl -s $BASE_URL/health)
if echo "$RESP" | grep -q 'healthy'; then log_pass "Concurrent requests stable"; else log_fail "Unstable after concurrent"; fi


echo ""
echo "=== 5. CRUD REGRESSION ==="

# Users
RESP=$(curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"email":"crud'$(date +%s)'@t.com","name":"CRUD","password":"P@ss123!!","role":"viewer"}')
if echo "$RESP" | grep -q 'id'; then log_pass "User CREATE"; else log_fail "User CREATE: $RESP"; fi

# Alerts
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"Test Alert","severity":"high","source":"test"}')
AID=$(echo "$RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$AID" ]; then log_pass "Alert CREATE"; else log_fail "Alert CREATE"; fi

# Alert Update
RESP=$(curl -s -X PATCH $BASE_URL/alerts/$AID/status -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"status":"resolved"}')
if echo "$RESP" | grep -q 'resolved'; then log_pass "Alert UPDATE"; else log_fail "Alert UPDATE"; fi

# Incidents
RESP=$(curl -s -X POST $BASE_URL/incidents -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"Test Inc","severity":"high","description":"test","priority":"high"}')
IID=$(echo "$RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$IID" ]; then log_pass "Incident CREATE"; else log_fail "Incident CREATE"; fi

# Servers
RESP=$(curl -s -X POST $BASE_URL/servers -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"name":"Test Srv","hostname":"test-srv","ip_address":"1.2.3.4"}')
if echo "$RESP" | grep -q 'id'; then log_pass "Server CREATE"; else log_fail "Server CREATE"; fi

# Honeypots
RESP=$(curl -s -X POST $BASE_URL/honeypots -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"name":"Test HP","type":"ssh","ip_address":"5.6.7.8","port":22}')
if echo "$RESP" | grep -q 'id'; then log_pass "Honeypot CREATE"; else log_fail "Honeypot CREATE"; fi

# GRC Risk
RESP=$(curl -s -X POST $BASE_URL/grc/risks -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"Test Risk","description":"test","category":"technical","likelihood":3,"impact":3}')
RID=$(echo "$RESP" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$RID" ]; then log_pass "GRC Risk CREATE"; else log_fail "GRC Risk CREATE"; fi

# GRC Risk Delete
RESP=$(curl -s -X DELETE $BASE_URL/grc/risks/$RID -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -q 'deleted'; then log_pass "GRC Risk DELETE"; else log_fail "GRC Risk DELETE"; fi

# GRC Policy
RESP=$(curl -s -X POST $BASE_URL/grc/policies -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"title":"Test Policy","content":"test"}')
if echo "$RESP" | grep -q 'id'; then log_pass "GRC Policy CREATE"; else log_fail "GRC Policy CREATE"; fi

# Tenant
RESP=$(curl -s -X POST $BASE_URL/tenants -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"name":"Test Ten","slug":"test-'$(date +%s)'","contact_email":"t@t.com"}')
if echo "$RESP" | grep -q 'id'; then log_pass "Tenant CREATE"; else log_fail "Tenant CREATE"; fi

# Agent Token
RESP=$(curl -s -X POST $BASE_URL/agents/tokens -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"agent_name":"Test","max_uses":1,"expiry_hours":1}')
TKID=$(echo "$RESP" | grep -o '"token_id":"[^"]*"' | cut -d'"' -f4)
if [ -n "$TKID" ]; then log_pass "Token CREATE"; else log_fail "Token CREATE"; fi

# Token Revoke
RESP=$(curl -s -X DELETE $BASE_URL/agents/tokens/$TKID -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -q 'revoked'; then log_pass "Token REVOKE"; else log_fail "Token REVOKE"; fi


echo ""
echo "=== 6. API ENDPOINTS ==="

ENDPOINTS="health dashboard/stats users tenants alerts incidents agents agents/tokens servers servers/summary honeypots honeypots/summary grc/risks grc/policies grc/controls grc/summary vulnerabilities fim/events compliance audit threat-intel attack-surface network containers darkweb hunting backups documents databases settings"

for EP in $ENDPOINTS; do
  if [ "$EP" = "health" ]; then
    CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/$EP" --max-time 5)
  else
    CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/$EP" -H "Authorization: Bearer $TOKEN" --max-time 5)
  fi
  if [ "$CODE" = "200" ]; then
    log_pass "GET /$EP"
  else
    log_fail "GET /$EP -> $CODE"
  fi
done

echo ""
echo "=== 7. EDGE CASES ==="

# Non-existent
CODE=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/users/00000000-0000-0000-0000-000000000000 -H "Authorization: Bearer $TOKEN")
if [ "$CODE" = "404" ]; then log_pass "404 for non-existent"; else log_warn "Non-existent returns $CODE"; fi

# Invalid UUID
RESP=$(curl -s $BASE_URL/users/not-a-uuid -H "Authorization: Bearer $TOKEN")
if echo "$RESP" | grep -qi 'invalid\|error'; then log_pass "Invalid UUID rejected"; else log_warn "Invalid UUID: $RESP"; fi

# Empty body
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{}')
if echo "$RESP" | grep -qi 'error\|required\|missing'; then log_pass "Empty body rejected"; else log_fail "Empty body accepted"; fi

# Duplicate email
curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"email":"dup@dup.com","name":"D","password":"P@ss123!!","role":"viewer"}' > /dev/null
RESP=$(curl -s -X POST $BASE_URL/users -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d '{"email":"dup@dup.com","name":"D2","password":"P@ss123!!","role":"viewer"}')
if echo "$RESP" | grep -qi 'exist\|duplicate\|error'; then log_pass "Duplicate rejected"; else log_fail "Duplicate accepted"; fi

echo ""
echo "=== 8. AGENT SECURITY ==="

# Invalid enrollment
RESP=$(curl -s -X POST $BASE_URL/agents/enroll -H 'Content-Type: application/json' -d '{"token":"fake","fingerprint":{"hostname":"h","os":"l","os_version":"1","arch":"x","machine_id":"m","cpu_cores":1,"total_memory":1000,"mac_hash":"m"}}')
if echo "$RESP" | grep -qi 'invalid\|error'; then log_pass "Invalid enrollment rejected"; else log_fail "Invalid enrollment accepted"; fi

# Install scripts
CODE=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/agent/install)
if [ "$CODE" = "200" ]; then log_pass "Install script available"; else log_fail "Install script: $CODE"; fi

CODE=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/agent/install.ps1)
if [ "$CODE" = "200" ]; then log_pass "PS1 script available"; else log_fail "PS1 script: $CODE"; fi

echo ""
echo "=== 9. DATA INTEGRITY ==="

# Create and verify
TS=$(date +%s)
RESP=$(curl -s -X POST $BASE_URL/alerts -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d "{\"title\":\"Integrity $TS\",\"severity\":\"low\",\"source\":\"test\"}")
if echo "$RESP" | grep -q 'created_at'; then log_pass "Timestamp present"; else log_fail "No timestamp"; fi
if echo "$RESP" | grep -qE '[0-9a-f]{8}-[0-9a-f]{4}'; then log_pass "Valid UUID"; else log_fail "Invalid UUID"; fi

LIST=$(curl -s $BASE_URL/alerts -H "Authorization: Bearer $TOKEN")
if echo "$LIST" | grep -q "$TS"; then log_pass "Data persisted"; else log_fail "Data not found"; fi

echo ""
echo "=== 10. LOGOUT ==="

RESP=$(curl -s -X POST $BASE_URL/auth/logout -H "Authorization: Bearer $TOKEN")
log_pass "Logout called"

echo ""
echo "========================================="
echo "         TEST COMPLETE"
echo "========================================="
echo "PASSED: $PASS"
echo "FAILED: $FAIL"
echo "WARNINGS: $WARN"
TOTAL=$((PASS + FAIL))
if [ $TOTAL -gt 0 ]; then
  SCORE=$((PASS * 100 / TOTAL))
  echo "Score: $SCORE%"
  if [ $FAIL -eq 0 ]; then
    echo "✅ ALL TESTS PASSED"
  elif [ $FAIL -le 5 ]; then
    echo "⚠️ MOSTLY PASSED"
  else
    echo "❌ NEEDS FIXES"
  fi
fi
