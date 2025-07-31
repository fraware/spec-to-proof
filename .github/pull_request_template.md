# Pull Request

## Description
Brief description of the changes made.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Security fix

## Triple-Check Policy Compliance

### ✅ Automated Review
- [ ] CI passes (lint + tests + proofs)
- [ ] Static analysis passes (ESLint strict, Clippy deny warnings)
- [ ] Security scans pass (cargo audit, Snyk)
- [ ] Coverage > 90%
- [ ] Mutation score > 85%

### ✅ Peer Review
- [ ] Two senior reviewers assigned
- [ ] Code review completed
- [ ] All review comments addressed
- [ ] Tests evidence provided below

### ✅ Manual QA
- [ ] `make smoke-qa` passes in clean Docker
- [ ] Results attached to PR
- [ ] No regressions introduced

## Test Evidence

### Unit Tests
```bash
# Run unit tests
bazel test //... --test_output=all
```

### Integration Tests
```bash
# Run integration tests
bazel test //tests/... --test_output=all
```

### E2E Tests
```bash
# Run E2E tests
bazel test //e2e/...
```

### Coverage Report
```bash
# Generate coverage
bazel coverage //... --combined_report=lcov
```

### Security Scan Results
```bash
# Security audit
cargo audit
cargo deny check
npx snyk test --severity-threshold=high
```

### Lean Proof Verification
```bash
# Verify Lean proofs
bazel build //proof/lean/...
```

## Performance Impact
- [ ] No performance regression
- [ ] Benchmarks updated if applicable
- [ ] Load testing completed if needed

## Security Considerations
- [ ] No new security vulnerabilities introduced
- [ ] Input validation added where needed
- [ ] Authentication/authorization reviewed
- [ ] Secrets management reviewed

## Documentation
- [ ] README updated if needed
- [ ] API documentation updated
- [ ] Code comments added for complex logic
- [ ] Architecture decisions documented

## Breaking Changes
- [ ] Breaking changes documented
- [ ] Migration guide provided if needed
- [ ] Version bump planned

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] All tests pass locally
- [ ] No console errors or warnings
- [ ] Error handling implemented
- [ ] Logging added where appropriate
- [ ] Dependencies updated if needed
- [ ] No sensitive data in logs or comments

## Related Issues
Closes #(issue number)

## Screenshots (if applicable)
Add screenshots for UI changes.

## Additional Notes
Any additional information or context. 