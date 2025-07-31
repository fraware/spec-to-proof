import { createTRPCRouter } from './trpc';
import { z } from 'zod';
import { coverageRouter } from './coverage';

// OpenAPI configuration
export const openApiConfig = {
  title: 'Spec-to-Proof Coverage API',
  version: '1.0.0',
  description: 'API for retrieving coverage statistics and proof metrics',
  servers: [
    {
      url: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000',
      description: 'Development server',
    },
  ],
  tags: [
    {
      name: 'coverage',
      description: 'Coverage statistics and metrics',
    },
  ],
  security: [
    {
      bearerAuth: [],
    },
  ],
};

// API router that includes coverage endpoints
export const apiRouter = createTRPCRouter({
  coverage: coverageRouter,
});

// OpenAPI schema definitions
export const openApiSchemas = {
  CoverageStats: {
    type: 'object',
    properties: {
      repository: { type: 'string' },
      totalSpecs: { type: 'integer' },
      provenSpecs: { type: 'integer' },
      coveragePercentage: { type: 'number' },
      averageProofLatency: { type: 'number' },
      lastUpdated: { type: 'string', format: 'date-time' },
    },
    required: ['repository', 'totalSpecs', 'provenSpecs', 'coveragePercentage', 'averageProofLatency', 'lastUpdated'],
  },
  
  UserStoryCoverage: {
    type: 'object',
    properties: {
      id: { type: 'string' },
      title: { type: 'string' },
      specs: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            id: { type: 'string' },
            title: { type: 'string' },
            status: { type: 'string', enum: ['proven', 'pending', 'failed'] },
            proofLatency: { type: 'number' },
          },
          required: ['id', 'title', 'status'],
        },
      },
      coveragePercentage: { type: 'number' },
    },
    required: ['id', 'title', 'specs', 'coveragePercentage'],
  },
  
  CoverageRequest: {
    type: 'object',
    properties: {
      repository: { type: 'string' },
      timeRange: { type: 'string', enum: ['1d', '7d', '30d', '90d'] },
      includeUserStories: { type: 'boolean' },
    },
  },
  
  CoverageResponse: {
    type: 'object',
    properties: {
      overallStats: { $ref: '#/components/schemas/CoverageStats' },
      userStories: {
        type: 'array',
        items: { $ref: '#/components/schemas/UserStoryCoverage' },
      },
      trends: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            date: { type: 'string', format: 'date' },
            coveragePercentage: { type: 'number' },
            newSpecs: { type: 'integer' },
            provenSpecs: { type: 'integer' },
          },
          required: ['date', 'coveragePercentage', 'newSpecs', 'provenSpecs'],
        },
      },
      topPerformers: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            userId: { type: 'string' },
            userName: { type: 'string' },
            specsProven: { type: 'integer' },
            averageLatency: { type: 'number' },
          },
          required: ['userId', 'userName', 'specsProven', 'averageLatency'],
        },
      },
    },
    required: ['overallStats', 'trends', 'topPerformers'],
  },
  
  Error: {
    type: 'object',
    properties: {
      code: { type: 'string' },
      message: { type: 'string' },
      details: { type: 'object' },
    },
    required: ['code', 'message'],
  },
};

