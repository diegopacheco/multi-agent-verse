#!/bin/bash
set -e

BASE_URL="http://127.0.0.1:8080"
FRONTEND_URL="http://127.0.0.1:5173"

echo "E2E Test Suite for Multi-Agent Verse"
echo "======================================"

check_backend() {
    echo -n "Checking backend health... "
    response=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health" 2>/dev/null || echo "000")
    if [ "$response" = "200" ]; then
        echo "OK"
        return 0
    else
        echo "FAILED (HTTP $response)"
        return 1
    fi
}

check_frontend() {
    echo -n "Checking frontend... "
    response=$(curl -s -o /dev/null -w "%{http_code}" "$FRONTEND_URL" 2>/dev/null || echo "000")
    if [ "$response" = "200" ]; then
        echo "OK"
        return 0
    else
        echo "FAILED (HTTP $response)"
        return 1
    fi
}

test_create_session() {
    echo -n "Testing session creation... "
    response=$(curl -s -X POST "$BASE_URL/api/session" \
        -H "Content-Type: application/json" \
        -d '{"model":"opus-4-5","cli_agent":"claude-code","worker_count":2,"tester_count":1}')
    session_id=$(echo "$response" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
    if [ -n "$session_id" ]; then
        echo "OK (session: $session_id)"
        echo "$session_id"
        return 0
    else
        echo "FAILED"
        return 1
    fi
}

test_get_status() {
    local session_id="$1"
    echo -n "Testing status endpoint... "
    response=$(curl -s "$BASE_URL/api/status/$session_id")
    if echo "$response" | grep -q "task_splitter"; then
        echo "OK"
        return 0
    else
        echo "FAILED"
        return 1
    fi
}

test_get_tasks() {
    local session_id="$1"
    echo -n "Testing tasks endpoint... "
    response=$(curl -s "$BASE_URL/api/tasks/$session_id")
    if echo "$response" | grep -q "tasks"; then
        echo "OK"
        return 0
    else
        echo "FAILED"
        return 1
    fi
}

test_get_events() {
    local session_id="$1"
    echo -n "Testing events endpoint... "
    response=$(curl -s "$BASE_URL/api/events/$session_id")
    if echo "$response" | grep -q "events"; then
        echo "OK"
        return 0
    else
        echo "FAILED"
        return 1
    fi
}

test_get_projects() {
    echo -n "Testing projects endpoint... "
    response=$(curl -s "$BASE_URL/api/projects")
    if echo "$response" | grep -q "projects"; then
        echo "OK"
        return 0
    else
        echo "FAILED"
        return 1
    fi
}

echo ""
echo "Pre-flight checks:"
echo "------------------"

if ! check_backend; then
    echo ""
    echo "ERROR: Backend is not running. Start with ./run.sh first."
    exit 1
fi

if ! check_frontend; then
    echo ""
    echo "WARNING: Frontend is not running. Some tests may be skipped."
fi

echo ""
echo "API Tests:"
echo "----------"

session_id=$(test_create_session)
if [ $? -eq 0 ] && [ -n "$session_id" ]; then
    test_get_status "$session_id"
    test_get_tasks "$session_id"
    test_get_events "$session_id"
fi

test_get_projects

echo ""
echo "======================================"
echo "E2E Tests Complete"
