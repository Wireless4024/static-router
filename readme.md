# Static route
Experimental static routing (no box) for http framework.  
if anyone want to adopt it to your http framework just
copy my idea and don't forget to implement better version of it.

This is just a prototype and macro doesn't implement yet. 

## Is this really faster?
No, if variable(s) didn't hold between `await` point (this will make future stateless).  
Yes, if variable(s) does hold between `await` point (this will cause future to have state).  

```rust
// you can check future size by `size_of_val(&future)`

// this future size 1
async fn stateless(var: usize) -> usize {
	var
}
// this future size 32
async fn stateful(var: usize) -> usize {
	async fn another(var: usize) -> usize { var + 1 }
	another(var).await - 1
}
```
Stateless future doesn't have much performance impact on boxing.  
But stateful is depends on size of state of future itself if future
is big mean they have bigger state to save and have a bigger vtable
based on `await` point


## Benchmark
result may vary on different machine. and this test only run with 2 routes.
```
baseline                time:   [13.562 ns 13.579 ns 13.599 ns]
stateless/bench-router  time:   [119.77 ns 119.98 ns 120.24 ns]
stateless/bench-boxed   time:   [148.87 ns 149.05 ns 149.25 ns]
stateful/bench-router   time:   [128.81 ns 130.26 ns 132.48 ns]
stateful/bench-boxed    time:   [143.88 ns 144.05 ns 144.25 ns]
```