// OpenAPI paths
export const openApiPaths = {
  '/api/coverage': {
    get: {
      tags: ['coverage'],
      summary: 'Get coverage statistics',
      description: 'Retrieve overall coverage statistics and metrics',
      parameters: [
        {
          name: 'repository',
          in: 'query',
          description: 'Repository name to filter by',
          schema: { type: 'string' },
        },
        {
          name: 'timeRange',
          in: 'query',
          description: 'Time range for the data',
          schema: { type: 'string', enum: ['1d', '7d', '30d', '90d'] },
          default: '30d',
        },
        {
          name: 'includeUserStories',
          in: 'query',
          description: 'Include user stories breakdown',
          schema: { type: 'boolean' },
          default: false,
        },
      ],
      responses: {
        '200': {
          description: 'Coverage data retrieved successfully',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/CoverageResponse' },
            },
          },
        },
        '400': {
          description: 'Bad request',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
        '500': {
          description: 'Internal server error',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
      },
    },
  },
  
  '/api/coverage/repository/{repository}': {
    get: {
      tags: ['coverage'],
      summary: 'Get repository coverage',
      description: 'Retrieve coverage statistics for a specific repository',
      parameters: [
        {
          name: 'repository',
          in: 'path',
          required: true,
          description: 'Repository name',
          schema: { type: 'string' },
        },
      ],
      responses: {
        '200': {
          description: 'Repository coverage data retrieved successfully',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/CoverageStats' },
            },
          },
        },
        '404': {
          description: 'Repository not found',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
        '500': {
          description: 'Internal server error',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
      },
    },
  },
  
  '/api/coverage/user-story/{userStoryId}': {
    get: {
      tags: ['coverage'],
      summary: 'Get user story coverage',
      description: 'Retrieve coverage statistics for a specific user story',
      parameters: [
        {
          name: 'userStoryId',
          in: 'path',
          required: true,
          description: 'User story ID',
          schema: { type: 'string' },
        },
      ],
      responses: {
        '200': {
          description: 'User story coverage data retrieved successfully',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/UserStoryCoverage' },
            },
          },
        },
        '404': {
          description: 'User story not found',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
        '500': {
          description: 'Internal server error',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
      },
    },
  },
  
  '/api/coverage/trends': {
    get: {
      tags: ['coverage'],
      summary: 'Get coverage trends',
      description: 'Retrieve coverage trends over time',
      parameters: [
        {
          name: 'repository',
          in: 'query',
          description: 'Repository name to filter by',
          schema: { type: 'string' },
        },
        {
          name: 'timeRange',
          in: 'query',
          description: 'Time range for the trends',
          schema: { type: 'string', enum: ['1d', '7d', '30d', '90d'] },
          default: '30d',
        },
      ],
      responses: {
        '200': {
          description: 'Coverage trends retrieved successfully',
          content: {
            'application/json': {
              schema: {
                type: 'array',
                items: {
                  type: 'object',
                  properties: {
                    date: { type: 'string', format: 'date' },
                    coveragePercentage: { type: 'number' },
                    newSpecs: { type: 'integer' },
                    provenSpecs: { type: 'integer' },
                  },
                  required: ['date', 'coveragePercentage', 'newSpecs', 'provenSpecs'],
                },
              },
            },
          },
        },
        '500': {
          description: 'Internal server error',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
      },
    },
  },
  
  '/api/coverage/top-performers': {
    get: {
      tags: ['coverage'],
      summary: 'Get top performers',
      description: 'Retrieve top performers based on specs proven',
      parameters: [
        {
          name: 'repository',
          in: 'query',
          description: 'Repository name to filter by',
          schema: { type: 'string' },
        },
        {
          name: 'limit',
          in: 'query',
          description: 'Number of top performers to return',
          schema: { type: 'integer', minimum: 1, maximum: 50 },
          default: 10,
        },
      ],
      responses: {
        '200': {
          description: 'Top performers retrieved successfully',
          content: {
            'application/json': {
              schema: {
                type: 'array',
                items: {
                  type: 'object',
                  properties: {
                    userId: { type: 'string' },
                    userName: { type: 'string' },
                    specsProven: { type: 'integer' },
                    averageLatency: { type: 'number' },
                  },
                  required: ['userId', 'userName', 'specsProven', 'averageLatency'],
                },
              },
            },
          },
        },
        '500': {
          description: 'Internal server error',
          content: {
            'application/json': {
              schema: { $ref: '#/components/schemas/Error' },
            },
          },
        },
      },
    },
  },
};

// Security schemes
export const securitySchemes = {
  bearerAuth: {
    type: 'http',
    scheme: 'bearer',
    bearerFormat: 'JWT',
  },
};

// Export the complete OpenAPI specification
export const openApiSpec = {
  openapi: '3.0.0',
  info: {
    title: openApiConfig.title,
    version: openApiConfig.version,
    description: openApiConfig.description,
  },
  servers: openApiConfig.servers,
  tags: openApiConfig.tags,
  paths: openApiPaths,
  components: {
    schemas: openApiSchemas,
    securitySchemes,
  },
  security: openApiConfig.security,
}; 