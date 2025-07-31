# Security Scan Failures Runbook

## Alert Description
Security scans are failing, indicating potential security vulnerabilities or compliance issues.

## Severity
**Info** - Business hours only

## Alert Details
- **Metric**: `security_scan_failures_total > 0`
- **Duration**: 1 hour
- **Team**: Security

## Immediate Actions

### 1. Verify the Alert
```bash
# Check current security scan status
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query?query=security_scan_failures_total

# Check recent scan results
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query_range?query=security_scan_failures_total&start=$(date -d '1 hour ago' +%s)&end=$(date +%s)&step=5m
```

### 2. Identify Scan Types
- [ ] SAST (Static Application Security Testing)
- [ ] SCA (Software Composition Analysis)
- [ ] Container image scanning
- [ ] Infrastructure as Code scanning
- [ ] Dependency vulnerability scanning

### 3. Quick Assessment
- [ ] False positive (scan configuration issue)
- [ ] Known vulnerability (documented)
- [ ] New vulnerability (requires immediate attention)
- [ ] Compliance violation (licensing, policy)

## Investigation Steps

### Step 1: Check Scan Logs
```bash
# Check CI/CD logs for scan failures
# Look for specific error messages
# Identify which components are affected

# Check Snyk logs
snyk test --json > snyk-results.json

# Check Trivy logs
trivy fs --format json . > trivy-results.json

# Check Bandit logs (Python)
bandit -r . -f json > bandit-results.json
```

### Step 2: Categorize Issues
```bash
# Parse scan results by severity
jq '.vulnerabilities[] | select(.severity == "critical")' snyk-results.json
jq '.vulnerabilities[] | select(.severity == "high")' snyk-results.json

# Check for known vulnerabilities
# Verify if patches are available
# Assess exploitability
```

### Step 3: Assess Impact
- **Critical**: Immediate fix required
- **High**: Fix within 24 hours
- **Medium**: Fix within 1 week
- **Low**: Fix within 1 month

## Resolution Steps

### For False Positives
1. **Update scan configuration**
   ```yaml
   # .snyk file
   version: v1.25.0
   ignore:
     'npm:package-name@version':
       - id: vulnerability-id
         reason: 'False positive - not exploitable in our context'
         expires: 2024-12-31T00:00:00.000Z
   ```

2. **Add scan exclusions**
   ```yaml
   # .trivyignore
   # Ignore specific paths or patterns
   vendor/
   node_modules/
   *.test.js
   ```

3. **Update scan rules**
   ```yaml
   # bandit config
   exclude: ['tests/', 'vendor/']
   skips: ['B101', 'B601']
   ```

### For Known Vulnerabilities
1. **Check for updates**
   ```bash
   # Update dependencies
   cargo update
   npm update
   pip install --upgrade package-name
   ```

2. **Apply security patches**
   ```bash
   # Apply specific patches
   cargo add package-name@patched-version
   npm install package-name@latest
   ```

3. **Implement workarounds**
   ```rust
   // Example: Disable vulnerable feature
   #[cfg(not(feature = "vulnerable-feature"))]
   pub fn secure_implementation() {
       // Secure implementation
   }
   ```

### For New Vulnerabilities
1. **Immediate containment**
   - Disable affected features if possible
   - Add input validation
   - Implement rate limiting

2. **Develop fix**
   ```rust
   // Example: Input validation
   pub fn secure_function(input: &str) -> Result<(), Error> {
       if input.len() > MAX_LENGTH {
           return Err(Error::InputTooLong);
       }
       // Process input safely
       Ok(())
   }
   ```

3. **Test the fix**
   ```bash
   # Run security tests
   cargo test security
   
   # Re-run security scans
   snyk test
   trivy fs .
   ```

## Verification

### Step 1: Re-run Security Scans
```bash
# Run all security scans
make security-scan

# Verify no critical/high vulnerabilities
snyk test --severity-threshold=medium
trivy fs --severity HIGH,CRITICAL .
```

### Step 2: Update Documentation
- Document vulnerability details
- Record remediation steps
- Update security policy if needed

### Step 3: Monitor for Recurrence
```bash
# Set up monitoring for specific vulnerabilities
# Track vulnerability trends
# Monitor dependency updates
```

## Prevention

### Dependency Management
- [ ] Use dependency vulnerability scanning
- [ ] Keep dependencies updated
- [ ] Use lock files for reproducible builds
- [ ] Regular dependency audits

### Code Security
- [ ] Follow secure coding practices
- [ ] Use security linters
- [ ] Implement input validation
- [ ] Use secure defaults

### Infrastructure Security
- [ ] Scan container images
- [ ] Use minimal base images
- [ ] Implement network policies
- [ ] Regular security updates

### CI/CD Security
- [ ] Integrate security scans in CI
- [ ] Block merges on critical vulnerabilities
- [ ] Regular security training
- [ ] Security code reviews

## Escalation

### When to Escalation
- Critical vulnerabilities detected
- Multiple high-severity issues
- Compliance violations
- Exploitable vulnerabilities in production

### Escalation Path
1. **Security Team Lead**
   - Assess vulnerability impact
   - Coordinate remediation
   - Update security policies

2. **CISO/CTO**
   - Review security posture
   - Approve emergency fixes
   - Coordinate with legal/compliance

## Related Documentation
- [Security Policy](../security-policy.md)
- [Vulnerability Management](../vulnerability-management.md)
- [Compliance Guidelines](../compliance.md)

## Runbook Owner
**Security Team** - Responsible for maintaining security standards and vulnerability management processes. 