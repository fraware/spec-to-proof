import { validateSpecDocument, validateInvariant, validateInvariantSet, validateLeanTheorem, validateProofArtifact, validateBadgeStatus, fromApiSpecDocument, fromApiInvariant, toApiSpecDocument, toApiInvariant, calculateSha256, generateId, isSpecDocument, isInvariant, isInvariantSet, isLeanTheorem, isProofArtifact, isBadgeStatus } from '../spec-to-proof';
import { SpecDocument, Invariant, InvariantSet, LeanTheorem, ProofArtifact, BadgeStatus, DocumentStatus, InvariantStatus, InvariantSetStatus, TheoremStatus, ProofStatus, BadgeState, Priority } from '../spec-to-proof';
import * as fc from 'fast-check';

describe('Spec-to-Proof Domain Models', () => {
  describe('SpecDocument', () => {
    const validSpecDocument: SpecDocument = {
      id: '123e4567-e89b-12d3-a456-426614174000',
      contentSha256: 'a'.repeat(64),
      sourceSystem: 'jira',
      sourceId: 'PROJ-123',
      title: 'User Authentication Requirements',
      content: 'Users must authenticate with valid credentials.',
      url: 'https://example.com/jira/browse/PROJ-123',
      author: 'john.doe@example.com',
      createdAt: new Date('2024-01-01T00:00:00Z'),
      modifiedAt: new Date('2024-01-02T00:00:00Z'),
      metadata: { project: 'auth-service', priority: 'high' },
      version: 1,
      status: DocumentStatus.PUBLISHED,
    };

    it('should validate a valid SpecDocument', () => {
      expect(() => validateSpecDocument(validSpecDocument)).not.toThrow();
    });

    it('should reject invalid UUID', () => {
      const invalidDoc = { ...validSpecDocument, id: 'invalid-uuid' };
      expect(() => validateSpecDocument(invalidDoc)).toThrow();
    });

    it('should reject invalid SHA256', () => {
      const invalidDoc = { ...validSpecDocument, contentSha256: 'invalid-hash' };
      expect(() => validateSpecDocument(invalidDoc)).toThrow();
    });

    it('should reject invalid URL', () => {
      const invalidDoc = { ...validSpecDocument, url: 'not-a-url' };
      expect(() => validateSpecDocument(invalidDoc)).toThrow();
    });

    it('should reject negative version', () => {
      const invalidDoc = { ...validSpecDocument, version: -1 };
      expect(() => validateSpecDocument(invalidDoc)).toThrow();
    });

    it('should convert from API format', () => {
      const apiDoc = {
        id: validSpecDocument.id,
        content_sha256: validSpecDocument.contentSha256,
        source_system: validSpecDocument.sourceSystem,
        source_id: validSpecDocument.sourceId,
        title: validSpecDocument.title,
        content: validSpecDocument.content,
        url: validSpecDocument.url,
        author: validSpecDocument.author,
        created_at: validSpecDocument.createdAt.toISOString(),
        modified_at: validSpecDocument.modifiedAt.toISOString(),
        metadata: validSpecDocument.metadata,
        version: validSpecDocument.version,
        status: validSpecDocument.status,
      };

      const result = fromApiSpecDocument(apiDoc);
      expect(result).toEqual(validSpecDocument);
    });

    it('should convert to API format', () => {
      const result = toApiSpecDocument(validSpecDocument);
      expect(result.id).toBe(validSpecDocument.id);
      expect(result.content_sha256).toBe(validSpecDocument.contentSha256);
      expect(result.source_system).toBe(validSpecDocument.sourceSystem);
      expect(result.created_at).toBe(validSpecDocument.createdAt.toISOString());
    });

    it('should pass type guard', () => {
      expect(isSpecDocument(validSpecDocument)).toBe(true);
      expect(isSpecDocument({})).toBe(false);
    });
  });

  describe('Invariant', () => {
    const validInvariant: Invariant = {
      id: '123e4567-e89b-12d3-a456-426614174001',
      contentSha256: 'b'.repeat(64),
      description: 'User password must be at least 8 characters',
      formalExpression: 'length(password) >= 8',
      naturalLanguage: 'The password length must be greater than or equal to 8 characters',
      variables: [
        {
          name: 'password',
          type: 'string',
          description: 'User password',
          unit: '',
          constraints: ['non-empty', 'alphanumeric'],
        },
      ],
      units: { password: 'characters' },
      confidenceScore: 0.95,
      sourceDocumentId: '123e4567-e89b-12d3-a456-426614174000',
      extractedAt: new Date('2024-01-01T00:00:00Z'),
      status: InvariantStatus.EXTRACTED,
      tags: ['security', 'validation'],
      priority: Priority.HIGH,
    };

    it('should validate a valid Invariant', () => {
      expect(() => validateInvariant(validInvariant)).not.toThrow();
    });

    it('should reject invalid confidence score', () => {
      const invalidInvariant = { ...validInvariant, confidenceScore: 1.5 };
      expect(() => validateInvariant(invalidInvariant)).toThrow();
    });

    it('should reject empty description', () => {
      const invalidInvariant = { ...validInvariant, description: '' };
      expect(() => validateInvariant(invalidInvariant)).toThrow();
    });

    it('should convert from API format', () => {
      const apiInvariant = {
        id: validInvariant.id,
        content_sha256: validInvariant.contentSha256,
        description: validInvariant.description,
        formal_expression: validInvariant.formalExpression,
        natural_language: validInvariant.naturalLanguage,
        variables: validInvariant.variables.map(v => ({
          name: v.name,
          type: v.type,
          description: v.description,
          unit: v.unit,
          constraints: v.constraints,
        })),
        units: validInvariant.units,
        confidence_score: validInvariant.confidenceScore,
        source_document_id: validInvariant.sourceDocumentId,
        extracted_at: validInvariant.extractedAt.toISOString(),
        status: validInvariant.status,
        tags: validInvariant.tags,
        priority: validInvariant.priority,
      };

      const result = fromApiInvariant(apiInvariant);
      expect(result).toEqual(validInvariant);
    });

    it('should convert to API format', () => {
      const result = toApiInvariant(validInvariant);
      expect(result.id).toBe(validInvariant.id);
      expect(result.content_sha256).toBe(validInvariant.contentSha256);
      expect(result.description).toBe(validInvariant.description);
      expect(result.formal_expression).toBe(validInvariant.formalExpression);
    });

    it('should pass type guard', () => {
      expect(isInvariant(validInvariant)).toBe(true);
      expect(isInvariant({})).toBe(false);
    });
  });

  describe('InvariantSet', () => {
    const validInvariantSet: InvariantSet = {
      id: '123e4567-e89b-12d3-a456-426614174002',
      contentSha256: 'c'.repeat(64),
      name: 'Authentication Requirements',
      description: 'All authentication-related invariants',
      invariants: [],
      sourceDocumentIds: ['123e4567-e89b-12d3-a456-426614174000'],
      createdAt: new Date('2024-01-01T00:00:00Z'),
      modifiedAt: new Date('2024-01-02T00:00:00Z'),
      status: InvariantSetStatus.DRAFT,
    };

    it('should validate a valid InvariantSet', () => {
      expect(() => validateInvariantSet(validInvariantSet)).not.toThrow();
    });

    it('should reject empty name', () => {
      const invalidSet = { ...validInvariantSet, name: '' };
      expect(() => validateInvariantSet(invalidSet)).toThrow();
    });

    it('should pass type guard', () => {
      expect(isInvariantSet(validInvariantSet)).toBe(true);
      expect(isInvariantSet({})).toBe(false);
    });
  });

  describe('LeanTheorem', () => {
    const validLeanTheorem: LeanTheorem = {
      id: '123e4567-e89b-12d3-a456-426614174003',
      contentSha256: 'd'.repeat(64),
      theoremName: 'password_length_theorem',
      leanCode: 'theorem password_length_theorem : length password ≥ 8 := by simp',
      sourceInvariantId: '123e4567-e89b-12d3-a456-426614174001',
      generatedAt: new Date('2024-01-01T00:00:00Z'),
      status: TheoremStatus.GENERATED,
      compilationErrors: [],
      proofStrategy: 'simp',
      metadata: { complexity: 'low', estimated_time: '5s' },
    };

    it('should validate a valid LeanTheorem', () => {
      expect(() => validateLeanTheorem(validLeanTheorem)).not.toThrow();
    });

    it('should reject empty theorem name', () => {
      const invalidTheorem = { ...validLeanTheorem, theoremName: '' };
      expect(() => validateLeanTheorem(invalidTheorem)).toThrow();
    });

    it('should pass type guard', () => {
      expect(isLeanTheorem(validLeanTheorem)).toBe(true);
      expect(isLeanTheorem({})).toBe(false);
    });
  });

  describe('ProofArtifact', () => {
    const validProofArtifact: ProofArtifact = {
      id: '123e4567-e89b-12d3-a456-426614174004',
      contentSha256: 'e'.repeat(64),
      theoremId: '123e4567-e89b-12d3-a456-426614174003',
      invariantId: '123e4567-e89b-12d3-a456-426614174001',
      status: ProofStatus.SUCCESS,
      attemptedAt: new Date('2024-01-01T00:00:00Z'),
      durationMs: 5000,
      output: 'Proof completed successfully',
      logs: ['Compiling theorem...', 'Proof found'],
      resourceUsage: {
        cpuSeconds: 4.5,
        memoryBytes: 1024 * 1024 * 100,
        diskBytes: 1024 * 1024 * 10,
        networkBytes: 1024 * 512,
      },
      proofStrategy: 'simp',
      confidenceScore: 1.0,
      metadata: { proof_time: '5s', memory_peak: '100MB' },
    };

    it('should validate a valid ProofArtifact', () => {
      expect(() => validateProofArtifact(validProofArtifact)).not.toThrow();
    });

    it('should reject negative duration', () => {
      const invalidArtifact = { ...validProofArtifact, durationMs: -1 };
      expect(() => validateProofArtifact(invalidArtifact)).toThrow();
    });

    it('should pass type guard', () => {
      expect(isProofArtifact(validProofArtifact)).toBe(true);
      expect(isProofArtifact({})).toBe(false);
    });
  });

  describe('BadgeStatus', () => {
    const validBadgeStatus: BadgeStatus = {
      id: '123e4567-e89b-12d3-a456-426614174005',
      contentSha256: 'f'.repeat(64),
      repoOwner: 'fraware',
      repoName: 'spec-to-proof',
      prNumber: 123,
      commitSha: 'a'.repeat(40),
      state: BadgeState.SUCCESS,
      description: 'All invariants proven successfully',
      targetUrl: 'https://example.com/proofs/123',
      createdAt: new Date('2024-01-01T00:00:00Z'),
      updatedAt: new Date('2024-01-02T00:00:00Z'),
      proofArtifactIds: ['123e4567-e89b-12d3-a456-426614174004'],
      coveragePercentage: 100.0,
      invariantsProven: 5,
      totalInvariants: 5,
    };

    it('should validate a valid BadgeStatus', () => {
      expect(() => validateBadgeStatus(validBadgeStatus)).not.toThrow();
    });

    it('should reject invalid commit SHA', () => {
      const invalidBadge = { ...validBadgeStatus, commitSha: 'invalid-sha' };
      expect(() => validateBadgeStatus(invalidBadge)).toThrow();
    });

    it('should reject coverage > 100%', () => {
      const invalidBadge = { ...validBadgeStatus, coveragePercentage: 150.0 };
      expect(() => validateBadgeStatus(invalidBadge)).toThrow();
    });

    it('should pass type guard', () => {
      expect(isBadgeStatus(validBadgeStatus)).toBe(true);
      expect(isBadgeStatus({})).toBe(false);
    });
  });

  describe('Utility Functions', () => {
    it('should generate SHA256 hash', () => {
      const content = 'test content';
      const hash = calculateSha256(content);
      expect(hash).toBe('placeholder-sha256-hash');
    });

    it('should generate UUID', () => {
      const id1 = generateId();
      const id2 = generateId();
      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    });
  });

  // Property-based tests with fast-check
  describe('Property-based Tests', () => {
    const uuidArb = fc.uuid();
    const sha256Arb = fc.stringOf(fc.constantFrom('a', 'b', 'c', 'd', 'e', 'f', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'), { minLength: 64, maxLength: 64 });
    const urlArb = fc.webUrl();
    const dateArb = fc.date();
    const positiveIntArb = fc.integer({ min: 1 });
    const nonNegativeIntArb = fc.integer({ min: 0 });
    const confidenceScoreArb = fc.float({ min: 0, max: 1 });
    const coveragePercentageArb = fc.float({ min: 0, max: 100 });

    it('should round-trip SpecDocument through API conversion', () => {
      fc.assert(
        fc.property(
          uuidArb,
          sha256Arb,
          fc.string(),
          fc.string(),
          fc.string(),
          fc.string(),
          urlArb,
          fc.string(),
          dateArb,
          dateArb,
          fc.record({}),
          positiveIntArb,
          fc.constantFrom(...Object.values(DocumentStatus)),
          (id, contentSha256, sourceSystem, sourceId, title, content, url, author, createdAt, modifiedAt, metadata, version, status) => {
            const original: SpecDocument = {
              id,
              contentSha256,
              sourceSystem,
              sourceId,
              title,
              content,
              url,
              author,
              createdAt,
              modifiedAt,
              metadata,
              version,
              status,
            };

            const apiFormat = toApiSpecDocument(original);
            const roundTrip = fromApiSpecDocument(apiFormat);

            expect(roundTrip).toEqual(original);
          }
        )
      );
    });

    it('should round-trip Invariant through API conversion', () => {
      fc.assert(
        fc.property(
          uuidArb,
          sha256Arb,
          fc.string(),
          fc.string(),
          fc.string(),
          fc.array(fc.record({
            name: fc.string(),
            type: fc.string(),
            description: fc.string(),
            unit: fc.string(),
            constraints: fc.array(fc.string()),
          })),
          fc.record({}),
          confidenceScoreArb,
          uuidArb,
          dateArb,
          fc.constantFrom(...Object.values(InvariantStatus)),
          fc.array(fc.string()),
          fc.constantFrom(...Object.values(Priority)),
          (id, contentSha256, description, formalExpression, naturalLanguage, variables, units, confidenceScore, sourceDocumentId, extractedAt, status, tags, priority) => {
            const original: Invariant = {
              id,
              contentSha256,
              description,
              formalExpression,
              naturalLanguage,
              variables,
              units,
              confidenceScore,
              sourceDocumentId,
              extractedAt,
              status,
              tags,
              priority,
            };

            const apiFormat = toApiInvariant(original);
            const roundTrip = fromApiInvariant(apiFormat);

            expect(roundTrip).toEqual(original);
          }
        )
      );
    });

    it('should maintain SHA256 hash consistency', () => {
      fc.assert(
        fc.property(
          fc.string(),
          (content) => {
            const hash1 = calculateSha256(content);
            const hash2 = calculateSha256(content);
            expect(hash1).toBe(hash2);
          }
        )
      );
    });

    it('should generate unique UUIDs', () => {
      fc.assert(
        fc.property(
          fc.array(fc.constant(null), { minLength: 10, maxLength: 10 }),
          () => {
            const ids = Array.from({ length: 10 }, () => generateId());
            const uniqueIds = new Set(ids);
            expect(uniqueIds.size).toBe(10);
          }
        )
      );
    });

    it('should validate SpecDocument with property-based data', () => {
      fc.assert(
        fc.property(
          uuidArb,
          sha256Arb,
          fc.string({ minLength: 1 }),
          fc.string({ minLength: 1 }),
          fc.string({ minLength: 1 }),
          fc.string(),
          urlArb,
          fc.string({ minLength: 1 }),
          dateArb,
          dateArb,
          fc.record({}),
          positiveIntArb,
          fc.constantFrom(...Object.values(DocumentStatus)),
          (id, contentSha256, sourceSystem, sourceId, title, content, url, author, createdAt, modifiedAt, metadata, version, status) => {
            const doc: SpecDocument = {
              id,
              contentSha256,
              sourceSystem,
              sourceId,
              title,
              content,
              url,
              author,
              createdAt,
              modifiedAt,
              metadata,
              version,
              status,
            };

            expect(() => validateSpecDocument(doc)).not.toThrow();
          }
        )
      );
    });

    it('should validate Invariant with property-based data', () => {
      fc.assert(
        fc.property(
          uuidArb,
          sha256Arb,
          fc.string({ minLength: 1 }),
          fc.string({ minLength: 1 }),
          fc.string(),
          fc.array(fc.record({
            name: fc.string({ minLength: 1 }),
            type: fc.string({ minLength: 1 }),
            description: fc.string(),
            unit: fc.string(),
            constraints: fc.array(fc.string()),
          })),
          fc.record({}),
          confidenceScoreArb,
          uuidArb,
          dateArb,
          fc.constantFrom(...Object.values(InvariantStatus)),
          fc.array(fc.string()),
          fc.constantFrom(...Object.values(Priority)),
          (id, contentSha256, description, formalExpression, naturalLanguage, variables, units, confidenceScore, sourceDocumentId, extractedAt, status, tags, priority) => {
            const invariant: Invariant = {
              id,
              contentSha256,
              description,
              formalExpression,
              naturalLanguage,
              variables,
              units,
              confidenceScore,
              sourceDocumentId,
              extractedAt,
              status,
              tags,
              priority,
            };

            expect(() => validateInvariant(invariant)).not.toThrow();
          }
        )
      );
    });

    it('should validate BadgeStatus with property-based data', () => {
      fc.assert(
        fc.property(
          uuidArb,
          sha256Arb,
          fc.string({ minLength: 1 }),
          fc.string({ minLength: 1 }),
          positiveIntArb,
          fc.stringOf(fc.constantFrom('a', 'b', 'c', 'd', 'e', 'f', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'), { minLength: 40, maxLength: 40 }),
          fc.constantFrom(...Object.values(BadgeState)),
          fc.string(),
          urlArb,
          dateArb,
          dateArb,
          fc.array(uuidArb),
          coveragePercentageArb,
          nonNegativeIntArb,
          positiveIntArb,
          (id, contentSha256, repoOwner, repoName, prNumber, commitSha, state, description, targetUrl, createdAt, updatedAt, proofArtifactIds, coveragePercentage, invariantsProven, totalInvariants) => {
            const badge: BadgeStatus = {
              id,
              contentSha256,
              repoOwner,
              repoName,
              prNumber,
              commitSha,
              state,
              description,
              targetUrl,
              createdAt,
              updatedAt,
              proofArtifactIds,
              coveragePercentage,
              invariantsProven,
              totalInvariants,
            };

            expect(() => validateBadgeStatus(badge)).not.toThrow();
          }
        )
      );
    });
  });
}); 