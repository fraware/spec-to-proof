import { z } from 'zod';

// TypeScript interfaces for domain models
export interface SpecDocument {
  id: string;
  contentSha256: string;
  sourceSystem: string;
  sourceId: string;
  title: string;
  content: string;
  url: string;
  author: string;
  createdAt: Date;
  modifiedAt: Date;
  metadata: Record<string, string>;
  version: number;
  status: DocumentStatus;
}

export interface Invariant {
  id: string;
  contentSha256: string;
  description: string;
  formalExpression: string;
  naturalLanguage: string;
  variables: Variable[];
  units: Record<string, string>;
  confidenceScore: number;
  sourceDocumentId: string;
  extractedAt: Date;
  status: InvariantStatus;
  tags: string[];
  priority: Priority;
}

export interface Variable {
  name: string;
  type: string;
  description: string;
  unit: string;
  constraints: string[];
}

export interface InvariantSet {
  id: string;
  contentSha256: string;
  name: string;
  description: string;
  invariants: Invariant[];
  sourceDocumentIds: string[];
  createdAt: Date;
  modifiedAt: Date;
  status: InvariantSetStatus;
}

export interface LeanTheorem {
  id: string;
  contentSha256: string;
  theoremName: string;
  leanCode: string;
  sourceInvariantId: string;
  generatedAt: Date;
  status: TheoremStatus;
  compilationErrors: string[];
  proofStrategy: string;
  metadata: Record<string, string>;
}

export interface ProofArtifact {
  id: string;
  contentSha256: string;
  theoremId: string;
  invariantId: string;
  status: ProofStatus;
  attemptedAt: Date;
  durationMs: number;
  output: string;
  logs: string[];
  resourceUsage: ResourceUsage;
  proofStrategy: string;
  confidenceScore: number;
  metadata: Record<string, string>;
}

export interface ResourceUsage {
  cpuSeconds: number;
  memoryBytes: number;
  diskBytes: number;
  networkBytes: number;
}

export interface BadgeStatus {
  id: string;
  contentSha256: string;
  repoOwner: string;
  repoName: string;
  prNumber: number;
  commitSha: string;
  state: BadgeState;
  description: string;
  targetUrl: string;
  createdAt: Date;
  updatedAt: Date;
  proofArtifactIds: string[];
  coveragePercentage: number;
  invariantsProven: number;
  totalInvariants: number;
}

// Enums
export enum DocumentStatus {
  UNSPECIFIED = 'UNSPECIFIED',
  DRAFT = 'DRAFT',
  PUBLISHED = 'PUBLISHED',
  ARCHIVED = 'ARCHIVED',
}

export enum InvariantStatus {
  UNSPECIFIED = 'UNSPECIFIED',
  EXTRACTED = 'EXTRACTED',
  CONFIRMED = 'CONFIRMED',
  REJECTED = 'REJECTED',
  PROVEN = 'PROVEN',
  FAILED = 'FAILED',
}

export enum InvariantSetStatus {
  UNSPECIFIED = 'UNSPECIFIED',
  DRAFT = 'DRAFT',
  REVIEW = 'REVIEW',
  APPROVED = 'APPROVED',
  PROVEN = 'PROVEN',
  FAILED = 'FAILED',
}

export enum TheoremStatus {
  UNSPECIFIED = 'UNSPECIFIED',
  GENERATED = 'GENERATED',
  COMPILING = 'COMPILING',
  COMPILED = 'COMPILED',
  PROVING = 'PROVING',
  PROVEN = 'PROVEN',
  FAILED = 'FAILED',
}

export enum ProofStatus {
  UNSPECIFIED = 'UNSPECIFIED',
  PENDING = 'PENDING',
  RUNNING = 'RUNNING',
  SUCCESS = 'SUCCESS',
  FAILED = 'FAILED',
  TIMEOUT = 'TIMEOUT',
  ERROR = 'ERROR',
}

export enum BadgeState {
  UNSPECIFIED = 'UNSPECIFIED',
  PENDING = 'PENDING',
  SUCCESS = 'SUCCESS',
  FAILURE = 'FAILURE',
  ERROR = 'ERROR',
}

export enum Priority {
  UNSPECIFIED = 'UNSPECIFIED',
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL',
}

