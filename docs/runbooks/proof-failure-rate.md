# Proof Failure Rate Alert

**Alert**: Proof failure rate > 5% for 5 minutes  
**Severity**: SEV-1 (Critical)  
**Business Hours**: Pages immediately, 24/7

## Alert Description

The proof failure rate has exceeded 5% for the last 5 minutes. This indicates that a significant portion of proof generation attempts are failing, which directly impacts the platform's core functionality.

**Prometheus Query**:
```promql
rate(proof_failures_total[5m]) / rate(proof_attempts_total[5m]) > 0.05
```

## Impact Assessment

- **User Impact**: Users cannot generate proofs for their specifications
- **Business Impact**: Core platform functionality is degraded
- **Revenue Impact**: Potential loss of customers due to service unavailability

## Immediate Actions (0-5 minutes)

1. **Acknowledge the alert** in PagerDuty
2. **Check service status**:
   ```bash
   kubectl get pods -n spec-to-proof -l app=lean-farm
   kubectl get pods -n spec-to-proof -l app=proof-service
   ```
3. **Check recent logs**:
   ```bash
   kubectl logs -f deployment/lean-farm --tail=100
   kubectl logs -f deployment/proof-service --tail=100
   ```
4. **Verify Lean Farm health**:
   ```bash
   curl http://lean-farm:8080/health
   ```

## Investigation (5-15 minutes)

### 1. Check Proof Service Metrics
```bash
# Query Prometheus for proof failure details
curl "http://prometheus:9090/api/v1/query?query=proof_failures_total"
curl "http://prometheus:9090/api/v1/query?query=proof_attempts_total"
```

### 2. Check Lean Farm Status
```bash
# Check Lean Farm resource usage
kubectl top pods -n spec-to-proof -l app=lean-farm

# Check Lean Farm logs for errors
kubectl logs deployment/lean-farm --tail=200 | grep -i error
```

### 3. Check Infrastructure
```bash
# Check node resources
kubectl top nodes

# Check persistent volumes
kubectl get pvc -n spec-to-proof
```

### 4. Check External Dependencies
```bash
# Check Claude API status
curl -H "Authorization: Bearer $CLAUDE_API_KEY" \
  https://api.anthropic.com/v1/messages

# Check S3 access
aws s3 ls s3://spec-to-proof-artifacts/
```

## Resolution

### If Lean Farm is down:
```bash
# Restart Lean Farm
kubectl rollout restart deployment/lean-farm

# Wait for rollout
kubectl rollout status deployment/lean-farm
```

### If proof service is down:
```bash
# Restart proof service
kubectl rollout restart deployment/proof-service

# Wait for rollout
kubectl rollout status deployment/proof-service
```

### If resource exhaustion:
```bash
# Scale up Lean Farm
kubectl scale deployment/lean-farm --replicas=5

# Check if HPA is working
kubectl get hpa -n spec-to-proof
```

### If Claude API issues:
1. Check Claude API status page
2. Verify API key validity
3. Check rate limits
4. Consider switching to backup model

## Prevention

### Short-term (1-7 days)
1. **Increase monitoring** - Add more granular proof failure metrics
2. **Improve alerting** - Add early warning at 2% failure rate
3. **Add circuit breakers** - Prevent cascading failures

### Long-term (1-4 weeks)
1. **Implement retry logic** with exponential backoff
2. **Add proof queue** with priority handling
3. **Implement fallback proof strategies**
4. **Add load testing** to identify capacity limits

## Post-Mortem

### Required Documentation
1. **Timeline** - When alert fired, when resolved
2. **Root cause** - What caused the failure
3. **Impact** - Number of affected users/proofs
4. **Actions taken** - What was done to resolve
5. **Lessons learned** - What could be improved

### Follow-up Tasks
- [ ] Update runbook based on lessons learned
- [ ] Implement preventive measures
- [ ] Schedule capacity planning review
- [ ] Update monitoring thresholds if needed

## Related Runbooks
- [High Latency](./high-latency.md)
- [Resource Exhaustion](./resource-exhaustion.md)
- [Service Unavailable](./service-unavailable.md) 