# Migration Guide for Spec-to-Proof Platform v1.0.0

This guide provides step-by-step instructions for early design partners migrating to the Spec-to-Proof platform v1.0.0 release.

## Table of Contents

1. [Overview](#overview)
2. [Pre-Migration Checklist](#pre-migration-checklist)
3. [Migration Steps](#migration-steps)
4. [Post-Migration Validation](#post-migration-validation)
5. [Rollback Procedures](#rollback-procedures)
6. [Troubleshooting](#troubleshooting)
7. [Support](#support)

## Overview

The Spec-to-Proof platform v1.0.0 introduces a production-ready architecture with comprehensive security, compliance, and performance features. This migration guide is designed for early design partners who have been testing pre-release versions.

### Key Changes in v1.0.0

- **Production-Ready Architecture**: Microservices with clear boundaries
- **Enhanced Security**: SOC2 compliance, immutable audit logs, encryption
- **Performance Optimization**: Load testing validated, P99 < 90s latency
- **Comprehensive Monitoring**: Prometheus, Grafana, alerting
- **Multiple Deployment Options**: Cloud SaaS, local PoC, air-gapped

### Migration Timeline

- **Estimated Duration**: 2-4 hours
- **Downtime**: Minimal (rolling deployment)
- **Rollback Time**: 15-30 minutes

## Pre-Migration Checklist

### System Requirements

- [ ] Kubernetes cluster (v1.24+) or Docker environment
- [ ] PostgreSQL 15+ with 50GB+ storage
- [ ] Redis 7+ with 10GB+ memory
- [ ] NATS 2.9+ for messaging
- [ ] S3-compatible storage for artifacts
- [ ] Network access to container registry

### Data Backup

- [ ] Backup existing spec documents
- [ ] Export proof artifacts
- [ ] Backup configuration files
- [ ] Document current environment settings

### Environment Preparation

- [ ] Update Helm to v3.12+
- [ ] Install kubectl v1.24+
- [ ] Configure container registry access
- [ ] Set up monitoring namespace
- [ ] Prepare SSL certificates

### Team Preparation

- [ ] Schedule maintenance window
- [ ] Notify stakeholders
- [ ] Prepare rollback team
- [ ] Review migration steps
- [ ] Test in staging environment

## Migration Steps

### Step 1: Environment Setup

#### 1.1 Update Helm Repositories

```bash
# Add required Helm repositories
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add nats https://nats-io.github.io/k8s/helm/charts/
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
```

#### 1.2 Create Namespace

```bash
# Create namespace for spec-to-proof
kubectl create namespace spec-to-proof
kubectl config set-context --current --namespace=spec-to-proof
```

#### 1.3 Configure Secrets

```bash
# Create Kubernetes secrets
kubectl create secret generic spec-to-proof-secrets \
  --from-literal=postgres-password=your-secure-password \
  --from-literal=redis-password=your-secure-password \
  --from-literal=api-token=your-api-token \
  --from-literal=claude-api-key=your-claude-key

# Create registry secret (if using private registry)
kubectl create secret docker-registry regcred \
  --docker-server=your-registry.com \
  --docker-username=your-username \
  --docker-password=your-password
```

### Step 2: Database Migration

#### 2.1 Backup Existing Data

```bash
# Backup PostgreSQL data
pg_dump -h your-db-host -U your-user -d spec_to_proof > backup-$(date +%Y%m%d).sql

# Backup Redis data
redis-cli --rdb backup-$(date +%Y%m%d).rdb
```

#### 2.2 Deploy PostgreSQL

```bash
# Install PostgreSQL with Helm
helm upgrade --install postgresql bitnami/postgresql \
  --namespace spec-to-proof \
  --set auth.postgresPassword=your-secure-password \
  --set auth.database=spec_to_proof \
  --set auth.username=spec_to_proof \
  --set auth.password=your-secure-password \
  --set primary.persistence.enabled=true \
  --set primary.persistence.size=50Gi \
  --wait --timeout=10m
```

#### 2.3 Run Database Migrations

```bash
# Apply database schema
kubectl apply -f charts/spec-to-proof/templates/supporting.yaml

# Verify database connection
kubectl run db-test --rm -i --restart=Never --image=postgres:15 \
  --env="PGPASSWORD=your-secure-password" \
  -- psql -h postgresql -U spec_to_proof -d spec_to_proof -c "SELECT version();"
```

### Step 3: Deploy Core Services

#### 3.1 Deploy Dependencies

```bash
# Deploy Redis
helm upgrade --install redis bitnami/redis \
  --namespace spec-to-proof \
  --set auth.enabled=true \
  --set auth.password=your-secure-password \
  --set master.persistence.enabled=true \
  --set master.persistence.size=10Gi \
  --wait --timeout=10m

# Deploy NATS
helm upgrade --install nats nats/nats \
  --namespace spec-to-proof \
  --set nats.jetstream.enabled=true \
  --set nats.jetstream.memStorage.enabled=true \
  --set nats.jetstream.memStorage.size=2Gi \
  --set nats.jetstream.fileStorage.enabled=true \
  --set nats.jetstream.fileStorage.size=20Gi \
  --wait --timeout=10m
```

#### 3.2 Deploy Spec-to-Proof Platform

```bash
# Create values file for your environment
cat > values-production.yaml << EOF
global:
  environment: production
  domain: your-domain.com
  imageRegistry: your-registry.com
  imagePullSecrets:
    - name: regcred

images:
  leanFarm:
    repository: your-registry.com/lean-farm
    tag: "1.0.0"
  nlp:
    repository: your-registry.com/nlp
    tag: "1.0.0"
  ingest:
    repository: your-registry.com/ingest
    tag: "1.0.0"
  proof:
    repository: your-registry.com/proof
    tag: "1.0.0"
  platform:
    repository: your-registry.com/platform
    tag: "1.0.0"
  ui:
    repository: your-registry.com/ui
    tag: "1.0.0"

# Production settings
leanFarm:
  replicaCount: 3
  resources:
    requests:
      cpu: 1000m
      memory: 2Gi
    limits:
      cpu: 4000m
      memory: 8Gi

monitoring:
  enabled: true

security:
  podSecurityStandards:
    enabled: true
  networkPolicies:
    enabled: true
  rbac:
    enabled: true
    create: true

backup:
  enabled: true
  schedule: "0 2 * * *"
  retention: 30
EOF

# Deploy the platform
helm upgrade --install spec-to-proof ./charts/spec-to-proof \
  --namespace spec-to-proof \
  --values values-production.yaml \
  --wait --timeout=15m
```

### Step 4: Configure Monitoring

#### 4.1 Deploy Prometheus

```bash
# Deploy Prometheus
helm upgrade --install prometheus prometheus-community/prometheus \
  --namespace spec-to-proof \
  --set server.persistentVolume.enabled=true \
  --set server.persistentVolume.size=10Gi \
  --wait --timeout=10m
```

#### 4.2 Deploy Grafana

```bash
# Deploy Grafana
helm upgrade --install grafana grafana/grafana \
  --namespace spec-to-proof \
  --set adminPassword=your-secure-password \
  --set persistence.enabled=true \
  --set persistence.size=5Gi \
  --wait --timeout=10m
```

### Step 5: Data Migration

#### 5.1 Import Existing Data

```bash
# Import spec documents
kubectl run data-import --rm -i --restart=Never \
  --image=your-registry.com/ingest:1.0.0 \
  -- env POSTGRES_HOST=postgresql \
     POSTGRES_PASSWORD=your-secure-password \
     python import_data.py

# Verify data import
kubectl exec -it deployment/spec-to-proof-ingest -- \
  curl -f http://localhost:8080/health
```

#### 5.2 Verify Data Integrity

```bash
# Check spec count
kubectl exec -it deployment/spec-to-proof-ingest -- \
  curl -f http://localhost:8080/api/v1/specs/count

# Check proof artifacts
kubectl exec -it deployment/spec-to-proof-proof -- \
  curl -f http://localhost:8080/api/v1/proofs/count
```

## Post-Migration Validation

### Health Checks

```bash
# Check all services are healthy
kubectl get pods -n spec-to-proof
kubectl get svc -n spec-to-proof

# Verify endpoints
curl -f https://your-domain.com/health
curl -f https://your-domain.com/api/v1/specs
```

### Performance Validation

```bash
# Run quick performance test
./scripts/run-benchmarks.sh --load-test --duration 5m --target 100

# Check performance metrics
kubectl port-forward svc/grafana 3000:3000 -n spec-to-proof
# Open http://localhost:3000 (admin/your-secure-password)
```

### Security Validation

```bash
# Verify security policies
kubectl get networkpolicies -n spec-to-proof
kubectl get podsecuritypolicies -n spec-to-proof

# Check audit logs
kubectl logs -l app=spec-to-proof-platform -n spec-to-proof
```

### Compliance Validation

```bash
# Run compliance checks
./scripts/ci-lint.sh
./scripts/security-benchmark.sh

# Verify SOC2 readiness
cat docs/compliance/soc2-readiness-checklist.md
```

## Rollback Procedures

### Quick Rollback (15 minutes)

```bash
# Rollback to previous version
helm rollback spec-to-proof 1 -n spec-to-proof

# Verify rollback
kubectl get pods -n spec-to-proof
kubectl get svc -n spec-to-proof
```

### Full Rollback (30 minutes)

```bash
# Uninstall current version
helm uninstall spec-to-proof -n spec-to-proof

# Restore from backup
kubectl apply -f backup/spec-to-proof-backup.yaml

# Restore database
kubectl run db-restore --rm -i --restart=Never --image=postgres:15 \
  --env="PGPASSWORD=your-secure-password" \
  -- psql -h postgresql -U spec_to_proof -d spec_to_proof < backup-$(date +%Y%m%d).sql
```

## Troubleshooting

### Common Issues

#### 1. Pod Startup Failures

```bash
# Check pod status
kubectl describe pod <pod-name> -n spec-to-proof

# Check logs
kubectl logs <pod-name> -n spec-to-proof

# Check events
kubectl get events -n spec-to-proof --sort-by='.lastTimestamp'
```

#### 2. Database Connection Issues

```bash
# Test database connectivity
kubectl run db-test --rm -i --restart=Never --image=postgres:15 \
  --env="PGPASSWORD=your-secure-password" \
  -- psql -h postgresql -U spec_to_proof -d spec_to_proof -c "SELECT 1;"
```

#### 3. Performance Issues

```bash
# Check resource usage
kubectl top pods -n spec-to-proof

# Check metrics
kubectl port-forward svc/prometheus 9090:9090 -n spec-to-proof
# Open http://localhost:9090
```

#### 4. Security Issues

```bash
# Check security policies
kubectl get networkpolicies -n spec-to-proof -o yaml

# Verify secrets
kubectl get secrets -n spec-to-proof
```

### Debug Commands

```bash
# Get detailed service information
kubectl get all -n spec-to-proof

# Check service endpoints
kubectl get endpoints -n spec-to-proof

# Verify ingress configuration
kubectl get ingress -n spec-to-proof

# Check persistent volumes
kubectl get pvc -n spec-to-proof
```

## Support

### Getting Help

- **Documentation**: [docs/](./docs/)
- **Issues**: [GitHub Issues](https://github.com/your-org/spec-to-proof/issues)
- **Email**: platform@your-org.com
- **Slack**: #spec-to-proof-platform

### Escalation Path

1. **Level 1**: Platform team (response: 4 hours)
2. **Level 2**: Senior engineers (response: 2 hours)
3. **Level 3**: Architecture team (response: 1 hour)

### Emergency Contacts

- **Platform Lead**: platform-lead@your-org.com
- **DevOps Lead**: devops-lead@your-org.com
- **Security Lead**: security-lead@your-org.com

### Post-Migration Support

- **Week 1**: Daily check-ins
- **Week 2-4**: Weekly reviews
- **Month 2+**: Monthly health checks

## Success Criteria

### Technical Success

- [ ] All services healthy and responding
- [ ] Performance targets met (P99 < 90s)
- [ ] Security policies enforced
- [ ] Monitoring and alerting functional
- [ ] Backup and recovery tested

### Business Success

- [ ] Zero data loss during migration
- [ ] Minimal downtime (< 30 minutes)
- [ ] All users can access platform
- [ ] Performance improved or maintained
- [ ] Compliance requirements met

### Operational Success

- [ ] Team trained on new platform
- [ ] Documentation updated
- [ ] Support processes established
- [ ] Monitoring dashboards configured
- [ ] Incident response procedures tested

---

*This migration guide is maintained by the Platform Team. For questions or updates, please contact platform@your-org.com.* 