// Zod schemas for validation
export const DocumentStatusSchema = z.nativeEnum(DocumentStatus);
export const InvariantStatusSchema = z.nativeEnum(InvariantStatus);
export const InvariantSetStatusSchema = z.nativeEnum(InvariantSetStatus);
export const TheoremStatusSchema = z.nativeEnum(TheoremStatus);
export const ProofStatusSchema = z.nativeEnum(ProofStatus);
export const BadgeStateSchema = z.nativeEnum(BadgeState);
export const PrioritySchema = z.nativeEnum(Priority);

export const VariableSchema = z.object({
  name: z.string().min(1),
  type: z.string().min(1),
  description: z.string(),
  unit: z.string(),
  constraints: z.array(z.string()),
});

export const ResourceUsageSchema = z.object({
  cpuSeconds: z.number().nonnegative(),
  memoryBytes: z.number().nonnegative(),
  diskBytes: z.number().nonnegative(),
  networkBytes: z.number().nonnegative(),
});

export const SpecDocumentSchema = z.object({
  id: z.string().uuid(),
  contentSha256: z.string().regex(/^[a-fA-F0-9]{64}$/),
  sourceSystem: z.string().min(1),
  sourceId: z.string().min(1),
  title: z.string().min(1),
  content: z.string(),
  url: z.string().url(),
  author: z.string().min(1),
  createdAt: z.date(),
  modifiedAt: z.date(),
  metadata: z.record(z.string()),
  version: z.number().int().positive(),
  status: DocumentStatusSchema,
});

export const InvariantSchema = z.object({
  id: z.string().uuid(),
  contentSha256: z.string().regex(/^[a-fA-F0-9]{64}$/),
  description: z.string().min(1),
  formalExpression: z.string().min(1),
  naturalLanguage: z.string(),
  variables: z.array(VariableSchema),
  units: z.record(z.string()),
  confidenceScore: z.number().min(0).max(1),
  sourceDocumentId: z.string().uuid(),
  extractedAt: z.date(),
  status: InvariantStatusSchema,
  tags: z.array(z.string()),
  priority: PrioritySchema,
});

export const InvariantSetSchema = z.object({
  id: z.string().uuid(),
  contentSha256: z.string().regex(/^[a-fA-F0-9]{64}$/),
  name: z.string().min(1),
  description: z.string(),
  invariants: z.array(InvariantSchema),
  sourceDocumentIds: z.array(z.string().uuid()),
  createdAt: z.date(),
  modifiedAt: z.date(),
  status: InvariantSetStatusSchema,
});

export const LeanTheoremSchema = z.object({
  id: z.string().uuid(),
  contentSha256: z.string().regex(/^[a-fA-F0-9]{64}$/),
  theoremName: z.string().min(1),
  leanCode: z.string().min(1),
  sourceInvariantId: z.string().uuid(),
  generatedAt: z.date(),
  status: TheoremStatusSchema,
  compilationErrors: z.array(z.string()),
  proofStrategy: z.string(),
  metadata: z.record(z.string()),
});

export const ProofArtifactSchema = z.object({
  id: z.string().uuid(),
  contentSha256: z.string().regex(/^[a-fA-F0-9]{64}$/),
  theoremId: z.string().uuid(),
  invariantId: z.string().uuid(),
  status: ProofStatusSchema,
  attemptedAt: z.date(),
  durationMs: z.number().int().nonnegative(),
  output: z.string(),
  logs: z.array(z.string()),
  resourceUsage: ResourceUsageSchema,
  proofStrategy: z.string(),
  confidenceScore: z.number().min(0).max(1),
  metadata: z.record(z.string()),
});

export const BadgeStatusSchema = z.object({
  id: z.string().uuid(),
  contentSha256: z.string().regex(/^[a-fA-F0-9]{64}$/),
  repoOwner: z.string().min(1),
  repoName: z.string().min(1),
  prNumber: z.number().int().positive(),
  commitSha: z.string().regex(/^[a-fA-F0-9]{40}$/),
  state: BadgeStateSchema,
  description: z.string(),
  targetUrl: z.string().url(),
  createdAt: z.date(),
  updatedAt: z.date(),
  proofArtifactIds: z.array(z.string().uuid()),
  coveragePercentage: z.number().min(0).max(100),
  invariantsProven: z.number().int().nonnegative(),
  totalInvariants: z.number().int().positive(),
});

// Utility functions
export function calculateSha256(content: string): string {
  // This would be implemented with a proper SHA256 library
  // For now, returning a placeholder
  return 'placeholder-sha256-hash';
}

