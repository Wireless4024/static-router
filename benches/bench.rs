#![feature(atomic_bool_fetch_not)]

use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use criterion::{black_box, Criterion, criterion_group, criterion_main};
use criterion::async_executor::FuturesExecutor;
use http::{Request, Response, StatusCode};
use matchit::{Params, Router};

use static_route::RouterStatic;

fn stateful(c: &mut Criterion) {
	let mut group = c.benchmark_group("Bench stateful");
	let mut router = Router::new();
	router.insert("/", 0).unwrap();
	router.insert("/api/v1/user", 1).unwrap();

	let mut i = 0;
	group.bench_function("baseline", |b| {
		b.iter(|| {
			if i & 1 == 1 {
				black_box(router.at("/").unwrap());
			} else {
				black_box(router.at("/api/v1/user").unwrap());
			}
			i += 1;
		});
	});

	group.bench_function("bench-router", |b| {
		let counter = Rc::new(AtomicBool::new(false));
		let router = Rc::new(RouterStatic::new());
		b.to_async(FuturesExecutor)
			.iter(move || {
				let counter = Rc::clone(&counter);
				let router = Rc::clone(&router);
				async move {
					if counter.load(Ordering::Relaxed) {
						assert_eq!(black_box(
							router.route(
								Request::get("/").body(()).unwrap()
							).await
						).status(), StatusCode::OK);
					} else {
						assert_eq!(black_box(
							router.route(
								Request::post("/api/v1/user").body(()).unwrap()
							).await
						).status(), StatusCode::OK);
					}
					counter.fetch_xor(true, Ordering::Relaxed);
				}
			});
	});

	{
		struct RouterBoxed {
			get: Router<Box<dyn Fn(Request<()>) -> Pin<Box<dyn Future<Output=Response<&'static str>>>>>>,
			post: Router<Box<dyn Fn(Request<()>) -> Pin<Box<dyn Future<Output=Response<&'static str>>>>>>,
		}

		impl RouterBoxed {
			fn new() -> Self {
				Self {
					get: Router::new(),
					post: Router::new(),
				}
			}

			fn build(mut self) -> Self {
				self.get.insert("/", Box::new(|req| Box::pin(Self::route_1(req)))).unwrap();
				self.post.insert("/api/v1/user", Box::new(|req| Box::pin(Self::route_2(req)))).unwrap();
				self
			}

			async fn route(&self, req: Request<()>) -> Response<&'static str> {
				match req.method().as_str() {
					"GET" => {
						self.route_get(req).await
					}
					"POST" => {
						self.route_post(req).await
					}
					// remaining method generate base on user requirement
					_ => {
						Self::not_found(req).await
					}
				}
			}

			async fn route_get(&self, req: Request<()>) -> Response<&'static str> {
				let path = req.uri().path();
				match self.get.at(path) {
					Ok(item) => {
						(item.value)(req).await
					}
					Err(_) => {
						Self::not_found(req).await
					}
				}
			}

			async fn route_post(&self, req: Request<()>) -> Response<&'static str> {
				let path = req.uri().path();
				match self.post.at(path) {
					Ok(item) => {
						(item.value)(req).await
					}
					Err(_) => {
						Self::not_found(req).await
					}
				}
			}

			async fn route_1(req: Request<()>) -> Response<&'static str> {
				let _ = async move { req }.await;
				http::Response::builder()
					.status(StatusCode::OK)
					.body("")
					.unwrap()
			}

			async fn route_2(req: Request<()>) -> Response<&'static str> {
				let _ = async move { req }.await;
				http::Response::builder()
					.status(StatusCode::OK)
					.body("")
					.unwrap()
			}
			async fn not_found(req: Request<()>) -> Response<&'static str> {
				let _ = async move { req }.await;
				http::Response::builder()
					.status(StatusCode::OK)
					.body("")
					.unwrap()
			}
		}
		let router = RouterBoxed::new().build();

		let router = Rc::new(router);
		group.bench_function("bench-boxed", |b| {
			let router = Rc::clone(&router);
			let counter = Rc::new(AtomicBool::new(false));
			b.to_async(FuturesExecutor)
				.iter(move || {
					let router = Rc::clone(&router);
					let counter = Rc::clone(&counter);
					async move {
						if counter.load(Ordering::Relaxed) {
							assert_eq!(black_box(
								router.route(Request::get("/").body(()).unwrap()).await
							).status(), StatusCode::OK);
						} else {
							assert_eq!(black_box(
								router.route(Request::post("/api/v1/user").body(()).unwrap()).await
							).status(), StatusCode::OK);
						}
						counter.fetch_xor(true, Ordering::Relaxed);
					}
				});
		});
	}
}

