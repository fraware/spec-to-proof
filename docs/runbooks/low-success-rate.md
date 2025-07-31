# Low Success Rate Runbook

## Alert Description
Success rate has dropped below 95% threshold, indicating increased error rates in the system.

## Severity
**Info** - Business hours only

## Alert Details
- **Metric**: `rate(http_requests_total{status=~"2.."}[5m]) / rate(http_requests_total[5m]) < 0.95`
- **Duration**: 5 minutes
- **Team**: Platform

## Immediate Actions

### 1. Verify the Alert
```bash
# Check current success rate
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query?query=rate(http_requests_total{status=~"2.."}[5m])%20/%20rate(http_requests_total[5m])

# Check recent success rate trends
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query_range?query=rate(http_requests_total{status=~"2.."}[5m])%20/%20rate(http_requests_total[5m])&start=$(date -d '1 hour ago' +%s)&end=$(date +%s)&step=5m
```

### 2. Identify Affected Endpoints
```bash
# Check which endpoints are failing
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query?query=rate(http_requests_total{status=~"5.."}[5m])%20by%20(endpoint)

# Check error distribution by status code
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query?query=rate(http_requests_total{status=~"[4-5].."}[5m])%20by%20(status)
```

### 3. Quick Assessment
- [ ] Temporary spike (resolved)
- [ ] Sustained degradation (investigation needed)
- [ ] Specific endpoint failure (targeted fix)
- [ ] System-wide issue (infrastructure problem)

## Investigation Steps

### Step 1: Check Error Logs
```bash
# Check application logs for errors
kubectl logs -l app=spec-to-proof --tail=100 | grep ERROR

# Check for specific error patterns
kubectl logs -l app=spec-to-proof --tail=100 | grep -E "(500|502|503|504)"

# Check for timeout errors
kubectl logs -l app=spec-to-proof --tail=100 | grep -i timeout
```

### Step 2: Analyze Error Types
```bash
# Check error distribution
# 4xx errors: Client issues (bad requests, auth, etc.)
# 5xx errors: Server issues (internal errors, timeouts, etc.)

# Check for specific error patterns
kubectl logs -l app=spec-to-proof --tail=100 | jq '.level == "error"' | head -20
```

### Step 3: Check Dependencies
```bash
# Check database connectivity
kubectl exec -it deployment/spec-to-proof -- curl -s http://database:5432/health

# Check external service dependencies
kubectl exec -it deployment/spec-to-proof -- curl -s http://claude-api:8080/health

# Check storage services
kubectl exec -it deployment/spec-to-proof -- curl -s http://s3-proxy:8080/health
```

## Resolution Steps

### For Client Errors (4xx)
1. **Check request validation**
   ```rust
   // Example: Improve input validation
   pub fn validate_request(input: &str) -> Result<(), Error> {
       if input.is_empty() {
           return Err(Error::EmptyInput);
       }
       if input.len() > MAX_LENGTH {
           return Err(Error::InputTooLong);
       }
       Ok(())
   }
   ```

2. **Update API documentation**
   - Clarify required parameters
   - Add example requests
   - Document error responses

3. **Improve error messages**
   ```rust
   // Example: Better error messages
   pub fn process_request(input: &str) -> Result<Response, Error> {
       match validate_request(input) {
           Ok(_) => Ok(Response::Success),
           Err(e) => Err(Error::ValidationFailed(format!("Invalid input: {}", e)))
       }
   }
   ```

### For Server Errors (5xx)
1. **Check resource constraints**
   ```bash
   # Check CPU/Memory usage
   kubectl top pods -l app=spec-to-proof
   
   # Check disk space
   kubectl exec -it deployment/spec-to-proof -- df -h
   ```

2. **Implement circuit breakers**
   ```rust
   // Example: Circuit breaker pattern
   use tokio::time::{Duration, Instant};
   
   pub struct CircuitBreaker {
       failure_threshold: usize,
       timeout: Duration,
       last_failure: Option<Instant>,
       failure_count: usize,
   }
   
   impl CircuitBreaker {
       pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
           Self {
               failure_threshold,
               timeout,
               last_failure: None,
               failure_count: 0,
           }
       }
       
       pub async fn call<F, T, E>(&mut self, f: F) -> Result<T, E>
       where
           F: FnOnce() -> Result<T, E>,
       {
           if self.is_open() {
               return Err(E::CircuitBreakerOpen);
           }
           
           match f() {
               Ok(result) => {
                   self.on_success();
                   Ok(result)
               }
               Err(e) => {
                   self.on_failure();
                   Err(e)
               }
           }
       }
   }
   ```

