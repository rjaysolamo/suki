export type Account = {
  id: string;
  name: string;
  email: string;
  phone: string;
  role: "customer" | "vendor" | "courier" | "admin";
  status: string;
};
export type MarketProduct = {
  id: number;
  vendor_id: string;
  vendor: string;
  name: string;
  local_name: string;
  description: string;
  category: string;
  unit: string;
  price_centavos: number;
  image: string;
  available: boolean;
  updated_at: number;
};
export type DeliveryQuote = {
  id: string;
  address: string;
  lat: number;
  lng: number;
  distance_meters: number;
  delivery_fee_centavos: number;
  base_fee_centavos: number;
  per_km_centavos: number;
  expires_at: number;
};
export type Settings = {
  base_fee_centavos: number;
  per_km_centavos: number;
  minimum_centavos: number;
  gcash_qr_url: string;
  pickup_lat: number;
  pickup_lng: number;
  south: number;
  north: number;
  west: number;
  east: number;
  max_distance_meters: number;
  chat_retention_days: number;
};
export type ServerOrder = {
  id: string;
  customer_id: string;
  courier_id: string | null;
  status: string;
  payment_status: string;
  address: string;
  lat: number;
  lng: number;
  distance_meters: number;
  delivery_fee_centavos: number;
  goods_subtotal_centavos: number;
  estimated_total_centavos: number;
  final_goods_centavos: number | null;
  authorized_goods_centavos: number;
  created_at: number;
  updated_at: number;
};
export type OrderDetail = {
  order: ServerOrder;
  items: {
    id: string;
    product_id: number;
    vendor_id: string;
    name: string;
    unit: string;
    unit_price_centavos: number;
    quantity_milli: number;
    estimated_cost_centavos: number;
    actual_cost_centavos: number | null;
    actual_quantity_milli: number | null;
    actual_unit_price_centavos: number | null;
    unavailable: boolean;
  }[];
  payments: {
    id: string;
    reference: string;
    status: string;
    claimed_amount_centavos: number;
    received_amount_centavos: number | null;
  }[];
  refunds: [string, number, string][];
  adjustments: [string, number, string, string][];
  substitutions: [string, string, string, number, number, string][];
  history: [string, string, number][];
};
export async function api<T>(
  path: string,
  method = "GET",
  body?: unknown,
): Promise<T> {
  const response = await fetch("/api" + path, {
    method,
    credentials: "same-origin",
    headers: { "Content-Type": "application/json", "X-Suki-Request": "1" },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
  });
  const result = await response
    .json()
    .catch(() => ({ error: "The server returned an invalid response" }));
  if (!response.ok)
    throw new Error(result.error || `Request failed (${response.status})`);
  return result as T;
}
