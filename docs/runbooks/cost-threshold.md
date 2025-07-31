# Cost Threshold Runbook

## Alert Description
- **Alert Name**: `CostThreshold` / `CostThresholdCritical`
- **Severity**: Info ($10K) / Critical ($15K)
- **Business Hours Only**: Yes
- **Trigger**: Monthly cost > $10K (info) or > $15K (critical)

## Summary
Monthly infrastructure costs have exceeded budget thresholds, requiring immediate attention to optimize spending.

## Impact
- **Business Impact**: Budget overruns, potential financial constraints
- **Operational Impact**: May require service scaling decisions
- **Strategic Impact**: Could affect pricing or feature decisions

## Initial Assessment

### 1. Verify Alert
```bash
# Check current monthly cost
curl -s "http://prometheus:9090/api/v1/query?query=monthly_cost_usd"

# Check cost by service
curl -s "http://prometheus:9090/api/v1/query?query=aws_cost_by_service"

# Check cost trends
curl -s "http://prometheus:9090/api/v1/query?query=rate(aws_cost_usd[24h])"
```

### 2. Check AWS Cost Explorer
```bash
# Check current month costs
aws ce get-cost-and-usage \
  --time-period Start=2024-01-01,End=2024-01-31 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --group-by Type=DIMENSION,Key=SERVICE

# Check cost by service
aws ce get-cost-and-usage \
  --time-period Start=2024-01-01,End=2024-01-31 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --group-by Type=DIMENSION,Key=SERVICE
```

## Investigation Steps

### Step 1: Identify High-Cost Services
```bash
# Get top cost services
curl -s "http://prometheus:9090/api/v1/query?query=topk(5, aws_cost_by_service)"

# Check EKS costs
curl -s "http://prometheus:9090/api/v1/query?query=aws_eks_cost_usd"

# Check RDS costs
curl -s "http://prometheus:9090/api/v1/query?query=aws_rds_cost_usd"

# Check Lambda costs
curl -s "http://prometheus:9090/api/v1/query?query=aws_lambda_cost_usd"
```

### Step 2: Check Resource Usage
```bash
# Check EKS node usage
kubectl top nodes

# Check pod resource requests vs usage
kubectl get pods -n spec-to-proof -o custom-columns="NAME:.metadata.name,CPU_REQ:.spec.containers[0].resources.requests.cpu,CPU_LIMIT:.spec.containers[0].resources.limits.cpu,MEM_REQ:.spec.containers[0].resources.requests.memory,MEM_LIMIT:.spec.containers[0].resources.limits.memory"

# Check storage usage
kubectl get pvc -n spec-to-proof
```

### Step 3: Check External API Costs
```bash
# Check Claude API usage
curl -s "http://prometheus:9090/api/v1/query?query=claude_api_cost_usd"

# Check API call volume
curl -s "http://prometheus:9090/api/v1/query?query=rate(claude_api_calls_total[24h])"

# Check average cost per request
curl -s "http://prometheus:9090/api/v1/query?query=claude_api_cost_usd / claude_api_calls_total"
```

### Step 4: Check Lean Farm Costs
```bash
# Check Lean Farm resource usage
kubectl top pods -n spec-to-proof -l app=lean-farm

# Check proof generation costs
curl -s "http://prometheus:9090/api/v1/query?query=proof_generation_cost_usd"

# Check queue size
curl -s "http://prometheus:9090/api/v1/query?query=lean_farm_queue_size"
```

## Common Root Causes

### 1. Over-Provisioned Resources
**Symptoms**: High EKS costs, underutilized nodes
**Resolution**:
```bash
# Check node utilization
kubectl top nodes

# Scale down underutilized nodes
kubectl scale deployment spec-to-proof-api --replicas=2

# Adjust resource requests
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"requests":{"memory":"512Mi","cpu":"250m"},"limits":{"memory":"1Gi","cpu":"500m"}}}]}}}}'
```

### 2. High External API Usage
**Symptoms**: High Claude API costs, excessive calls
**Resolution**:
```bash
# Check API call patterns
curl -s "http://prometheus:9090/api/v1/query?query=rate(claude_api_calls_total[1h]) by (endpoint)"

# Implement caching
# Add request batching
# Optimize prompt usage
```

### 3. Storage Costs
**Symptoms**: High EBS costs, excessive storage
**Resolution**:
```bash
# Check storage usage
kubectl get pvc -n spec-to-proof -o custom-columns="NAME:.metadata.name,SIZE:.spec.resources.requests.storage,USED:.status.capacity.storage"

# Clean up unused volumes
kubectl delete pvc <unused-pvc>

# Implement storage lifecycle policies
```

### 4. Lean Farm Overhead
**Symptoms**: High CPU/Memory costs for proof generation
**Resolution**:
```bash
# Optimize Lean Farm resources
kubectl patch deployment lean-farm -p '{"spec":{"template":{"spec":{"containers":[{"name":"lean-farm","resources":{"requests":{"memory":"1Gi","cpu":"500m"},"limits":{"memory":"2Gi","cpu":"1000m"}}}]}}}}'

# Implement job queuing
# Add proof result caching
```

### 5. Development/Testing Costs
**Symptoms**: High costs from non-production environments
**Resolution**:
```bash
# Check environment costs
aws ce get-cost-and-usage \
  --time-period Start=2024-01-01,End=2024-01-31 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --group-by Type=TAG,Key=Environment

# Scale down dev environments
kubectl scale deployment spec-to-proof-api --replicas=1 -n spec-to-proof-dev
```

## Escalation

### Immediate Actions (0-5 minutes)
1. **Assess cost breakdown** by service
2. **Post to #finance** Slack channel
3. **Notify management** if critical threshold exceeded

### Short-term Actions (5-15 minutes)
1. **Identify immediate cost savings** opportunities
2. **Scale down non-critical resources**
3. **Begin cost optimization** investigation

### Medium-term Actions (15-60 minutes)
1. **Implement cost optimizations**
2. **Coordinate with finance team**
3. **Prepare cost reduction plan**

## Resolution Steps

### 1. Immediate Cost Reductions
```bash
# Scale down non-critical services
kubectl scale deployment spec-to-proof-api --replicas=2

# Reduce resource requests
kubectl patch deployment spec-to-proof-api -p '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"requests":{"memory":"512Mi","cpu":"250m"}}}]}}}}'

# Clean up unused resources
kubectl delete pvc <unused-pvc>
```

### 2. Optimizations
```bash
# Implement caching for external API calls
# Optimize database queries
# Add resource auto-scaling
# Implement cost monitoring
```

### 3. Verify Cost Reduction
```bash
# Monitor cost trends
watch -n 300 'curl -s "http://prometheus:9090/api/v1/query?query=monthly_cost_usd"'

# Check resource usage
kubectl top pods -n spec-to-proof
```

### 4. Post-Incident
1. **Document cost optimization** measures
2. **Schedule cost review** meeting
3. **Implement cost monitoring** improvements

## Prevention

### Monitoring Improvements
- Add cost alerts for specific services
- Monitor cost trends proactively
- Set up budget alerts

### Process Improvements
- Implement cost reviews
- Add cost optimization to CI/CD
- Regular cost audits

### Infrastructure Improvements
- Auto-scaling policies
- Resource optimization
- Cost-effective instance types

## Contact Information
- **Finance**: finance@spec-to-proof.com
- **Slack**: #spec-to-proof-finance
- **Management**: management@spec-to-proof.com

## Related Documentation
- [Cost Optimization Guide](../cost-optimization.md)
- [Budget Planning](../budget-planning.md)
- [Resource Management](../resource-management.md) 