export function generateId(): string {
  return crypto.randomUUID();
}

export function validateSpecDocument(doc: unknown): SpecDocument {
  return SpecDocumentSchema.parse(doc);
}

export function validateInvariant(invariant: unknown): Invariant {
  return InvariantSchema.parse(invariant);
}

export function validateInvariantSet(set: unknown): InvariantSet {
  return InvariantSetSchema.parse(set);
}

export function validateLeanTheorem(theorem: unknown): LeanTheorem {
  return LeanTheoremSchema.parse(theorem);
}

export function validateProofArtifact(artifact: unknown): ProofArtifact {
  return ProofArtifactSchema.parse(artifact);
}

export function validateBadgeStatus(badge: unknown): BadgeStatus {
  return BadgeStatusSchema.parse(badge);
}

// Type guards
export function isSpecDocument(obj: unknown): obj is SpecDocument {
  return SpecDocumentSchema.safeParse(obj).success;
}

export function isInvariant(obj: unknown): obj is Invariant {
  return InvariantSchema.safeParse(obj).success;
}

export function isInvariantSet(obj: unknown): obj is InvariantSet {
  return InvariantSetSchema.safeParse(obj).success;
}

export function isLeanTheorem(obj: unknown): obj is LeanTheorem {
  return LeanTheoremSchema.safeParse(obj).success;
}

export function isProofArtifact(obj: unknown): obj is ProofArtifact {
  return ProofArtifactSchema.safeParse(obj).success;
}

export function isBadgeStatus(obj: unknown): obj is BadgeStatus {
  return BadgeStatusSchema.safeParse(obj).success;
}

// Conversion functions for API responses
export function fromApiSpecDocument(apiDoc: any): SpecDocument {
  return {
    id: apiDoc.id,
    contentSha256: apiDoc.content_sha256,
    sourceSystem: apiDoc.source_system,
    sourceId: apiDoc.source_id,
    title: apiDoc.title,
    content: apiDoc.content,
    url: apiDoc.url,
    author: apiDoc.author,
    createdAt: new Date(apiDoc.created_at),
    modifiedAt: new Date(apiDoc.modified_at),
    metadata: apiDoc.metadata || {},
    version: apiDoc.version,
    status: apiDoc.status as DocumentStatus,
  };
}

export function fromApiInvariant(apiInvariant: any): Invariant {
  return {
    id: apiInvariant.id,
    contentSha256: apiInvariant.content_sha256,
    description: apiInvariant.description,
    formalExpression: apiInvariant.formal_expression,
    naturalLanguage: apiInvariant.natural_language,
    variables: (apiInvariant.variables || []).map((v: any) => ({
      name: v.name,
      type: v.type,
      description: v.description,
      unit: v.unit,
      constraints: v.constraints || [],
    })),
    units: apiInvariant.units || {},
    confidenceScore: apiInvariant.confidence_score,
    sourceDocumentId: apiInvariant.source_document_id,
    extractedAt: new Date(apiInvariant.extracted_at),
    status: apiInvariant.status as InvariantStatus,
    tags: apiInvariant.tags || [],
    priority: apiInvariant.priority as Priority,
  };
}

export function toApiSpecDocument(doc: SpecDocument): any {
  return {
    id: doc.id,
    content_sha256: doc.contentSha256,
    source_system: doc.sourceSystem,
    source_id: doc.sourceId,
    title: doc.title,
    content: doc.content,
    url: doc.url,
    author: doc.author,
    created_at: doc.createdAt.toISOString(),
    modified_at: doc.modifiedAt.toISOString(),
    metadata: doc.metadata,
    version: doc.version,
    status: doc.status,
  };
}

export function toApiInvariant(invariant: Invariant): any {
  return {
    id: invariant.id,
    content_sha256: invariant.contentSha256,
    description: invariant.description,
    formal_expression: invariant.formalExpression,
    natural_language: invariant.naturalLanguage,
    variables: invariant.variables.map(v => ({
      name: v.name,
      type: v.type,
      description: v.description,
      unit: v.unit,
      constraints: v.constraints,
    })),
    units: invariant.units,
    confidence_score: invariant.confidenceScore,
    source_document_id: invariant.sourceDocumentId,
    extracted_at: invariant.extractedAt.toISOString(),
    status: invariant.status,
    tags: invariant.tags,
    priority: invariant.priority,
  };
} 