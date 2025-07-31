# SRE Runbooks

This directory contains runbooks for all alerts and operational procedures for the Spec-to-Proof platform.

## Alert Categories

### Critical Alerts (SEV-1)
- [Proof Failure Rate](./proof-failure-rate.md) - Proof failure rate > 5% for 5 minutes
- [Drift Backlog Overflow](./drift-backlog-overflow.md) - Drift backlog > 100 items
- [Service Unavailable](./service-unavailable.md) - Public API unavailable

### Warning Alerts (SEV-2)
- [High Latency](./high-latency.md) - P99 latency > 90 seconds
- [High Error Rate](./high-error-rate.md) - Error rate > 2% for 5 minutes
- [Resource Exhaustion](./resource-exhaustion.md) - CPU/Memory usage > 80%

### Info Alerts (SEV-3)
- [Cost Threshold](./cost-threshold.md) - Monthly cost approaching budget
- [Coverage Drop](./coverage-drop.md) - Test coverage < 90%
- [Security Scan Failures](./security-scan-failures.md) - Security scans failing

## Business Hours Policy

- **SEV-1**: Page immediately, 24/7
- **SEV-2**: Page during business hours (9 AM - 6 PM EST), email outside
- **SEV-3**: Email only, no paging

## Runbook Template

Each runbook follows this structure:

1. **Alert Description** - What triggered the alert
2. **Impact Assessment** - What this means for users
3. **Immediate Actions** - Steps to take within 5 minutes
4. **Investigation** - How to diagnose the root cause
5. **Resolution** - How to fix the issue
6. **Prevention** - How to prevent recurrence
7. **Post-Mortem** - What to document after resolution

## Quick Reference

### Emergency Contacts
- **On-Call Engineer**: Check PagerDuty
- **Platform Lead**: @platform-lead
- **Security Team**: security@company.com

### Common Commands
```bash
# Check service status
kubectl get pods -n spec-to-proof

# View logs
kubectl logs -f deployment/spec-to-proof-api

# Check metrics
curl http://prometheus:9090/api/v1/query?query=up

# Restart service
kubectl rollout restart deployment/spec-to-proof-api
```

### Escalation Path
1. On-call engineer (15 minutes)
2. Platform lead (30 minutes)
3. Engineering manager (1 hour)
4. CTO (2 hours) 