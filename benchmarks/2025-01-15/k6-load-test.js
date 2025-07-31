import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { htmlReport } from "https://raw.githubusercontent.com/benc-uk/k6-reporter/main/dist/bundle.js";

// Custom metrics
const proofSuccessRate = new Rate('proof_success_rate');
const proofFailureRate = new Rate('proof_failure_rate');
const specProcessingTime = new Trend('spec_processing_time');
const proofGenerationTime = new Trend('proof_generation_time');
const invariantExtractionTime = new Trend('invariant_extraction_time');
const totalEndToEndTime = new Trend('total_end_to_end_time');
const activeProofs = new Counter('active_proofs');
const completedProofs = new Counter('completed_proofs');

// Configuration
export const options = {
  stages: [
    // Warm-up phase
    { duration: '2m', target: 10 },
    // Ramp-up phase
    { duration: '5m', target: 50 },
    // Sustained load phase (target: 1K specs in < 30 min)
    { duration: '25m', target: 100 },
    // Peak load phase
    { duration: '5m', target: 150 },
    // Cool-down phase
    { duration: '3m', target: 0 },
  ],
  thresholds: {
    // Performance thresholds
    'http_req_duration': ['p(99)<90000'], // p99 < 90s
    'http_req_failed': ['rate<0.05'], // < 5% failure rate
    'proof_success_rate': ['rate>0.95'], // > 95% success rate
    'total_end_to_end_time': ['p(99)<90000'], // p99 < 90s
    'spec_processing_time': ['p(95)<30000'], // p95 < 30s
    'proof_generation_time': ['p(95)<60000'], // p95 < 60s
    'invariant_extraction_time': ['p(95)<15000'], // p95 < 15s
  },
};

// Test data generation
const testSpecs = [
  {
    title: "Simple mathematical theorem",
    content: "For all natural numbers n, if n is even, then n² is even.",
    complexity: "low"
  },
  {
    title: "Complex algorithm verification",
    content: "The quicksort algorithm has O(n log n) average time complexity and O(n²) worst-case time complexity.",
    complexity: "medium"
  },
  {
    title: "Cryptographic protocol verification",
    content: "In the RSA encryption scheme, if p and q are distinct prime numbers, then φ(n) = (p-1)(q-1) where n = pq.",
    complexity: "high"
  },
  {
    title: "Database transaction consistency",
    content: "ACID properties ensure that database transactions are processed reliably: Atomicity, Consistency, Isolation, and Durability.",
    complexity: "medium"
  },
  {
    title: "Distributed system consensus",
    content: "The Raft consensus algorithm ensures that if a majority of servers are operational, the system can make progress.",
    complexity: "high"
  }
];

// Helper functions
function generateRandomSpec() {
  const spec = testSpecs[Math.floor(Math.random() * testSpecs.length)];
  const id = Math.random().toString(36).substring(7);
  
  return {
    id: id,
    title: `${spec.title} - ${id}`,
    content: spec.content,
    complexity: spec.complexity,
    metadata: {
      author: `test-user-${id}`,
      created_at: new Date().toISOString(),
      tags: ['benchmark', 'test', spec.complexity]
    }
  };
}

function sleepRandom(min, max) {
  const delay = Math.random() * (max - min) + min;
  sleep(delay / 1000); // Convert to seconds
}