3. **Add retry logic**
   ```rust
   // Example: Exponential backoff retry
   use tokio::time::{sleep, Duration};
   
   pub async fn retry_with_backoff<F, T, E>(
       mut f: F,
       max_retries: usize,
       base_delay: Duration,
   ) -> Result<T, E>
   where
       F: FnMut() -> Result<T, E>,
   {
       let mut delay = base_delay;
       
       for attempt in 0..=max_retries {
           match f() {
               Ok(result) => return Ok(result),
               Err(e) if attempt == max_retries => return Err(e),
               Err(_) => {
                   sleep(delay).await;
                   delay *= 2;
               }
           }
       }
       
       unreachable!()
   }
   ```

### For Timeout Errors
1. **Optimize slow operations**
   ```rust
   // Example: Async processing
   pub async fn process_large_request(input: &str) -> Result<Response, Error> {
       // Process in background
       let handle = tokio::spawn(async move {
           // Heavy processing
           process_data(input).await
       });
       
       // Wait with timeout
       match tokio::time::timeout(Duration::from_secs(30), handle).await {
           Ok(Ok(result)) => Ok(result),
           Ok(Err(e)) => Err(e),
           Err(_) => Err(Error::Timeout),
       }
   }
   ```

2. **Implement caching**
   ```rust
   // Example: Redis caching
   use redis::AsyncCommands;
   
   pub async fn get_cached_result(key: &str) -> Result<Option<String>, Error> {
       let mut redis = redis_client.get_async_connection().await?;
       redis.get(key).await.map_err(Error::RedisError)
   }
   
   pub async fn set_cached_result(key: &str, value: &str, ttl: usize) -> Result<(), Error> {
       let mut redis = redis_client.get_async_connection().await?;
       redis.set_ex(key, value, ttl).await.map_err(Error::RedisError)
   }
   ```

## Verification

### Step 1: Monitor Success Rate
```bash
# Check if success rate is improving
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query?query=rate(http_requests_total{status=~"2.."}[5m])%20/%20rate(http_requests_total[5m])

# Monitor error rate trends
curl -s https://grafana.spec-to-proof.com/api/datasources/proxy/1/api/v1/query_range?query=rate(http_requests_total{status=~"[4-5].."}[5m])&start=$(date -d '1 hour ago' +%s)&end=$(date +%s)&step=5m
```

### Step 2: Test Affected Endpoints
```bash
# Test specific endpoints
curl -X POST https://api.spec-to-proof.com/v1/invariants \
  -H "Content-Type: application/json" \
  -d '{"content": "test", "source_type": "test"}'

# Check response times
curl -w "@curl-format.txt" -o /dev/null -s https://api.spec-to-proof.com/health
```

### Step 3: Update Monitoring
- Add specific alerts for affected endpoints
- Set up detailed error tracking
- Monitor dependency health

## Prevention

### Code Quality
- [ ] Comprehensive error handling
- [ ] Input validation
- [ ] Timeout configuration
- [ ] Circuit breaker patterns

### Infrastructure
- [ ] Resource monitoring
- [ ] Auto-scaling policies
- [ ] Health checks
- [ ] Load balancing

### Monitoring
- [ ] Real-time error tracking
- [ ] Performance metrics
- [ ] Dependency monitoring
- [ ] User experience tracking

## Escalation

### When to Escalate
- Success rate drops below 90%
- Multiple endpoints affected
- Sustained degradation > 1 hour
- User impact reported

### Escalation Path
1. **Platform Team Lead**
   - Assess system-wide impact
   - Coordinate with infrastructure team
   - Plan rollback if needed

2. **Engineering Manager**
   - Evaluate user impact
   - Coordinate communication
   - Review incident response

## Related Documentation
- [Error Handling Guidelines](../error-handling.md)
- [Performance Optimization](../performance.md)
- [Monitoring Strategy](../monitoring.md)

## Runbook Owner
**Platform Team** - Responsible for maintaining system reliability and performance. 