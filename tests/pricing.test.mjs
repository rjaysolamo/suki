import { test } from 'node:test';
import assert from 'node:assert/strict';
import { goodsSubtotalCentavos, meetsMinimumGoodsOrder } from '../lib/pricing.ts';

test('minimum order boundary excludes delivery fees', () => {
  assert.equal(meetsMinimumGoodsOrder(44_999), false);
  assert.equal(meetsMinimumGoodsOrder(45_000), true);
  assert.equal(meetsMinimumGoodsOrder(45_001), true);
  assert.equal(meetsMinimumGoodsOrder(0), false);
});

test('goods total uses quantities and exact centavos', () => {
  assert.equal(goodsSubtotalCentavos([{ price: 160, quantity: 3 }]), 48_000);
  assert.equal(goodsSubtotalCentavos([{ price: 0.1, quantity: 3 }, { price: 0.2, quantity: 1 }]), 50);
  assert.equal(goodsSubtotalCentavos([]), 0);
});

test('invalid quantities, prices, and unsafe totals are rejected', () => {
  for (const item of [{ price: -1, quantity: 1 }, { price: NaN, quantity: 1 }, { price: 80, quantity: -1 }, { price: 80, quantity: 1.5 }, { price: Number.MAX_SAFE_INTEGER, quantity: 50 }]) {
    assert.throws(() => goodsSubtotalCentavos([item]), RangeError);
  }
  assert.equal(meetsMinimumGoodsOrder(Infinity), false);
  assert.equal(meetsMinimumGoodsOrder(45000.1), false);
});
