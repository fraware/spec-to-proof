import { createTRPCReact } from '@trpc/react-query';
import { httpBatchLink } from '@trpc/client';
import { QueryClient } from '@tanstack/react-query';
import { z } from 'zod';

// Define the tRPC router type
export type AppRouter = {
  invariant: {
    list: {
      input: z.object({
        documentId: z.string().optional(),
        status: z.enum(['EXTRACTED', 'CONFIRMED', 'REJECTED']).optional(),
        limit: z.number().min(1).max(100).default(20),
        offset: z.number().min(0).default(0),
      });
      output: z.object({
        invariants: z.array(z.any()), // Will be validated by our types
        total: z.number(),
        hasMore: z.boolean(),
      });
    };
    update: {
      input: z.object({
        id: z.string().uuid(),
        description: z.string().min(1).optional(),
        formalExpression: z.string().min(1).optional(),
        naturalLanguage: z.string().optional(),
        status: z.enum(['CONFIRMED', 'REJECTED']).optional(),
        tags: z.array(z.string()).optional(),
        priority: z.enum(['LOW', 'MEDIUM', 'HIGH', 'CRITICAL']).optional(),
      });
      output: z.any(); // Will be validated by our types
    };
    split: {
      input: z.object({
        id: z.string().uuid(),
        invariants: z.array(z.object({
          description: z.string().min(1),
          formalExpression: z.string().min(1),
          naturalLanguage: z.string(),
          variables: z.array(z.object({
            name: z.string().min(1),
            type: z.string().min(1),
            description: z.string(),
            unit: z.string(),
            constraints: z.array(z.string()),
          })),
          tags: z.array(z.string()),
          priority: z.enum(['LOW', 'MEDIUM', 'HIGH', 'CRITICAL']),
        })),
      });
      output: z.object({
        originalId: z.string().uuid(),
        newInvariants: z.array(z.any()), // Will be validated by our types
      });
    };
    rename: {
      input: z.object({
        id: z.string().uuid(),
        description: z.string().min(1),
      });
      output: z.any(); // Will be validated by our types
    };
  };
  invariantSet: {
    create: {
      input: z.object({
        name: z.string().min(1),
        description: z.string(),
        invariantIds: z.array(z.string().uuid()),
        sourceDocumentIds: z.array(z.string().uuid()),
      });
      output: z.any(); // Will be validated by our types
    };
    publish: {
      input: z.object({
        id: z.string().uuid(),
      });
      output: z.object({
        success: z.boolean(),
        message: z.string(),
      });
    };
  };
};

// Create the tRPC client
export const trpc = createTRPCReact<AppRouter>();

// Create a query client
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: (failureCount, error: any) => {
        // Don't retry on 4xx errors
        if (error?.data?.code >= 400 && error?.data?.code < 500) {
          return false;
        }
        return failureCount < 3;
      },
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes
    },
    mutations: {
      retry: false,
    },
  },
});

// Create tRPC client configuration
export const trpcClient = trpc.createClient({
  links: [
    httpBatchLink({
      url: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001/trpc',
      // Add headers for authentication if needed
      headers: () => {
        const token = typeof window !== 'undefined' ? localStorage.getItem('auth-token') : null;
        return token ? { Authorization: `Bearer ${token}` } : {};
      },
    }),
  ],
});

// Error handling utilities
export class TRPCError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number,
    public data?: any
  ) {
    super(message);
    this.name = 'TRPCError';
  }
}

export function handleTRPCError(error: any): TRPCError {
  if (error?.data?.code) {
    return new TRPCError(
      error.data.message || 'An error occurred',
      error.data.code,
      error.data.httpStatus || 500,
      error.data
    );
  }
  
  return new TRPCError(
    error?.message || 'An unknown error occurred',
    'UNKNOWN_ERROR',
    500,
    error
  );
}

// Server-side tRPC exports for testing and server implementations
export const createTRPCRouter = (routes: any) => routes;
export const publicProcedure = {
  input: (schema: any) => ({
    output: (schema: any) => ({
      query: (fn: any) => fn,
      mutation: (fn: any) => fn,
    }),
    query: (fn: any) => fn,
    mutation: (fn: any) => fn,
  }),
  output: (schema: any) => ({
    query: (fn: any) => fn,
    mutation: (fn: any) => fn,
  }),
  query: (fn: any) => fn,
  mutation: (fn: any) => fn,
};
export const protectedProcedure = publicProcedure; 