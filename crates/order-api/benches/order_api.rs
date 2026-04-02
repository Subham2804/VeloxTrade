use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use order_api::types::order::{
    order_response_from, validate, OrderRequest, OrderResponse, OrdersRequest, Side,
};

fn sample_order() -> OrderRequest {
    OrderRequest {
        user_id: 1,
        market_id: 42,
        side: Side::Buy,
        qty: 100,
        price: 10.5,
    }
}

fn bench_validate(c: &mut Criterion) {
    let o = sample_order();
    c.bench_function("validate/single", |b| {
        b.iter(|| validate(black_box(&o)).unwrap());
    });
}

fn bench_order_response_from(c: &mut Criterion) {
    let o = sample_order();
    c.bench_function("order_response_from/single", |b| {
        b.iter(|| {
            order_response_from(
                black_box(o.clone()),
                black_box(1_u64),
                black_box(1_234_567_i64),
            )
        });
    });
}

fn bench_orders_request_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("orders_request/json_deserialize");
    for size in [1usize, 10, 100] {
        let req = OrdersRequest {
            orders: (0..size)
                .map(|i| OrderRequest {
                    user_id: i as u64,
                    market_id: 42,
                    side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
                    qty: 100 + i as u64,
                    price: 10.0 + i as f64,
                })
                .collect(),
        };
        let json = serde_json::to_string(&req).unwrap();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, j| {
            b.iter(|| serde_json::from_str::<OrdersRequest>(black_box(j.as_str())).unwrap());
        });
    }
    group.finish();
}

fn bench_order_response_serialize(c: &mut Criterion) {
    let o = sample_order();
    let resp = order_response_from(o, 1, 1_234_567);
    let json = serde_json::to_string(&resp).unwrap();
    c.bench_function("order_response/json_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&resp)).unwrap());
    });
    c.bench_function("order_response/json_deserialize", |b| {
        b.iter(|| serde_json::from_str::<OrderResponse>(black_box(json.as_str())).unwrap());
    });
}

criterion_group!(
    benches,
    bench_validate,
    bench_order_response_from,
    bench_orders_request_deserialize,
    bench_order_response_serialize,
);
criterion_main!(benches);
