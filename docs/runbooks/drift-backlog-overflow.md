# Drift Backlog Overflow Alert

**Alert**: Drift backlog > 100 items  
**Severity**: SEV-1 (Critical)  
**Business Hours**: Pages immediately, 24/7

## Alert Description

The drift backlog has exceeded 100 items, indicating that the system is accumulating more drift detection events than it can process. This suggests either a processing bottleneck or an unusually high rate of drift detection.

**Prometheus Query**:
```promql
drift_backlog_size > 100
```

## Impact Assessment

- **User Impact**: Delayed drift notifications and potential data staleness
- **Business Impact**: Reduced confidence in drift detection accuracy
- **Operational Impact**: Potential memory pressure and processing delays

## Immediate Actions (0-5 minutes)

1. **Acknowledge the alert** in PagerDuty
2. **Check drift processor status**:
   ```bash
   kubectl get pods -n spec-to-proof -l app=drift-processor
   kubectl get pods -n spec-to-proof -l app=drift-detector
   ```
3. **Check drift backlog metrics**:
   ```bash
   curl "http://prometheus:9090/api/v1/query?query=drift_backlog_size"
   curl "http://prometheus:9090/api/v1/query?query=drift_processing_rate"
   ```
4. **Check drift processor logs**:
   ```bash
   kubectl logs -f deployment/drift-processor --tail=100
   ```

## Investigation (5-15 minutes)

### 1. Analyze Backlog Composition
```bash
# Check drift types in backlog
curl "http://prometheus:9090/api/v1/query?query=drift_backlog_by_type"

# Check drift age distribution
curl "http://prometheus:9090/api/v1/query?query=drift_backlog_age_seconds"
```

### 2. Check Processing Capacity
```bash
# Check drift processor resource usage
kubectl top pods -n spec-to-proof -l app=drift-processor

# Check drift processor configuration
kubectl get configmap drift-processor-config -n spec-to-proof -o yaml
```

### 3. Check Data Sources
```bash
# Check ingestion rates
curl "http://prometheus:9090/api/v1/query?query=rate(ingestion_events_total[5m])"

# Check connector health
kubectl get pods -n spec-to-proof -l app=connector
```

### 4. Check Storage and Queue Health
```bash
# Check Redis queue size
kubectl exec -it deployment/redis -- redis-cli llen drift_queue

# Check database connection pool
kubectl logs deployment/drift-processor | grep -i "connection pool"
```

## Resolution

### If drift processor is down:
```bash
# Restart drift processor
kubectl rollout restart deployment/drift-processor

# Wait for rollout
kubectl rollout status deployment/drift-processor
```

### If processing is slow:
```bash
# Scale up drift processor
kubectl scale deployment/drift-processor --replicas=5

# Check if HPA is working
kubectl get hpa -n spec-to-proof -l app=drift-processor
```

### If memory pressure:
```bash
# Check memory usage
kubectl top pods -n spec-to-proof -l app=drift-processor

# Increase memory limits if needed
kubectl patch deployment/drift-processor -p '{"spec":{"template":{"spec":{"containers":[{"name":"drift-processor","resources":{"limits":{"memory":"2Gi"}}}]}}}}'
```

### If high ingestion rate:
1. **Temporary solution**: Increase drift processor replicas
2. **Investigate**: Check if there's a data source issue
3. **Optimize**: Review drift detection algorithms

## Prevention

### Short-term (1-7 days)
1. **Add backlog monitoring** - Alert at 50 items
2. **Implement backpressure** - Slow down ingestion if needed
3. **Add drift prioritization** - Process critical drifts first

### Long-term (1-4 weeks)
1. **Optimize drift detection** - Reduce false positives
2. **Implement drift batching** - Process drifts in batches
3. **Add drift archiving** - Archive old drifts automatically
4. **Implement drift sampling** - Sample drifts during high load

## Post-Mortem

### Required Documentation
1. **Timeline** - When alert fired, when resolved
2. **Root cause** - What caused the backlog
3. **Impact** - Number of affected drifts/users
4. **Actions taken** - What was done to resolve
5. **Lessons learned** - What could be improved

### Follow-up Tasks
- [ ] Update runbook based on lessons learned
- [ ] Implement preventive measures
- [ ] Review drift detection algorithms
- [ ] Update processing capacity planning

## Related Runbooks
- [High Latency](./high-latency.md)
- [Resource Exhaustion](./resource-exhaustion.md)
- [Service Unavailable](./service-unavailable.md) 