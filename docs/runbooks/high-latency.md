# High Latency Runbook

## Alert Description
- **Alert Name**: `HighLatency` / `APILatencyCritical`
- **Severity**: Warning (60s) / Critical (90s)
- **Business Hours Only**: Yes (Warning) / No (Critical)
- **Trigger**: P99 latency > 60s (warning) or > 90s (critical)

## Summary
API response times have exceeded acceptable thresholds, indicating performance degradation that may impact user experience.

## Impact
- **User Impact**: Slow response times, potential timeouts
- **Business Impact**: Poor user experience, potential SLA violations
- **SLA Impact**: May violate 90s P99 latency SLA

## Initial Assessment

### 1. Verify Alert
```bash
# Check current latency metrics
curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))"

# Check latency by endpoint
curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{endpoint=\"/api/v1/invariants\"}[5m]))"

# Check recent requests
curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total[5m])"
```

### 2. Check System Resources
```bash
# Check CPU usage
kubectl top pods -n spec-to-proof

# Check memory usage
kubectl exec -n spec-to-proof <pod-name> -- free -h

# Check disk I/O
kubectl exec -n spec-to-proof <pod-name> -- iostat -x 1 5
```

## Investigation Steps

### Step 1: Identify Slow Endpoints
```bash
# Get latency by endpoint
curl -s "http://prometheus:9090/api/v1/query?query=topk(5, histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])))"

# Check error rates by endpoint
curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m]) by (endpoint)"
```

### Step 2: Check Application Logs
```bash
# Check for slow queries
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "slow\|timeout\|duration"

# Check for errors
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "error\|exception"

# Check for resource exhaustion
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "memory\|cpu\|gc"
```

### Step 3: Check Dependencies
```bash
# Check database performance
kubectl logs -n spec-to-proof deployment/postgresql --tail=50 | grep -i "slow\|timeout"

# Check external API calls
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "http\|api\|external"

# Check Lean Farm status
kubectl logs -n spec-to-proof deployment/lean-farm --tail=50
```

### Step 4: Check Infrastructure
```bash
# Check node resources
kubectl top nodes

# Check network connectivity
kubectl exec -n spec-to-proof <pod-name> -- ping -c 5 <external-api>

# Check DNS resolution
kubectl exec -n spec-to-proof <pod-name> -- nslookup <external-api>
```

## Common Root Causes

### 1. Database Performance Issues
**Symptoms**: Slow queries, connection pool exhaustion
**Resolution**:
```bash
# Check database connections
kubectl exec -n spec-to-proof <pod-name> -- psql -h <db-host> -c "SELECT count(*) FROM pg_stat_activity;"

# Check slow queries
kubectl exec -n spec-to-proof <pod-name> -- psql -h <db-host> -c "SELECT query, mean_time FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"

# Restart database if needed
kubectl rollout restart deployment/postgresql
```

### 2. External API Slowdown
**Symptoms**: Timeouts to external services
**Resolution**:
```bash
# Check external API status
curl -w "@curl-format.txt" -o /dev/null -s "https://api.claude.ai/v1/messages"

# Implement circuit breaker
# Add timeout configurations
# Consider caching responses
```

### 3. Resource Exhaustion
**Symptoms**: High CPU/Memory usage, OOM kills
**Resolution**:
```bash
# Scale up resources
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"requests":{"memory":"2Gi","cpu":"1000m"},"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'

# Scale out horizontally
kubectl scale deployment spec-to-proof-api --replicas=5

# Check for memory leaks
kubectl exec -n spec-to-proof <pod-name> -- jstat -gc <pid> 1000 10
```

### 4. Network Issues
**Symptoms**: High latency to external services
**Resolution**:
```bash
# Check network policies
kubectl get networkpolicies -n spec-to-proof

# Check ingress controller
kubectl logs -n ingress-nginx deployment/ingress-nginx-controller --tail=50

# Check service mesh if applicable
kubectl get virtualservices -n spec-to-proof
```

### 5. Lean Farm Bottleneck
**Symptoms**: Proof generation taking too long
**Resolution**:
```bash
# Check Lean Farm queue
kubectl logs -n spec-to-proof deployment/lean-farm --tail=100 | grep -i "queue\|job"

# Scale Lean Farm
kubectl scale deployment lean-farm --replicas=10

# Check resource allocation
kubectl describe pod -n spec-to-proof -l app=lean-farm
```

## Escalation

### Immediate Actions (0-5 minutes)
1. **Assess impact** - how many users affected?
2. **Check recent deployments** - did this start after a deployment?
3. **Post to #incidents** Slack channel

### Short-term Actions (5-15 minutes)
1. **Implement quick fixes** (scale up, restart services)
2. **Monitor metrics** for improvement
3. **Begin deeper investigation**

### Medium-term Actions (15-60 minutes)
1. **Implement optimizations** (caching, query optimization)
2. **Coordinate with team** for architectural improvements
3. **Prepare customer communication** if needed

## Resolution Steps

### 1. Quick Fixes
```bash
# Scale up API pods
kubectl scale deployment spec-to-proof-api --replicas=5

# Scale up Lean Farm
kubectl scale deployment lean-farm --replicas=10

# Restart problematic services
kubectl rollout restart deployment/spec-to-proof-api
```

### 2. Optimizations
```bash
# Add caching headers
# Implement database connection pooling
# Add request timeouts
# Implement circuit breakers
```

### 3. Verify Resolution
```bash
# Monitor latency metrics
watch -n 10 'curl -s "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))"'

# Test specific endpoints
curl -w "@curl-format.txt" -o /dev/null -s "https://api.spec-to-proof.com/api/v1/invariants"
```

### 4. Post-Incident
1. **Document performance improvements** needed
2. **Schedule capacity planning** meeting
3. **Implement monitoring improvements**

## Prevention

### Monitoring Improvements
- Add latency alerts for specific endpoints
- Monitor external API response times
- Set up database performance monitoring

### Performance Improvements
- Implement caching strategies
- Optimize database queries
- Add connection pooling
- Implement request timeouts

### Infrastructure Improvements
- Auto-scaling policies
- Resource monitoring and alerting
- Load balancing optimization

## Contact Information
- **On-call**: Check PagerDuty
- **Slack**: #spec-to-proof-performance
- **Email**: performance@spec-to-proof.com

## Related Documentation
- [Performance Tuning Guide](../performance.md)
- [Database Optimization](../database.md)
- [Architecture Overview](../architecture.md) 