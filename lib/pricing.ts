export const MINIMUM_GOODS_CENTAVOS = 45_000;

/** Convert each displayed PHP unit price before summing to avoid float drift. */
export function goodsSubtotalCentavos(items: { price: number; quantity: number }[]): number {
  return items.reduce((total, item) => {
    const unitCentavos = Math.round(item.price * 100);
    if (!Number.isFinite(item.price) || item.price < 0 || !Number.isInteger(item.quantity) || item.quantity < 0) {
      throw new RangeError('Invalid basket price or quantity');
    }
    const next = total + unitCentavos * item.quantity;
    if (!Number.isSafeInteger(next)) throw new RangeError('Basket total exceeds supported precision');
    return next;
  }, 0);
}

/** Delivery fees do not contribute to the minimum goods order. */
export function meetsMinimumGoodsOrder(goodsCentavos: number): boolean {
  return Number.isSafeInteger(goodsCentavos) && goodsCentavos >= MINIMUM_GOODS_CENTAVOS;
}