// Main test scenario
export default function() {
  const baseUrl = __ENV.BASE_URL || 'http://localhost:8080';
  const spec = generateRandomSpec();
  
  // Start timing
  const startTime = Date.now();
  
  // Step 1: Submit spec for processing
  const submitResponse = http.post(`${baseUrl}/api/v1/specs`, JSON.stringify(spec), {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${__ENV.API_TOKEN || 'test-token'}`,
    },
  });
  
  check(submitResponse, {
    'spec submission successful': (r) => r.status === 202,
    'spec submission response time < 5s': (r) => r.timings.duration < 5000,
  });
  
  if (submitResponse.status !== 202) {
    proofFailureRate.add(1);
    return;
  }
  
  const specId = submitResponse.json('spec_id');
  const specProcessingDuration = Date.now() - startTime;
  specProcessingTime.add(specProcessingDuration);
  
  // Step 2: Wait for invariant extraction
  let invariantResponse;
  let invariantExtractionDuration = 0;
  const invariantStartTime = Date.now();
  
  for (let i = 0; i < 30; i++) { // Wait up to 30 seconds
    invariantResponse = http.get(`${baseUrl}/api/v1/specs/${specId}/invariants`, {
      headers: {
        'Authorization': `Bearer ${__ENV.API_TOKEN || 'test-token'}`,
      },
    });
    
    if (invariantResponse.status === 200) {
      const invariants = invariantResponse.json('invariants');
      if (invariants && invariants.length > 0) {
        invariantExtractionDuration = Date.now() - invariantStartTime;
        invariantExtractionTime.add(invariantExtractionDuration);
        break;
      }
    }
    
    sleep(1); // Wait 1 second before next check
  }
  
  // Step 3: Monitor proof generation
  let proofResponse;
  let proofGenerationDuration = 0;
  const proofStartTime = Date.now();
  activeProofs.add(1);
  
  for (let i = 0; i < 120; i++) { // Wait up to 120 seconds (2 minutes)
    proofResponse = http.get(`${baseUrl}/api/v1/specs/${specId}/proof`, {
      headers: {
        'Authorization': `Bearer ${__ENV.API_TOKEN || 'test-token'}`,
      },
    });
    
    if (proofResponse.status === 200) {
      const proof = proofResponse.json('proof');
      if (proof && proof.status === 'completed') {
        proofGenerationDuration = Date.now() - proofStartTime;
        proofGenerationTime.add(proofGenerationDuration);
        completedProofs.add(1);
        
        // Verify proof quality
        check(proof, {
          'proof has valid structure': (p) => p.theorem && p.artifact && p.verification_status,
          'proof verification successful': (p) => p.verification_status === 'verified',
          'proof has reasonable size': (p) => p.artifact && p.artifact.size < 1000000, // < 1MB
        });
        
        proofSuccessRate.add(1);
        break;
      } else if (proof && proof.status === 'failed') {
        proofFailureRate.add(1);
        break;
      }
    }
    
    sleep(1); // Wait 1 second before next check
  }
  
  // Calculate total end-to-end time
  const totalDuration = Date.now() - startTime;
  totalEndToEndTime.add(totalDuration);
  
  // Log detailed metrics
  console.log(`Spec ${specId}: Processing=${specProcessingDuration}ms, Invariant=${invariantExtractionDuration}ms, Proof=${proofGenerationDuration}ms, Total=${totalDuration}ms`);
  
  // Random sleep between requests to simulate real usage
  sleepRandom(1000, 5000);
}

// Handle test completion
export function handleSummary(data) {
  return {
    'benchmarks/2025-01-15/load-test-report.html': htmlReport(data),
    'benchmarks/2025-01-15/load-test-summary.json': JSON.stringify(data, null, 2),
  };
}

// Setup and teardown functions
export function setup() {
  console.log('Starting load test for Spec-to-Proof platform');
  console.log(`Target: 1K specs → proofs in < 30 min, p99 latency < 90s`);
  console.log(`Base URL: ${__ENV.BASE_URL || 'http://localhost:8080'}`);
  
  // Verify platform is accessible
  const healthResponse = http.get(`${__ENV.BASE_URL || 'http://localhost:8080'}/health`);
  if (healthResponse.status !== 200) {
    throw new Error('Platform is not accessible');
  }
  
  return {
    startTime: new Date().toISOString(),
    testConfig: {
      targetSpecs: 1000,
      maxDuration: '30m',
      p99LatencyThreshold: 90000,
      successRateThreshold: 0.95
    }
  };
}

export function teardown(data) {
  console.log('Load test completed');
  console.log(`Test duration: ${data.endTime - data.startTime}ms`);
  console.log(`Total requests: ${data.metrics.http_reqs.values.count}`);
  console.log(`Success rate: ${data.metrics.proof_success_rate.values.rate}`);
  console.log(`P99 latency: ${data.metrics.http_req_duration.values.p(99)}ms`);
} 