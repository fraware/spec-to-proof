# Service Unavailable Runbook

## Alert Description
- **Alert Name**: `ServiceUnavailable`
- **Severity**: Critical
- **Business Hours Only**: No (24/7)
- **Trigger**: `probe_success == 0` for 1 minute

## Summary
The service is not responding to health checks from the blackbox exporter. This indicates a complete service outage.

## Impact
- **User Impact**: Complete service unavailability
- **Business Impact**: High - no users can access the platform
- **SLA Impact**: Violates 99.9% uptime SLA

## Initial Assessment

### 1. Verify Alert
```bash
# Check if service is actually down
curl -f https://api.spec-to-proof.com/health
curl -f https://api.spec-to-proof.com/ready

# Check blackbox exporter logs
kubectl logs -n monitoring deployment/blackbox-exporter --tail=50
```

### 2. Check Service Status
```bash
# Check pod status
kubectl get pods -n spec-to-proof

# Check service endpoints
kubectl get endpoints -n spec-to-proof

# Check ingress status
kubectl get ingress -n spec-to-proof
```

## Investigation Steps

### Step 1: Infrastructure Check
```bash
# Check node status
kubectl get nodes
kubectl describe nodes

# Check cluster events
kubectl get events --sort-by='.lastTimestamp'

# Check resource usage
kubectl top nodes
kubectl top pods -n spec-to-proof
```

### Step 2: Application Check
```bash
# Check application logs
kubectl logs -n spec-to-proof deployment/spec-to-proof-api --tail=100

# Check if containers are running
kubectl describe pods -n spec-to-proof

# Check service configuration
kubectl get svc -n spec-to-proof -o yaml
```

### Step 3: Network Check
```bash
# Test DNS resolution
nslookup api.spec-to-proof.com

# Test connectivity
telnet api.spec-to-proof.com 443

# Check SSL certificate
openssl s_client -connect api.spec-to-proof.com:443 -servername api.spec-to-proof.com
```

## Common Root Causes

### 1. Pod Crash
**Symptoms**: Pod status shows `CrashLoopBackOff` or `Error`
**Resolution**:
```bash
# Check pod logs
kubectl logs -n spec-to-proof <pod-name> --previous

# Restart deployment
kubectl rollout restart deployment/spec-to-proof-api

# Check resource limits
kubectl describe pod <pod-name>
```

### 2. Resource Exhaustion
**Symptoms**: High CPU/Memory usage, OOM kills
**Resolution**:
```bash
# Scale up resources
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"requests":{"memory":"1Gi","cpu":"500m"},"limits":{"memory":"2Gi","cpu":"1000m"}}}]}}}}'

# Check for memory leaks
kubectl exec -n spec-to-proof <pod-name> -- ps aux
```

### 3. Network Issues
**Symptoms**: DNS resolution failures, SSL certificate issues
**Resolution**:
```bash
# Check ingress controller
kubectl get pods -n ingress-nginx
kubectl logs -n ingress-nginx deployment/ingress-nginx-controller

# Check certificate
kubectl get certificates -n spec-to-proof
kubectl describe certificate -n spec-to-proof <cert-name>
```

### 4. Database Issues
**Symptoms**: Database connection errors in logs
**Resolution**:
```bash
# Check database connectivity
kubectl exec -n spec-to-proof <pod-name> -- nc -zv <db-host> <db-port>

# Check database status
kubectl logs -n spec-to-proof deployment/postgresql
```

## Escalation

### Immediate Actions (0-5 minutes)
1. **Page on-call engineer** (if not already paged)
2. **Post to #incidents** Slack channel
3. **Update status page** to "Investigating"

### Short-term Actions (5-15 minutes)
1. **Assess impact** - how many users affected?
2. **Check recent deployments** - did this start after a deployment?
3. **Begin investigation** using steps above

### Medium-term Actions (15-60 minutes)
1. **Implement workaround** if possible
2. **Coordinate with team** for deeper investigation
3. **Prepare customer communication**

## Resolution Steps

### 1. Quick Fix (if applicable)
```bash
# Restart deployment
kubectl rollout restart deployment/spec-to-proof-api

# Scale up if resource issue
kubectl scale deployment spec-to-proof-api --replicas=3
```

### 2. Verify Resolution
```bash
# Check service health
curl -f https://api.spec-to-proof.com/health

# Check blackbox metrics
curl -s http://prometheus:9090/api/v1/query?query=probe_success

# Monitor for 5 minutes
watch -n 10 'curl -s https://api.spec-to-proof.com/health'
```

### 3. Post-Incident
1. **Update status page** to "Resolved"
2. **Document incident** in post-mortem
3. **Schedule follow-up** to prevent recurrence

## Prevention

### Monitoring Improvements
- Add more granular health checks
- Monitor resource usage proactively
- Set up alerting for resource thresholds

### Process Improvements
- Implement blue-green deployments
- Add circuit breakers for dependencies
- Improve logging and observability

### Infrastructure Improvements
- Add redundancy for critical components
- Implement auto-scaling policies
- Regular security and dependency updates

## Contact Information
- **On-call**: Check PagerDuty
- **Slack**: #spec-to-proof-alerts
- **Email**: alerts@spec-to-proof.com

## Related Documentation
- [Deployment Guide](../deployment.md)
- [Troubleshooting Guide](../troubleshooting.md)
- [Architecture Overview](../architecture.md) 