fn stateless(c: &mut Criterion) {
	let mut group = c.benchmark_group("Bench stateless");
	let mut router = Router::new();
	router.insert("/", 0).unwrap();
	router.insert("/api/v1/user", 1).unwrap();

	let mut i = 0;
	group.bench_function("baseline", |b| {
		b.iter(|| {
			if i & 1 == 1 {
				black_box(router.at("/").unwrap());
			} else {
				black_box(router.at("/api/v1/user").unwrap());
			}
			i += 1;
		});
	});

	{
		group.bench_function("bench-router", |b| {
			let counter = Rc::new(AtomicBool::new(false));
			let router = Rc::new(RouterStaticStateless::new());
			b.to_async(FuturesExecutor)
				.iter(move || {
					let counter = Rc::clone(&counter);
					let router = Rc::clone(&router);
					async move {
						if counter.load(Ordering::Relaxed) {
							assert_eq!(black_box(
								router.route(
									Request::get("/").body(()).unwrap()
								).await
							).status(), StatusCode::OK);
						} else {
							assert_eq!(black_box(
								router.route(
									Request::post("/api/v1/user").body(()).unwrap()
								).await
							).status(), StatusCode::OK);
						}
						counter.fetch_xor(true, Ordering::Relaxed);
					}
				});
		});
	}
	{
		struct RouterBoxed {
			get: Router<Box<dyn Fn(Request<()>) -> Pin<Box<dyn Future<Output=Response<&'static str>>>>>>,
			post: Router<Box<dyn Fn(Request<()>) -> Pin<Box<dyn Future<Output=Response<&'static str>>>>>>,
		}

		impl RouterBoxed {
			fn new() -> Self {
				Self {
					get: Router::new(),
					post: Router::new(),
				}
			}

			fn build(mut self) -> Self {
				self.get.insert("/", Box::new(|req| Box::pin(route_1_no_param(req)))).unwrap();
				self.post.insert("/api/v1/user", Box::new(|req| Box::pin(route_2_no_param(req)))).unwrap();
				self
			}

			async fn route(&self, req: Request<()>) -> Response<&'static str> {
				match req.method().as_str() {
					"GET" => {
						self.route_get(req).await
					}
					"POST" => {
						self.route_post(req).await
					}
					// remaining method generate base on user requirement
					_ => {
						Self::not_found(req).await
					}
				}
			}

			async fn route_get(&self, req: Request<()>) -> Response<&'static str> {
				let path = req.uri().path();
				match self.get.at(path) {
					Ok(item) => {
						(item.value)(req).await
					}
					Err(_) => {
						Self::not_found(req).await
					}
				}
			}

			async fn route_post(&self, req: Request<()>) -> Response<&'static str> {
				let path = req.uri().path();
				match self.post.at(path) {
					Ok(item) => {
						(item.value)(req).await
					}
					Err(_) => {
						Self::not_found(req).await
					}
				}
			}
			async fn not_found(req: Request<()>) -> Response<&'static str> {
				let _ = async move { req }.await;
				http::Response::builder()
					.status(StatusCode::OK)
					.body("")
					.unwrap()
			}
		}
		let router = RouterBoxed::new().build();

		let router = Rc::new(router);
		group.bench_function("bench-boxed", |b| {
			let router = Rc::clone(&router);
			let counter = Rc::new(AtomicBool::new(false));
			b.to_async(FuturesExecutor)
				.iter(move || {
					let router = Rc::clone(&router);
					let counter = Rc::clone(&counter);
					async move {
						if counter.load(Ordering::Relaxed) {
							assert_eq!(black_box(
								router.route(Request::get("/").body(()).unwrap()).await
							).status(), StatusCode::OK);
						} else {
							assert_eq!(black_box(
								router.route(Request::post("/api/v1/user").body(()).unwrap()).await
							).status(), StatusCode::OK);
						}
						counter.fetch_xor(true, Ordering::Relaxed);
					}
				});
		});
	}
}

async fn not_found(req: Request<()>) -> Response<&'static str> {
	let req = async move { req }.await;
	drop(req);
	http::Response::builder()
		.status(StatusCode::NOT_FOUND)
		.body("")
		.unwrap()
}

async fn route_1_stateless<'a, 'b>(_: &'a Request<()>, _: Params<'a, 'b>) -> Response<&'static str> {
	http::Response::builder()
		.status(StatusCode::OK)
		.body("")
		.unwrap()
}

async fn route_1_no_param(_: Request<()>) -> Response<&'static str> {
	http::Response::builder()
		.status(StatusCode::OK)
		.body("")
		.unwrap()
}

async fn route_2_stateless<'a, 'b>(_: &'a Request<()>, _: Params<'a, 'b>) -> Response<&'static str> {
	http::Response::builder()
		.status(StatusCode::OK)
		.body("")
		.unwrap()
}

async fn route_2_no_param(_: Request<()>) -> Response<&'static str> {
	http::Response::builder()
		.status(StatusCode::OK)
		.body("")
		.unwrap()
}

pub struct RouterStaticStateless {
	get: Router<u8>,
	post: Router<u8>,
}

impl RouterStaticStateless {
	pub fn new() -> Self {
		let mut get = Router::new();
		get.insert("/", 0).unwrap();
		let mut post = Router::new();
		post.insert("/api/v1/user", 0).unwrap();
		Self {
			get,
			post,
		}
	}

	pub async fn route(&self, req: Request<()>) -> Response<&'static str> {
		match req.method().as_str() {
			"GET" => {
				self.route_get(req).await
			}
			"POST" => {
				self.route_post(req).await
			}
			// remaining method generate base on user requirement
			_ => {
				not_found(req).await
			}
		}
	}

	// this function will generate based on method required with above method routing
	async fn route_get(&self, req: Request<()>) -> Response<&'static str> {
		let path = req.uri().path();
		match self.get.at(path) {
			Ok(item) => {
				match *item.value {
					0 => route_1_stateless(&req, item.params).await,
					_ => not_found(req).await,
				}
			}
			Err(_) => {
				not_found(req).await
			}
		}
	}

	// this function will generate based on method required with above method routing
	async fn route_post(&self, req: Request<()>) -> Response<&'static str> {
		let path = req.uri().path();
		match self.post.at(path) {
			Ok(item) => {
				match *item.value {
					0 => route_2_stateless(&req, item.params).await,
					_ => not_found(req).await,
				}
			}
			Err(_) => {
				not_found(req).await
			}
		}
	}
}

criterion_group!(benches, stateless, stateful);
criterion_main!(benches);