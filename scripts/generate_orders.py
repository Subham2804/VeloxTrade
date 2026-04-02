#!/usr/bin/env python3
"""
POST generated order batches to the VeloxTrade order-api (POST /orders).

Example:
  python scripts/generate_orders.py --url http://127.0.0.1:3000/orders --batches 5 --orders-per-batch 10
"""

from __future__ import annotations

import argparse
import json
import random
import sys
import urllib.error
import urllib.request
from typing import Any


def build_order(rng: random.Random) -> dict[str, Any]:
    return {
        "user_id": rng.randint(1, 10_000),
        "market_id": rng.randint(1, 256),
        "side": rng.choice(("buy", "sell")),
        "qty": rng.randint(1, 10_000),
        "price": round(rng.uniform(0.01, 10_000.0), 4),
    }


def post_orders(url: str, orders: list[dict[str, Any]], timeout_s: float) -> tuple[int, str]:
    body = json.dumps({"orders": orders}).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=body,
        method="POST",
        headers={
            "Content-Type": "application/json",
            "Accept": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        raw = resp.read().decode("utf-8")
        return resp.status, raw


def main() -> int:
    p = argparse.ArgumentParser(description="Generate and POST /orders payloads to order-api.")
    p.add_argument(
        "--url",
        default="http://127.0.0.1:3000/orders",
        help="Full URL for POST /orders (default: %(default)s)",
    )
    p.add_argument(
        "--batches",
        type=int,
        default=1,
        help="How many HTTP requests to send (default: %(default)s)",
    )
    p.add_argument(
        "--orders-per-batch",
        type=int,
        default=1,
        help="How many orders in each request body (default: %(default)s)",
    )
    p.add_argument(
        "--seed",
        type=int,
        default=None,
        help="RNG seed for reproducible payloads (default: random)",
    )
    p.add_argument(
        "--timeout",
        type=float,
        default=30.0,
        help="HTTP timeout in seconds (default: %(default)s)",
    )
    p.add_argument(
        "--quiet",
        action="store_true",
        help="Only print errors",
    )
    args = p.parse_args()

    if args.batches < 1 or args.orders_per_batch < 1:
        print("batches and orders-per-batch must be >= 1", file=sys.stderr)
        return 2

    rng = random.Random(args.seed)
    failures = 0

    for i in range(args.batches):
        orders = [build_order(rng) for _ in range(args.orders_per_batch)]
        try:
            status, raw = post_orders(args.url, orders, args.timeout)
        except urllib.error.HTTPError as e:
            failures += 1
            err_body = e.read().decode("utf-8", errors="replace")
            print(f"batch {i + 1}/{args.batches} HTTP {e.code}: {err_body}", file=sys.stderr)
            continue
        except urllib.error.URLError as e:
            failures += 1
            print(f"batch {i + 1}/{args.batches} request failed: {e}", file=sys.stderr)
            continue

        if not args.quiet:
            try:
                parsed = json.loads(raw)
                preview = json.dumps(parsed, indent=2) if len(raw) < 4000 else raw[:4000] + "\n…"
            except json.JSONDecodeError:
                preview = raw[:2000]
            print(f"batch {i + 1}/{args.batches} HTTP {status}\n{preview}\n")

    if failures:
        print(f"done: {failures}/{args.batches} batch(es) failed", file=sys.stderr)
        return 1
    if args.quiet:
        print(f"ok: {args.batches} batch(es), {args.orders_per_batch} order(s) each")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
