# GitHub App Badge Flow - End-to-End Test Repository

This repository demonstrates the complete badge flow for the Spec-to-Proof GitHub App, including branch protection rules and automated verification.

## Overview

This test repository contains:
- Sample spec documents with various formats
- GitHub Actions workflows for testing
- Branch protection rules requiring badge success
- Example pull requests demonstrating the flow

## Repository Structure

```
e2e/gh-fixtures/
├── README.md                    # This file
├── .github/
│   ├── workflows/
│   │   ├── test-badge-flow.yml # Test workflow
│   │   └── ci.yml              # CI workflow
│   └── branch-protection.yml   # Branch protection rules
├── specs/
│   ├── api-spec.yaml           # API specification
│   ├── security-requirements.md # Security requirements
│   └── data-model.json         # Data model specification
├── tests/
│   ├── badge-flow.test.js      # Badge flow tests
│   └── integration.test.js     # Integration tests
└── docs/
    └── badge-flow.md           # Badge flow documentation
```

## Badge Flow

### 1. Pull Request Creation
When a pull request is created or updated:

1. **Webhook Trigger**: GitHub sends a webhook to the Spec-to-Proof App
2. **Spec Document Detection**: The app scans the PR for spec document references
3. **Badge Status Update**: The app updates the commit status to "pending"

### 2. Spec Document Verification
The app processes spec documents:

1. **Content Extraction**: Extracts spec content from various formats
2. **Proof Generation**: Generates formal proofs using the Lean farm
3. **Sigstore Integration**: Signs proofs with Sigstore (Fulcio + Rekor)
4. **Badge Update**: Updates status based on verification results

### 3. Branch Protection
Branch protection rules ensure:

1. **Required Status Checks**: PR must have successful badge status
2. **No Direct Pushes**: All changes must go through PRs
3. **Up-to-date Branches**: Branches must be up-to-date before merge

## Test Scenarios

### Scenario 1: Valid Spec Documents
- **Input**: PR with valid spec documents
- **Expected**: Badge shows "success" with proof artifacts
- **Result**: PR can be merged

### Scenario 2: Invalid Spec Documents
- **Input**: PR with invalid spec documents
- **Expected**: Badge shows "failure" with error details
- **Result**: PR cannot be merged

### Scenario 3: No Spec Documents
- **Input**: PR with no spec document references
- **Expected**: Badge shows "pending" with "no specs found"
- **Result**: PR cannot be merged (requires specs)

### Scenario 4: Spec Document Updates
- **Input**: PR updates existing spec documents
- **Expected**: Badge re-verifies and updates status
- **Result**: Status reflects new verification results

## Spec Document Formats

The app recognizes spec documents in various formats:

### YAML Specifications
```yaml
# specs/api-spec.yaml
openapi: 3.0.0
info:
  title: API Specification
  version: 1.0.0
paths:
  /users:
    get:
      summary: Get users
      responses:
        '200':
          description: Success
```

### Markdown Requirements
```markdown
# specs/security-requirements.md
# Security Requirements

## Authentication
- All endpoints require OAuth2 authentication
- Tokens expire after 1 hour
- Refresh tokens valid for 30 days

## Authorization
- Role-based access control (RBAC)
- Admin users have full access
- Regular users have read-only access
```

### JSON Data Models
```json
{
  "spec": "data-model",
  "version": "1.0",
  "entities": {
    "User": {
      "properties": {
        "id": {"type": "string", "format": "uuid"},
        "email": {"type": "string", "format": "email"},
        "name": {"type": "string"}
      }
    }
  }
}
```

## Badge Status Types

### Success
- **Color**: Green
- **Message**: "All X spec document(s) verified"
- **Details**: Links to proof artifacts and Sigstore entries

### Failure
- **Color**: Red
- **Message**: "X spec document(s) failed verification"
- **Details**: Error messages and failed proof details

### Pending
- **Color**: Yellow
- **Message**: "Verifying X spec document(s)..."
- **Details**: Progress information

### Error
- **Color**: Red
- **Message**: "Error during verification"
- **Details**: System error information

## Sigstore Integration

### Proof Artifacts
Each verified spec document produces:
- **Content Hash**: SHA256 of the spec content
- **Proof Hash**: SHA256 of the formal proof
- **Rekor Entry**: Immutable log entry
- **Fulcio Certificate**: Identity certificate

### Verification Links
Badge details include links to:
- **Rekor Entry**: `https://rekor.sigstore.dev/api/v1/log/entries/{entry_id}`
- **Fulcio Certificate**: `https://fulcio.sigstore.dev/api/v1/signingCert`
- **Proof Artifact**: Direct link to proof file

## Branch Protection Rules

### Required Status Checks
- `spec-to-proof/verification` must be successful
- Status checks must be up-to-date
- No direct pushes to protected branches

### Required Reviews
- At least 2 approving reviews
- Dismiss stale reviews on new commits
- Require review from code owners

### Restrictions
- No force pushes
- No deletion of protected branches
- Require linear history

## Testing the Flow

### Manual Testing
1. Create a new branch
2. Add spec documents to the branch
3. Create a pull request
4. Observe badge status updates
5. Verify branch protection behavior

### Automated Testing
```bash
# Run badge flow tests
npm test badge-flow

# Run integration tests
npm test integration

# Run all tests
npm test
```

### Test Commands
```bash
# Test with valid specs
curl -X POST http://localhost:8080/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: pull_request" \
  -d @test-data/valid-pr.json

# Test with invalid specs
curl -X POST http://localhost:8080/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: pull_request" \
  -d @test-data/invalid-pr.json
```

## Configuration

### Environment Variables
```bash
# GitHub App Configuration
GH_APP_ID=12345
GH_APP_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----..."
GH_APP_WEBHOOK_SECRET=your-webhook-secret
GH_APP_INSTALLATION_ID=67890

# Server Configuration
GH_APP_HOST=0.0.0.0
GH_APP_PORT=8080
GH_APP_LOG_LEVEL=info
```

### Webhook Configuration
- **URL**: `https://your-app.example.com/webhook`
- **Content Type**: `application/json`
- **Secret**: Your webhook secret
- **Events**: `pull_request`, `push`, `status`

## Monitoring and Debugging

### Logs
```bash
# View application logs
docker logs gh-app

# View webhook logs
docker logs gh-app | grep webhook

# View badge updates
docker logs gh-app | grep badge
```

### Metrics
```bash
# Get metrics
curl http://localhost:8080/metrics

# Health check
curl http://localhost:8080/health
```

### Debug Mode
```bash
# Run with debug logging
GH_APP_LOG_LEVEL=debug ./gh-app
```

## Troubleshooting

### Common Issues

1. **Badge not updating**
   - Check webhook delivery logs
   - Verify webhook secret
   - Check app permissions

2. **Branch protection blocking merges**
   - Ensure badge status is "success"
   - Check required status checks configuration
   - Verify up-to-date branch status

3. **Spec documents not detected**
   - Check file patterns in webhook processor
   - Verify spec document format
   - Check webhook payload structure

### Debug Commands
```bash
# Test webhook signature verification
curl -X POST http://localhost:8080/webhook \
  -H "X-Hub-Signature-256: sha256=..." \
  -d '{"test": "payload"}'

# Test badge status update
curl -X POST http://localhost:8080/badge/test-repo/123 \
  -H "Content-Type: application/json" \
  -d '{"spec_document_ids": ["DOC-123"]}'
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This test repository is licensed under the MIT License. 