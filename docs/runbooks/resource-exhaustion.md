# Resource Exhaustion Runbook

## Alert Description
- **Alert Name**: `HighCPUUsage` / `HighMemoryUsage`
- **Severity**: Warning
- **Business Hours Only**: Yes
- **Trigger**: CPU > 80% or Memory > 80% for 5 minutes

## Summary
System resources (CPU or Memory) are being heavily utilized, potentially leading to performance degradation or service failures.

## Impact
- **User Impact**: Slow response times, potential timeouts
- **Business Impact**: Poor performance, potential SLA violations
- **SLA Impact**: May violate performance SLAs

## Initial Assessment

### 1. Verify Alert
```bash
# Check current CPU usage
curl -s "http://prometheus:9090/api/v1/query?query=100 - (avg by(instance) (rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)"

# Check current memory usage
curl -s "http://prometheus:9090/api/v1/query?query=(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100"

# Check by pod
kubectl top pods -n spec-to-proof
```

### 2. Check System Resources
```bash
# Check node resources
kubectl top nodes

# Check pod resources
kubectl top pods -n spec-to-proof

# Check resource limits
kubectl describe pods -n spec-to-proof | grep -A 10 "Limits\|Requests"
```

## Investigation Steps

### Step 1: Identify Resource-Intensive Processes
```bash
# Check top processes in pods
kubectl exec -n spec-to-proof <pod-name> -- ps aux --sort=-%cpu | head -10
kubectl exec -n spec-to-proof <pod-name> -- ps aux --sort=-%mem | head -10

# Check Java heap if applicable
kubectl exec -n spec-to-proof <pod-name> -- jstat -gc <pid> 1000 10

# Check for memory leaks
kubectl exec -n spec-to-proof <pod-name> -- cat /proc/meminfo
```

### Step 2: Check Application Logs
```bash
# Check for memory-related errors
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "memory\|oom\|gc\|heap"

# Check for CPU-intensive operations
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "cpu\|performance\|slow"

# Check for infinite loops or long-running tasks
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100 | grep -i "loop\|task\|job"
```

### Step 3: Check Lean Farm Resources
```bash
# Check Lean Farm CPU usage
kubectl top pods -n spec-to-proof -l app=lean-farm

# Check Lean Farm memory usage
kubectl exec -n spec-to-proof <lean-farm-pod> -- free -h

# Check Lean Farm logs for resource issues
kubectl logs -n spec-to-proof deployment/lean-farm --tail=100 | grep -i "memory\|cpu\|resource"
```

### Step 4: Check Database Resources
```bash
# Check database CPU usage
kubectl top pods -n spec-to-proof -l app=postgresql

# Check database memory usage
kubectl exec -n spec-to-proof <db-pod> -- free -h

# Check database connections
kubectl exec -n spec-to-proof <pod-name> -- psql -h <db-host> -c "SELECT count(*) FROM pg_stat_activity;"
```

## Common Root Causes

### 1. Memory Leaks
**Symptoms**: Gradually increasing memory usage, OOM kills
**Resolution**:
```bash
# Check for memory leaks
kubectl exec -n spec-to-proof <pod-name> -- cat /proc/meminfo | grep -E "(MemTotal|MemFree|MemAvailable)"

# Restart service to clear memory
kubectl rollout restart deployment/spec-to-proof-api

# Increase memory limits
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"limits":{"memory":"4Gi"}}}]}}}}'
```

### 2. CPU-Intensive Operations
**Symptoms**: High CPU usage, slow response times
**Resolution**:
```bash
# Identify CPU-intensive processes
kubectl exec -n spec-to-proof <pod-name> -- top -b -n 1 | head -20

# Scale up CPU resources
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"limits":{"cpu":"2000m"}}}]}}}}'

# Scale out horizontally
kubectl scale deployment spec-to-proof-api --replicas=5
```

### 3. Lean Farm Overload
**Symptoms**: High CPU/Memory in Lean Farm pods
**Resolution**:
```bash
# Scale Lean Farm
kubectl scale deployment lean-farm --replicas=10

# Increase Lean Farm resources
kubectl patch deployment lean-farm -p '{"spec":{"template":{"spec":{"containers":[{"name":"lean-farm","resources":{"requests":{"memory":"2Gi","cpu":"1000m"},"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'

# Check Lean Farm queue
kubectl logs -n spec-to-proof deployment/lean-farm --tail=100 | grep -i "queue\|job"
```

### 4. Database Performance Issues
**Symptoms**: High CPU/Memory in database pods
**Resolution**:
```bash
# Check database performance
kubectl exec -n spec-to-proof <pod-name> -- psql -h <db-host> -c "SELECT query, mean_time FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"

# Restart database if needed
kubectl rollout restart deployment/postgresql

# Increase database resources
kubectl patch deployment postgresql -p '{"spec":{"template":{"spec":{"containers":[{"name":"postgresql","resources":{"requests":{"memory":"2Gi","cpu":"1000m"},"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'
```

### 5. External API Bottlenecks
**Symptoms**: High CPU due to external API calls
**Resolution**:
```bash
# Check external API response times
curl -w "@curl-format.txt" -o /dev/null -s "https://api.claude.ai/v1/messages"

# Implement caching
# Add request timeouts
# Implement circuit breakers
```

## Escalation

### Immediate Actions (0-5 minutes)
1. **Assess impact** - how many users affected?
2. **Check recent deployments** - did this start after a deployment?
3. **Post to #incidents** Slack channel

### Short-term Actions (5-15 minutes)
1. **Scale up resources** immediately
2. **Monitor resource usage** for improvement
3. **Begin deeper investigation**

### Medium-term Actions (15-60 minutes)
1. **Implement optimizations** (caching, query optimization)
2. **Coordinate with team** for architectural improvements
3. **Prepare capacity planning** meeting

## Resolution Steps

### 1. Quick Fixes
```bash
# Scale up resources
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"requests":{"memory":"2Gi","cpu":"1000m"},"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'

# Scale out horizontally
kubectl scale deployment spec-to-proof-api --replicas=5

# Scale Lean Farm
kubectl scale deployment lean-farm --replicas=10
```

### 2. Optimizations
```bash
# Implement caching
# Optimize database queries
# Add request timeouts
# Implement circuit breakers
```

### 3. Verify Resolution
```bash
# Monitor resource usage
watch -n 10 'kubectl top pods -n spec-to-proof'

# Check metrics
curl -s "http://prometheus:9090/api/v1/query?query=100 - (avg by(instance) (rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)"
```

### 4. Post-Incident
1. **Document resource improvements** needed
2. **Schedule capacity planning** meeting
3. **Implement monitoring improvements**

## Prevention

### Monitoring Improvements
- Add resource usage alerts for specific thresholds
- Monitor external API response times
- Set up alerting for resource trends

### Performance Improvements
- Implement caching strategies
- Optimize database queries
- Add connection pooling
- Implement request timeouts

### Infrastructure Improvements
- Auto-scaling policies
- Resource monitoring and alerting
- Load balancing optimization
- Regular capacity planning

## Contact Information
- **On-call**: Check PagerDuty
- **Slack**: #spec-to-proof-performance
- **Email**: performance@spec-to-proof.com

## Related Documentation
- [Performance Tuning Guide](../performance.md)
- [Capacity Planning](../capacity-planning.md)
- [Architecture Overview](../architecture.md) 