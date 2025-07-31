# High Error Rate Runbook

## Alert Description
- **Alert Name**: `HighErrorRate`
- **Severity**: Warning
- **Business Hours Only**: Yes
- **Trigger**: Error rate > 2% for 5 minutes

## Summary
The API is returning a high percentage of error responses (5xx status codes), indicating system instability or issues.

## Impact
- **User Impact**: Failed requests, poor user experience
- **Business Impact**: Reduced reliability, potential SLA violations
- **SLA Impact**: May violate 99.9% availability SLA

## Initial Assessment

### 1. Verify Alert
```bash
# Check current error rate
curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m]) / rate(http_requests_total[5m])"

# Check error rate by endpoint
curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m]) by (endpoint)"

# Check specific error codes
curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m]) by (status)"
```

### 2. Check Application Logs
```bash
# Check recent errors
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "error\|exception\|500\|502\|503"

# Check for stack traces
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -A 10 -B 5 "Exception\|Error"

# Check for OOM kills
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "oom\|memory"
```

## Investigation Steps

### Step 1: Identify Error Patterns
```bash
# Get top error endpoints
curl -s "http://prometheus:9090/api/v1/query?query=topk(5, rate(http_requests_total{status=~\"5..\"}[5m]) by (endpoint))"

# Check error rate over time
curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m])"

# Check by status code
curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m]) by (status)"
```

### Step 2: Check System Resources
```bash
# Check CPU usage
kubectl top pods -n spec-to-proof

# Check memory usage
kubectl exec -n spec-to-proof <pod-name> -- free -h

# Check disk space
kubectl exec -n spec-to-proof <pod-name> -- df -h

# Check file descriptors
kubectl exec -n spec-to-proof <pod-name> -- cat /proc/sys/fs/file-nr
```

### Step 3: Check Dependencies
```bash
# Check database connectivity
kubectl exec -n spec-to-proof <pod-name> -- nc -zv <db-host> <db-port>

# Check external API status
curl -s "https://api.claude.ai/v1/messages" -H "Authorization: Bearer $CLAUDE_API_KEY"

# Check Lean Farm status
kubectl logs -n spec-to-proof deployment/lean-farm --tail=50 | grep -i "error\|exception"
```

### Step 4: Check Infrastructure
```bash
# Check pod status
kubectl get pods -n spec-to-proof

# Check service endpoints
kubectl get endpoints -n spec-to-proof

# Check ingress status
kubectl get ingress -n spec-to-proof
```

## Common Root Causes

### 1. Database Connection Issues
**Symptoms**: 500 errors, database connection timeouts
**Resolution**:
```bash
# Check database connections
kubectl exec -n spec-to-proof <pod-name> -- psql -h <db-host> -c "SELECT count(*) FROM pg_stat_activity;"

# Check database logs
kubectl logs -n spec-to-proof deployment/postgresql --tail=50

# Restart database if needed
kubectl rollout restart deployment/postgresql
```

### 2. External API Failures
**Symptoms**: 502/503 errors, timeouts to external services
**Resolution**:
```bash
# Check external API status
curl -s "https://api.claude.ai/v1/messages" -H "Authorization: Bearer $CLAUDE_API_KEY"

# Implement circuit breaker
# Add retry logic
# Consider fallback mechanisms
```

### 3. Resource Exhaustion
**Symptoms**: 503 errors, OOM kills, high CPU/Memory
**Resolution**:
```bash
# Scale up resources
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"requests":{"memory":"2Gi","cpu":"1000m"},"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'

# Scale out horizontally
kubectl scale deployment spec-to-proof-api --replicas=5

# Check for memory leaks
kubectl exec -n spec-to-proof <pod-name> -- jstat -gc <pid> 1000 10
```

### 4. Application Bugs
**Symptoms**: 500 errors, stack traces in logs
**Resolution**:
```bash
# Check recent deployments
kubectl rollout history deployment/spec-to-proof-api

# Rollback if needed
kubectl rollout undo deployment/spec-to-proof-api

# Check configuration
kubectl get configmap -n spec-to-proof -o yaml
```

### 5. Network Issues
**Symptoms**: 502/503 errors, connection timeouts
**Resolution**:
```bash
# Check network policies
kubectl get networkpolicies -n spec-to-proof

# Check ingress controller
kubectl logs -n ingress-nginx deployment/ingress-nginx-controller --tail=50

# Check service mesh if applicable
kubectl get virtualservices -n spec-to-proof
```

## Escalation

### Immediate Actions (0-5 minutes)
1. **Assess impact** - how many users affected?
2. **Check recent deployments** - did this start after a deployment?
3. **Post to #incidents** Slack channel

### Short-term Actions (5-15 minutes)
1. **Implement quick fixes** (restart services, scale up)
2. **Monitor error rates** for improvement
3. **Begin deeper investigation**

### Medium-term Actions (15-60 minutes)
1. **Implement fixes** (bug fixes, configuration changes)
2. **Coordinate with team** for root cause analysis
3. **Prepare customer communication** if needed

## Resolution Steps

### 1. Quick Fixes
```bash
# Restart problematic services
kubectl rollout restart deployment/spec-to-proof-api

# Scale up resources
kubectl scale deployment spec-to-proof-api --replicas=5

# Rollback recent deployment if needed
kubectl rollout undo deployment/spec-to-proof-api
```

### 2. Implement Fixes
```bash
# Fix configuration issues
kubectl patch configmap spec-to-proof-config -p '{"data":{"key":"value"}}'

# Update environment variables
kubectl set env deployment/spec-to-proof-api KEY=value

# Apply new deployment
kubectl apply -f k8s/deployment.yaml
```

### 3. Verify Resolution
```bash
# Monitor error rates
watch -n 10 'curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m]) / rate(http_requests_total[5m])"'

# Test specific endpoints
curl -s "https://api.spec-to-proof.com/api/v1/invariants" -H "Content-Type: application/json" -d '{"test":"data"}'

# Check application logs
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=50 | grep -i "error"
```

### 4. Post-Incident
1. **Document root cause** and resolution
2. **Implement monitoring improvements**
3. **Schedule follow-up** to prevent recurrence

## Prevention

### Monitoring Improvements
- Add error rate alerts for specific endpoints
- Monitor external API health
- Set up alerting for resource thresholds

### Process Improvements
- Implement blue-green deployments
- Add health checks for dependencies
- Improve error handling and logging

### Infrastructure Improvements
- Add redundancy for critical components
- Implement circuit breakers
- Regular security and dependency updates

## Contact Information
- **On-call**: Check PagerDuty
- **Slack**: #spec-to-proof-alerts
- **Email**: alerts@spec-to-proof.com

## Related Documentation
- [Error Handling Guide](../error-handling.md)
- [Deployment Guide](../deployment.md)
- [Troubleshooting Guide](../troubleshooting.md) 