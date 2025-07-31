# Test Coverage Drop Runbook

## Alert Description
Test coverage has dropped below 90% threshold.

## Severity
**Info** - Business hours only

## Alert Details
- **Metric**: `test_coverage_percentage < 90`
- **Duration**: 1 hour
- **Team**: Platform

## Immediate Actions

### 1. Verify the Alert
```bash
# Check current test coverage
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query?query=test_coverage_percentage

# Check recent coverage trends
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query_range?query=test_coverage_percentage&start=$(date -d '1 hour ago' +%s)&end=$(date +%s)&step=5m
```

### 2. Identify Affected Components
- Check which modules have reduced coverage
- Review recent code changes that might have introduced uncovered code
- Examine test execution logs for failures

### 3. Quick Assessment
- [ ] Coverage drop is temporary (test failures)
- [ ] Coverage drop is permanent (new uncovered code)
- [ ] Coverage drop is due to code removal without test updates

## Investigation Steps

### Step 1: Check Test Execution
```bash
# Run tests locally to verify current state
make test
make test-coverage

# Check for test failures
cargo test --no-run
```

### Step 2: Analyze Coverage Report
```bash
# Generate detailed coverage report
cargo tarpaulin --out Html --output-dir coverage-report

# Open coverage report
open coverage-report/tarpaulin-report.html
```

### Step 3: Identify Coverage Gaps
- Look for new code paths without tests
- Check for conditional branches not covered
- Review error handling paths

## Resolution Steps

### For Temporary Coverage Drops (Test Failures)
1. **Fix failing tests**
   ```bash
   # Run specific failing tests
   cargo test --test <test_name>
   
   # Update tests if needed
   # Re-run coverage
   cargo tarpaulin
   ```

2. **Update test expectations**
   - Review test assertions
   - Update mock data if needed
   - Fix test setup/teardown

### For Permanent Coverage Drops (New Code)
1. **Add missing tests**
   ```bash
   # Identify uncovered functions
   cargo tarpaulin --skip-clean --out Html
   
   # Add unit tests for uncovered code
   # Focus on critical paths first
   ```

2. **Update integration tests**
   - Add end-to-end test scenarios
   - Test error conditions
   - Test edge cases

3. **Add property-based tests**
   ```rust
   use proptest::prelude::*;
   
   proptest! {
       #[test]
       fn test_property(/* parameters */) {
           // Property-based test implementation
       }
   }
   ```

### For Code Removal
1. **Update test suite**
   - Remove tests for deleted code
   - Update test descriptions
   - Ensure remaining tests still pass

2. **Clean up test utilities**
   - Remove unused test helpers
   - Update test fixtures

## Verification

### Step 1: Re-run Coverage
```bash
# Run full test suite with coverage
cargo tarpaulin --out Html

# Verify coverage is above 90%
# Check coverage report for any remaining gaps
```

### Step 2: Update CI Pipeline
```bash
# Ensure coverage check is enforced
# Update coverage thresholds if needed
# Verify coverage reporting in CI
```

### Step 3: Document Changes
- Update test documentation
- Add comments for complex test scenarios
- Document any new test patterns

## Prevention

### Code Review Process
- [ ] Require test coverage for new code
- [ ] Review test quality, not just coverage percentage
- [ ] Ensure error paths are tested

### Monitoring
- [ ] Set up coverage trend alerts
- [ ] Monitor coverage by module
- [ ] Track test execution time

### Best Practices
- [ ] Write tests before implementing features (TDD)
- [ ] Use property-based testing for complex logic
- [ ] Test error conditions and edge cases
- [ ] Keep tests focused and maintainable

## Escalation

### When to Escalate
- Coverage drops below 80%
- Multiple modules affected
- Coverage drop persists for > 24 hours
- Critical functionality uncovered

### Escalation Path
1. **Platform Team Lead**
   - Review coverage trends
   - Assess impact on code quality
   - Plan coverage improvement strategy

2. **Engineering Manager**
   - Evaluate resource allocation
   - Review testing strategy
   - Consider process improvements

## Related Documentation
- [Testing Strategy](../testing-strategy.md)
- [Code Review Guidelines](../code-review.md)
- [CI/CD Pipeline](../ci-cd.md)

## Runbook Owner
**Platform Team** - Responsible for maintaining test coverage standards and quality assurance processes. 