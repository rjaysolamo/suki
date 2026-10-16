"use client";

import { useEffect, useMemo, useState, useRef } from "react";
import { goodsSubtotalCentavos } from "../lib/pricing";
import {
  api,
  type Account,
  type MarketProduct,
  type Settings,
  type DeliveryQuote,
  type OrderDetail,
  type ServerOrder,
} from "../lib/api";
import {
  ArrowRight,
  ArrowUpRight,
  MapPin,
  ChevronDown,
  Search,
  ShoppingBasket,
  Leaf,
  Heart,
  Plus,
  Minus,
  X,
  Check,
  ShieldCheck,
  Truck,
  SlidersHorizontal,
  ArrowLeft,
  Sprout,
  Apple,
  Fish,
  Beef,
  Egg,
  Store,
  Package,
  MessageCircle,
  Send,
  CheckCircle2,
  Info,
  Trash2,
} from "lucide-react";

type Product = {
  id: number;
  name: string;
  local: string;
  category: string;
  price: number;
  unit: string;
  vendor: string;
  image: string;
  tag?: string;
  stock: boolean;
  vendor_id?: string;
  raw?: MarketProduct;
};
const categories = [
  { name: "All products", icon: ShoppingBasket },
  { name: "Vegetables", icon: Sprout },
  { name: "Fruits", icon: Apple },
  { name: "Meat & Poultry", icon: Beef },
  { name: "Fish & Seafood", icon: Fish },
  { name: "Pantry & Others", icon: Egg },
];
const money = (value: number) =>
  new Intl.NumberFormat("en-PH", {
    style: "currency",
    currency: "PHP",
    minimumFractionDigits: 2,
  }).format(value);
type Order = {
  raw?: OrderDetail;
  id: string;
  items: { name: string; quantity: number; price: number; unit: string }[];
  subtotal: number;
  address: string;
  created: string;
  status: string;
};
type Cart = Record<number, number>;

function displayOrder(raw: OrderDetail): Order {
  return {
    raw,
    id: raw.order.id,
    items: raw.items.map((i) => ({
      name: i.name,
      quantity: i.quantity_milli / 1000,
      price: i.unit_price_centavos / 100,
      unit: i.unit,
    })),
    subtotal: raw.order.goods_subtotal_centavos / 100,
    address: raw.order.address,
    created: new Date(raw.order.created_at * 1000).toISOString(),
    status: raw.order.status.replaceAll("_", " "),
  };
}
export default function Home() {
  const [user, setUser] = useState<Account | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [apiError, setApiError] = useState("");
  const [busy, setBusy] = useState(false);
  const [authMode, setAuthMode] = useState("login");
  const [authError, setAuthError] = useState("");
  const [lat, setLat] = useState("");
  const [lng, setLng] = useState("");
  const [quote, setQuote] = useState<DeliveryQuote | null>(null);
  const [reference, setReference] = useState("");
  const submitKey = useRef({ payload: "", key: "" });
  async function refreshCatalog() {
    const list = await api<MarketProduct[]>("/products");
    setProducts(
      list.map((p) => ({
        id: p.id,
        name: p.name,
        local: p.local_name,
        category: p.category,
        price: p.price_centavos / 100,
        unit: p.unit,
        vendor: p.vendor,
        image: p.image || "/images/hero.jpg",
        stock: p.available,
        vendor_id: p.vendor_id,
        raw: p,
      })),
    );
  }
  async function refreshOrders(account = user) {
    if (!account || account.role === "vendor") {
      setOrders([]);
      return;
    }
    const list = await api<ServerOrder[]>("/orders");
    const details = await Promise.all(
      list.map((o) => api<OrderDetail>("/orders/" + o.id)),
    );
    setOrders(details.map(displayOrder));
  }
  async function refreshOrder(oid: string) {
    const order = displayOrder(await api<OrderDetail>("/orders/" + oid));
    setActiveOrder(order);
    setOrders((o) => o.map((v) => (v.id === oid ? order : v)));
    return order;
  }
  async function authenticate(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setBusy(true);
    setAuthError("");
    const fields = new FormData(e.currentTarget);
    const email = String(fields.get("email"));
    const password = String(fields.get("password"));
    try {
      if (authMode === "register") {
        const result = await api<{ status: string }>("/auth/register", "POST", {
          name: fields.get("name"),
          email,
          phone: fields.get("phone"),
          password,
          role: fields.get("role"),
          shop_name: fields.get("shop_name") || null,
        });
        if (result.status === "pending") {
          setAuthError(
            "Your vendor account was created and is awaiting administrator approval.",
          );
          setAuthMode("login");
          return;
        }
      }
      await api("/auth/login", "POST", { email, password });
      const account = await api<Account>("/auth/me");
      setUser(account);
      await refreshOrders(account);
      setModal(null);
      setToast("Welcome, " + account.name + ".");
    } catch (e) {
      setAuthError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function signOut() {
    try {
      await api("/auth/logout", "POST");
      setUser(null);
      setOrders([]);
      setActiveOrder(null);
      setRoleMenu(false);
      setView("shop");
    } catch (e) {
      setToast((e as Error).message);
    }
  }
  async function requestQuote() {
    setBusy(true);
    setQuote(null);
    setApiError("");
    try {
      const q = await api<DeliveryQuote>("/delivery/quote", "POST", {
        address,
        lat: Number(lat),
        lng: Number(lng),
      });
      setQuote(q);
    } catch (e) {
      setApiError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function submitPayment() {
    if (!activeOrder?.raw) return;
    setBusy(true);
    try {
      const o = activeOrder.raw;
      const paid = o.payments
        .filter((p) => p.status === "paid")
        .reduce((sum, p) => sum + (p.received_amount_centavos || 0), 0);
      await api("/orders/" + o.order.id + "/payments", "POST", {
        reference,
        amount_centavos:
          o.order.authorized_goods_centavos +
          o.order.delivery_fee_centavos -
          paid,
      });
      await refreshOrder(o.order.id);
      setReference("");
      setToast("Payment reference submitted for administrator verification.");
    } catch (e) {
      setToast((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function openChat() {
    if (!activeOrder) return;
    setMessages([]);
    setModal("chat");
  }
  async function sendChat(e: React.FormEvent) {
    e.preventDefault();
    if (!message.trim() || !activeOrder) return;
    setBusy(true);
    try {
      await api("/orders/" + activeOrder.id + "/messages", "POST", {
        content: message.trim(),
      });
      setMessage("");
      await loadMessages(activeOrder.id);
    } catch (e) {
      setToast((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function loadMessages(oid: string) {
    type ChatRow = {
      id: number;
      content: string;
      sender_id: string;
      sender_name: string;
      created_at: number;
    };
    let rows: ChatRow[] = [];
    let after = 0;
    while (true) {
      const page = await api<ChatRow[]>(
        "/orders/" + oid + "/messages?after=" + after,
      );
      rows = rows.concat(page);
      if (page.length < 200) break;
      after = page[page.length - 1].id;
    }
    setMessages(
      rows.map((m) => ({
        text: m.content,
        mine: m.sender_id === user?.id,
        name: m.sender_name,
      })),
    );
  }
  async function toggleAvailability(p: Product) {
    if (!p.raw) return;
    setBusy(true);
    try {
      await api("/products/" + p.id, "PUT", {
        name: p.raw.name,
        local_name: p.raw.local_name,
        description: p.raw.description,
        category: p.raw.category,
        unit: p.raw.unit,
        price_centavos: p.raw.price_centavos,
        image: p.raw.image,
        available: !p.stock,
      });
      await refreshCatalog();
      if (p.stock)
        setCart((c) => {
          const next = { ...c };
          delete next[p.id];
          return next;
        });
    } catch (e) {
      setToast((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  const [products, setProducts] = useState<Product[]>([]);
  const [cart, setCart] = useState<Cart>({});
  const [favorites, setFavorites] = useState<number[]>([]);
  const [ready, setReady] = useState(false);
  const [category, setCategory] = useState("All products");
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState("featured");
  const [onlyFavorites, setOnlyFavorites] = useState(false);
  const [view, setView] = useState("shop");
  const [modal, setModal] = useState<string | null>(null);
  const [selected, setSelected] = useState<Product | null>(null);
  const [orders, setOrders] = useState<Order[]>([]);
  const [activeOrder, setActiveOrder] = useState<Order | null>(null);
  const [address, setAddress] = useState("");
  const [toast, setToast] = useState("");
  const [mobileCart, setMobileCart] = useState(false);
  const [roleMenu, setRoleMenu] = useState(false);
  const [messages, setMessages] = useState<
    { text: string; mine: boolean; name?: string }[]
  >([]);
  const [message, setMessage] = useState("");
  const [availableOnly, setAvailableOnly] = useState(false);
  useEffect(() => {
    refreshCatalog().catch((e) => {
      setProducts([]);
      setApiError(e.message);
    });
    api<Settings>("/settings")
      .then(setSettings)
      .catch((e) => setApiError(e.message));
    api<Account>("/auth/me")
      .then((a) => {
        setUser(a);
        return refreshOrders(a);
      })
      .catch(() => {});
  }, []);
  useEffect(() => {
    setQuote(null);
  }, [address, lat, lng]);
  useEffect(() => {
    if (modal !== "chat" || !activeOrder || !user) return;
    let stopped = false;
    const refresh = () => {
      if (!stopped)
        loadMessages(activeOrder.id).catch((e) => setToast(e.message));
    };
    refresh();
    const timer = setInterval(refresh, 5000);
    let socket: WebSocket | null = null;
    let retry: ReturnType<typeof setTimeout> | undefined;
    let delay = 1000;
    const connect = () => {
      if (stopped) return;
      socket = new WebSocket(
        `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/api/orders/${activeOrder.id}/events`,
      );
      socket.onopen = () => {
        delay = 1000;
        refresh();
      };
      socket.onmessage = () => refresh();
      socket.onclose = () => {
        if (!stopped) {
          retry = setTimeout(connect, delay);
          delay = Math.min(30000, delay * 2);
        }
      };
      socket.onerror = () => socket?.close();
    };
    connect();
    return () => {
      stopped = true;
      clearInterval(timer);
      clearTimeout(retry);
      socket?.close();
    };
  }, [modal, activeOrder?.id, user?.id]);

  useEffect(() => {
    try {
      const saved = JSON.parse(localStorage.getItem("suki-demo") || "{}");
      if (saved.cart && typeof saved.cart === "object")
        setCart(
          Object.fromEntries(
            Object.entries(saved.cart).filter(
              ([id, q]) =>
                Number.isInteger(Number(id)) &&
                Number(id) > 0 &&
                typeof q === "number" &&
                Number.isInteger(q) &&
                q > 0 &&
                q <= 50,
            ),
          ) as Cart,
        );
      if (Array.isArray(saved.favorites))
        setFavorites(
          saved.favorites.filter((id: unknown) => typeof id === "number"),
        );
    } catch {}
    setReady(true);
  }, []);
  useEffect(() => {
    if (ready)
      localStorage.setItem("suki-demo", JSON.stringify({ cart, favorites }));
  }, [cart, favorites, ready]);
  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(""), 3000);
    return () => clearTimeout(timer);
  }, [toast]);
  useEffect(() => {
    if (!modal && !selected) return;
    const close = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setModal(null);
        setSelected(null);
      }
    };
    document.addEventListener("keydown", close);
    const previous = document.activeElement as HTMLElement | null;
    const dialog = document.querySelector<HTMLElement>("[role=dialog]");
    const controls = () =>
      Array.from(
        dialog?.querySelectorAll<HTMLElement>(
          'button:not([disabled]),input,select,a[href],[tabindex="0"]',
        ) || [],
      );
    const trap = (e: KeyboardEvent) => {
      if (e.key !== "Tab") return;
      const nodes = controls();
      const first = nodes[0];
      const last = nodes[nodes.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last?.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first?.focus();
      }
    };
    controls()[0]?.focus();
    document.addEventListener("keydown", trap);
    const old = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", close);
      document.removeEventListener("keydown", trap);
      document.body.style.overflow = old;
      previous?.focus();
    };
  }, [modal, selected]);
  const items = products.filter((p) => cart[p.id] > 0);
  const subtotal =
    goodsSubtotalCentavos(
      items.map((p) => ({ price: p.price, quantity: cart[p.id] })),
    ) / 100;
  const minimum = (settings?.minimum_centavos ?? 45000) / 100;
  const eligible =
    Math.round(subtotal * 100) >= (settings?.minimum_centavos ?? 45000);
  const count = Object.values(cart).reduce((s, q) => s + q, 0);
  const filtered = useMemo(
    () =>
      products
        .filter(
          (p) =>
            (category === "All products" || p.category === category) &&
            (!onlyFavorites || favorites.includes(p.id)) &&
            (!availableOnly || p.stock) &&
            `${p.name} ${p.local} ${p.vendor}`
              .toLowerCase()
              .includes(search.toLowerCase()),
        )
        .sort((a, b) =>
          sort === "low"
            ? a.price - b.price
            : sort === "high"
              ? b.price - a.price
              : 0,
        ),
    [products, category, search, sort, onlyFavorites, favorites, availableOnly],
  );
  function change(id: number, amount: number) {
    const p = products.find((p) => p.id === id);
    if (amount > 0 && !p?.stock) return;
    setCart((c) => {
      const next = {
        ...c,
        [id]: Math.min(50, Math.max(0, (c[id] || 0) + amount)),
      };
      if (!next[id]) delete next[id];
      return next;
    });
  }
  function favorite(id: number) {
    setFavorites((f) =>
      f.includes(id) ? f.filter((i) => i !== id) : [...f, id],
    );
  }
  function startCheckout() {
    if (user && user.role !== "customer") {
      setToast("Use a customer account to place an order.");
      return;
    }
    if (!eligible) {
      setToast(
        "Add " +
          money(minimum - subtotal) +
          " more to reach the minimum order.",
      );
      return;
    }
    setApiError("");
    setModal(user ? "checkout" : "auth");
    setMobileCart(false);
  }
  async function placeOrder(e: React.FormEvent) {
    e.preventDefault();
    if (!eligible || !quote || !user) return;
    setBusy(true);
    setApiError("");
    const payload = {
      quote_id: quote.id,
      items: items.map((p) => ({
        product_id: p.id,
        quantity_milli: cart[p.id] * 1000,
        expected_price_centavos: Math.round(p.price * 100),
      })),
    };
    const fingerprint = JSON.stringify(payload);
    if (submitKey.current.payload !== fingerprint)
      submitKey.current = { payload: fingerprint, key: crypto.randomUUID() };
    try {
      const result = await api<{ id: string }>("/orders", "POST", {
        ...payload,
        idempotency_key: submitKey.current.key,
      });
      const order = displayOrder(
        await api<OrderDetail>("/orders/" + result.id),
      );
      setOrders((o) => [order, ...o.filter((v) => v.id !== order.id)]);
      setActiveOrder(order);
      setCart({});
      setQuote(null);
      setModal("confirmed");
    } catch (e) {
      setApiError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  const basket = (
    <aside className={`basket ${mobileCart ? "mobile-open" : ""}`}>
      <div className="basket-heading">
        <div>
          <ShoppingBasket size={21} />
          <h2>Your basket</h2>
          <span className="count">{count}</span>
        </div>
        <button
          className="mobile-close icon-button"
          aria-label="Close basket"
          onClick={() => setMobileCart(false)}
        >
          <X size={20} />
        </button>
        <span className="basket-dot" />
      </div>
      <div className="basket-body">
        {items.length === 0 ? (
          <div className="empty-basket">
            <div className="basket-illustration">
              <ShoppingBasket size={46} strokeWidth={1.2} />
              <span>
                <Leaf size={19} />
              </span>
            </div>
            <h3>A little fresh, a lot of good.</h3>
            <p>
              Your basket is waiting for some
              <br />
              fresh market finds.
            </p>
            <button
              onClick={() => {
                document
                  .getElementById("catalog")
                  ?.scrollIntoView({ behavior: "smooth" });
                setMobileCart(false);
              }}
            >
              Let’s fill it up <ArrowRight size={15} />
            </button>
          </div>
        ) : (
          <div className="cart-items">
            {items.map((p) => (
              <div className="cart-item" key={p.id}>
                <img src={p.image} alt="" />
                <div>
                  <strong>{p.name}</strong>
                  <small>
                    {money(p.price)} / {p.unit}
                  </small>
                  <div className="small-stepper">
                    <button
                      onClick={() => change(p.id, -1)}
                      aria-label={`Remove one ${p.name}`}
                    >
                      <Minus size={12} />
                    </button>
                    <span>{cart[p.id]}</span>
                    <button
                      onClick={() => change(p.id, 1)}
                      aria-label={`Add one ${p.name}`}
                    >
                      <Plus size={12} />
                    </button>
                  </div>
                </div>
                <b>{money(p.price * cart[p.id])}</b>
              </div>
            ))}
          </div>
        )}
        <div className="minimum">
          <div>
            <span>
              {subtotal >= minimum ? (
                <>
                  <CheckCircle2 size={14} /> You’re ready to checkout!
                </>
              ) : (
                <>
                  <Info size={14} /> Minimum order of {money(minimum)}
                </>
              )}
            </span>
            <b>{Math.min(100, Math.round((subtotal / minimum) * 100))}%</b>
          </div>
          <div className="progress">
            <span
              style={{ width: `${Math.min(100, (subtotal / minimum) * 100)}%` }}
            />
          </div>
          <p>
            {subtotal
              ? subtotal >= minimum
                ? "Fresh finds, on their way to your table."
                : `Add ${money(minimum - subtotal)} more in goods to checkout.`
              : "A basket of goodness starts with fresh finds."}
          </p>
        </div>
        <div className="basket-totals">
          <div>
            <span>Goods subtotal</span>
            <b>{money(subtotal)}</b>
          </div>
          <div>
            <span>
              Delivery fee <Info size={12} />
            </span>
            <small>Calculated at checkout</small>
          </div>
          <div className="total">
            <strong>Estimated total</strong>
            <b>{money(subtotal)}</b>
          </div>
        </div>
        <button
          className="primary checkout"
          disabled={!eligible}
          onClick={startCheckout}
        >
          Proceed to checkout <ArrowRight size={17} />
        </button>
        <div className="secure">
          <ShieldCheck size={14} /> Safe checkout · Pay with <b>GCash</b>
        </div>
      </div>
      <div className="basket-note">
        <Leaf size={17} />
        <p>
          Fresh prices can change a little.
          <br />
          We’ll always check with you first.
        </p>
      </div>
    </aside>
  );
  return (
    <>
      <div className="announcement">
        <span>
          <Sprout size={14} /> A fresher way to shop local.
        </span>
        <span>
          From Bankerohan, with love. <Heart size={12} />
        </span>
      </div>
      <header>
        <div className="nav-wrap">
          <button
            className="brand"
            onClick={() => setView("shop")}
            aria-label="Suki home"
          >
            suki
            <span className="brand-leaf">
              <Leaf size={19} fill="currentColor" />
            </span>
            <span className="brand-period">.</span>
          </button>
          <div className="nav-divider" />
          <button className="location" onClick={() => setModal("location")}>
            <MapPin size={20} />
            <span>
              <small>DELIVERING TO</small>
              <b>
                Davao City <ChevronDown size={13} />
              </b>
            </span>
          </button>
          <nav>
            <button
              className={view === "shop" ? "active" : ""}
              onClick={() => setView("shop")}
            >
              Shop the market
            </button>
            <button
              className={view === "orders" ? "active" : ""}
              onClick={() => {
                if (user) {
                  setView("orders");
                  refreshOrders().catch((e) => setToast(e.message));
                } else setModal("auth");
              }}
            >
              My orders{" "}
              {orders.length > 0 && (
                <span className="nav-badge">{orders.length}</span>
              )}
            </button>
            <button onClick={() => setModal("how")}>
              How it works <ArrowUpRight size={13} />
            </button>
          </nav>
          <div className="nav-actions">
            <button
              className={`icon-button favorite-top ${onlyFavorites ? "is-favorite" : ""}`}
              onClick={() => {
                setOnlyFavorites(!onlyFavorites);
                setView("shop");
              }}
              aria-label="Show favorite products"
            >
              <Heart size={20} fill={onlyFavorites ? "currentColor" : "none"} />
            </button>
            <button
              className="mobile-basket icon-button"
              aria-label="Open basket"
              onClick={() => setMobileCart(true)}
            >
              <ShoppingBasket size={21} />
              <span>{count}</span>
            </button>
            <div className="profile-wrap">
              <button
                className="profile"
                onClick={() =>
                  user ? setRoleMenu(!roleMenu) : setModal("auth")
                }
              >
                <span>{user?.name.slice(0, 1) || "S"}</span>
                <span className="profile-name">
                  {user ? "Hi, " + user.name.split(" ")[0] : "Sign in"}{" "}
                  <ChevronDown size={13} />
                </span>
              </button>
              {roleMenu && (
                <div className="role-menu">
                  <strong>Welcome to Suki</strong>
                  <small>{user?.email}</small>
                  <button
                    onClick={() => {
                      setView("orders");
                      setRoleMenu(false);
                      refreshOrders().catch((e) => setToast(e.message));
                    }}
                  >
                    <Package size={16} /> My orders
                  </button>
                  {user?.role === "vendor" && (
                    <button
                      onClick={() => {
                        setModal("vendor");
                        setRoleMenu(false);
                      }}
                    >
                      <Store size={16} /> Vendor workspace
                    </button>
                  )}
                  <button onClick={signOut}>
                    <ArrowRight size={16} /> Sign out
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </header>
      <main>
        {apiError && modal !== "checkout" && (
          <div className="notice" role="alert">
            <Info size={18} />
            {apiError}
            <button
              onClick={() => {
                setApiError("");
                refreshCatalog().catch((e) => setApiError(e.message));
              }}
            >
              Retry
            </button>
          </div>
        )}
        {view === "shop" ? (
          <>
            <section className="hero">
              <div className="hero-copy">
                <span className="eyebrow">
                  <span /> YOUR NEIGHBORHOOD MARKET, ONLINE
                </span>
                <h1>
                  Fresh from the market.
                  <br />
                  Straight to your <em>door.</em>
                </h1>
                <p>
                  The best of Bankerohan, without the busy trip.
                  <br />
                  Fresh picks from local suki, delivered across Davao.
                </p>
                <button
                  className="primary hero-button"
                  onClick={() =>
                    document
                      .getElementById("catalog")
                      ?.scrollIntoView({ behavior: "smooth" })
                  }
                >
                  Shop fresh finds <ArrowRight size={17} />
                </button>
                <div className="hero-trust">
                  <span>
                    <ShieldCheck size={14} /> Trusted local vendors
                  </span>
                  <span className="tiny-dot" />
                  <span>Handpicked with care</span>
                </div>
              </div>
              <div className="hero-image">
                <img
                  src="/images/hero.jpg"
                  alt="An abundant selection of fresh vegetables at the market"
                />
                <div className="image-shade" />
                <div className="fresh-stamp">
                  <Sprout size={25} />
                  <span>
                    FRESH PICKS.
                    <br />
                    <b>LOCAL ROOTS.</b>
                  </span>
                </div>
                <div className="market-label">
                  <span>
                    <MapPin size={16} />
                  </span>
                  <div>
                    <b>Bankerohan Public Market</b>
                    <small>Davao’s freshest, every day.</small>
                  </div>
                  <ArrowUpRight size={17} />
                </div>
              </div>
            </section>
            <section className="benefits">
              <div>
                <span className="benefit-icon">
                  <Sprout size={22} />
                </span>
                <p>
                  <b>Fresh & locally sourced</b>
                  <small>Good food from people you know</small>
                </p>
              </div>
              <div>
                <span className="benefit-icon">
                  <Store size={21} />
                </span>
                <p>
                  <b>Support our local vendors</b>
                  <small>Every order helps a small business</small>
                </p>
              </div>
              <div>
                <span className="benefit-icon">
                  <Truck size={23} />
                </span>
                <p>
                  <b>Fair, transparent delivery</b>
                  <small>{money((settings?.base_fee_centavos??6000)/100)} base + {money((settings?.per_km_centavos??350)/100)} per kilometer</small>
                </p>
              </div>
              <div>
                <span className="benefit-icon gcash-icon">G</span>
                <p>
                  <b>Easy GCash payments</b>
                  <small>Scan, pay, and we’ll handle the rest</small>
                </p>
              </div>
            </section>
            <div className="shop-layout" id="catalog">
              <section className="catalog">
                <div className="catalog-heading">
                  <div>
                    <span className="section-kicker">
                      A LITTLE BIT OF EVERYTHING
                    </span>
                    <h2>
                      What’s fresh today<span>?</span>
                    </h2>
                  </div>
                  <div className="market-open">
                    <span /> Market is open{" "}
                    <span className="hours">· 6 AM – 6 PM</span>
                  </div>
                </div>
                <div className="categories">
                  {categories.map((c) => (
                    <button
                      key={c.name}
                      className={category === c.name ? "selected" : ""}
                      onClick={() => setCategory(c.name)}
                    >
                      <c.icon size={18} strokeWidth={1.6} />
                      {c.name}
                    </button>
                  ))}
                </div>
                <div className="catalog-tools">
                  <div className="search">
                    <Search size={18} />
                    <input
                      aria-label="Search the market"
                      placeholder="Search for fresh finds..."
                      value={search}
                      onChange={(e) => setSearch(e.target.value)}
                    />
                    {search && (
                      <button
                        aria-label="Clear search"
                        onClick={() => setSearch("")}
                      >
                        <X size={15} />
                      </button>
                    )}
                  </div>
                  <button
                    className={`filter-button ${availableOnly ? "enabled" : ""}`}
                    aria-label="Toggle in stock only"
                    title="Show only in stock products"
                    onClick={() => setAvailableOnly(!availableOnly)}
                  >
                    <SlidersHorizontal size={17} />
                    <span>Filters</span>
                    {availableOnly && <span className="filter-dot" />}
                  </button>
                  <div className="sort">
                    <label htmlFor="sort">Sort by:</label>
                    <select
                      id="sort"
                      value={sort}
                      onChange={(e) => setSort(e.target.value)}
                    >
                      <option value="featured">Recommended</option>
                      <option value="low">Price: low to high</option>
                      <option value="high">Price: high to low</option>
                    </select>
                  </div>
                </div>
                <div className="results">
                  <span>
                    {onlyFavorites
                      ? "Your favorites"
                      : category === "All products"
                        ? "Fresh picks for you"
                        : category}{" "}
                    <small>({filtered.length} products)</small>
                  </span>
                  {onlyFavorites ? (
                    <button onClick={() => setOnlyFavorites(false)}>
                      Show all <X size={12} />
                    </button>
                  ) : (
                    <small>
                      <span /> Vendor-managed prices
                    </small>
                  )}
                </div>
                <div className="product-grid">
                  {filtered.map((p) => (
                    <article className="product-card" key={p.id}>
                      <div className="product-image">
                        <button
                          className="product-photo"
                          onClick={() => setSelected(p)}
                          aria-label={`View ${p.name}`}
                        >
                          <img src={p.image} alt={p.name} />
                        </button>
                        {p.tag && (
                          <span
                            className={`product-tag ${p.tag === "LOCAL FAVORITE" ? "orange" : ""}`}
                          >
                            <Leaf size={10} />
                            {p.tag}
                          </span>
                        )}
                        <button
                          className={`heart ${favorites.includes(p.id) ? "saved" : ""}`}
                          aria-label={`${favorites.includes(p.id) ? "Unsave" : "Save"} ${p.name}`}
                          onClick={() => favorite(p.id)}
                        >
                          <Heart
                            size={16}
                            fill={
                              favorites.includes(p.id) ? "currentColor" : "none"
                            }
                          />
                        </button>
                        <span className="stock-label">
                          <span />
                          {p.stock ? "In stock" : "Out of stock"}
                        </span>
                      </div>
                      <div className="product-info">
                        <button
                          className="product-title"
                          onClick={() => setSelected(p)}
                        >
                          {p.name}
                        </button>
                        <span className="local-name">{p.local}</span>
                        <span className="vendor">
                          <Store size={12} />
                          {p.vendor}
                        </span>
                        <div className="product-bottom">
                          <div>
                            <b>₱{p.price.toFixed(2)}</b>
                            <span> / {p.unit}</span>
                          </div>
                          {cart[p.id] ? (
                            <div className="stepper">
                              <button
                                onClick={() => change(p.id, -1)}
                                aria-label={`Remove one ${p.name}`}
                              >
                                <Minus size={14} />
                              </button>
                              <span>{cart[p.id]}</span>
                              <button
                                onClick={() => change(p.id, 1)}
                                aria-label={`Add one ${p.name}`}
                              >
                                <Plus size={14} />
                              </button>
                            </div>
                          ) : (
                            <button
                              className="add-button"
                              disabled={!p.stock}
                              onClick={() => change(p.id, 1)}
                              aria-label={`Add ${p.name} to basket`}
                            >
                              <Plus size={17} />
                              <span>Add</span>
                            </button>
                          )}
                        </div>
                      </div>
                    </article>
                  ))}
                </div>
                {filtered.length === 0 && (
                  <div className="no-results">
                    <Search size={32} />
                    <h3>No fresh finds here yet</h3>
                    <p>Try another search or explore a different category.</p>
                    <button
                      className="text-link"
                      onClick={() => {
                        setCategory("All products");
                        setSearch("");
                        setOnlyFavorites(false);
                        setAvailableOnly(false);
                      }}
                    >
                      Explore all products <ArrowRight size={15} />
                    </button>
                  </div>
                )}
                <div className="catalog-footnote">
                  <Info size={14} /> Market prices and actual weights may vary.
                  You’ll approve any changes.
                </div>
              </section>
              {basket}
            </div>
            <section className="local-banner">
              <div className="local-icon">
                <Store size={34} strokeWidth={1.3} />
              </div>
              <div>
                <span>GOOD FOOD. GOOD NEIGHBORS.</span>
                <h2>Behind every fresh find is a local suki.</h2>
                <p>
                  Shop small, eat fresh, and keep our Davao community growing.
                </p>
              </div>
              <button onClick={() => setModal("vendors")}>
                Meet our vendors <ArrowUpRight size={16} />
              </button>
            </section>
          </>
        ) : (
          <section className="orders-page">
            <button className="text-link" onClick={() => setView("shop")}>
              <ArrowLeft size={16} /> Back to the market
            </button>
            <span className="section-kicker">YOUR MARKET JOURNEY</span>
            <h1>My orders</h1>
            <p>Every basket has a little local goodness.</p>
            {orders.length === 0 ? (
              <div className="empty-orders">
                <Package size={45} />
                <h2>Your first fresh delivery starts here.</h2>
                <p>Browse the market and fill a basket with your favorites.</p>
                <button className="primary" onClick={() => setView("shop")}>
                  Explore the market <ArrowRight size={16} />
                </button>
              </div>
            ) : (
              orders.map((o) => (
                <button
                  className="order-row"
                  key={o.id}
                  onClick={() => {
                    setActiveOrder(o);
                    setModal("order");
                  }}
                >
                  <span className="order-icon">
                    <ShoppingBasket size={25} />
                  </span>
                  <div>
                    <b>{o.id.slice(0, 8)}</b>
                    <small>
                      {new Date(o.created).toLocaleDateString("en-PH", {
                        month: "long",
                        day: "numeric",
                        year: "numeric",
                      })}{" "}
                      · {o.items.length} products
                    </small>
                  </div>
                  <span className="order-status">{o.status}</span>
                  <b>{money(o.subtotal)}</b>
                  <ArrowRight size={18} />
                </button>
              ))
            )}
          </section>
        )}
      </main>
      <footer>
        <a className="footer-brand" href="#">
          suki<span>.</span>
        </a>
        <span>Fresh finds. Local hearts. Davao, always.</span>
        <div>
          <button onClick={() => setModal("how")}>How it works</button>
          <button onClick={() => setModal("support")}>Need a hand?</button>
          <span>© {new Date().getFullYear()} Suki</span>
        </div>
      </footer>
      {toast && (
        <div className="toast" role="status">
          <CheckCircle2 size={18} />
          {toast}
        </div>
      )}
      {(modal || selected) && (
        <div
          className="modal-backdrop"
          onClick={() => {
            setModal(null);
            setSelected(null);
          }}
        >
          <section
            className={`modal ${modal === "checkout" ? "checkout-modal" : ""}`}
            role="dialog"
            aria-modal="true"
            aria-label={selected?.name || "Suki details"}
            onClick={(e) => e.stopPropagation()}
          >
            <button
              className="modal-close icon-button"
              aria-label="Close dialog"
              onClick={() => {
                setModal(null);
                setSelected(null);
              }}
            >
              <X size={20} />
            </button>
            {selected && (
              <>
                <img
                  className="detail-image"
                  src={selected.image}
                  alt={selected.name}
                />
                <div className="modal-content">
                  <span className="section-kicker">FRESH FROM BANKEROHAN</span>
                  <h2>{selected.name}</h2>
                  <p>
                    {selected.local} · Sold by {selected.vendor}
                  </p>
                  <p>
                    Carefully selected fresh market produce, sourced from your
                    neighborhood vendor. Listed prices are estimates; actual
                    weights may vary.
                  </p>
                  <div className="detail-price">
                    <b>
                      {money(selected.price)} / {selected.unit}
                    </b>
                    <button
                      className="primary"
                      onClick={() => {
                        change(selected.id, 1);
                        setToast(selected.name + " added to your basket.");
                      }}
                    >
                      Add to basket <Plus size={17} />
                    </button>
                  </div>
                </div>
              </>
            )}
            {modal === "how" && (
              <div className="modal-content">
                <span className="section-kicker">
                  FROM OUR MARKET TO YOUR TABLE
                </span>
                <h2>Your market trip, made easy.</h2>
                {[
                  {
                    icon: ShoppingBasket,
                    title: "Pick your fresh favorites",
                    text: `Shop our local vendors and build a basket of at least ${money(minimum)} in goods.`,
                  },
                  {
                    icon: MapPin,
                    title: "Tell us where to go",
                    text: `Confirm your Davao delivery address. Delivery is ${money((settings?.base_fee_centavos??6000)/100)} + ${money((settings?.per_km_centavos??350)/100)} per road kilometer.`,
                  },
                  {
                    icon: ShieldCheck,
                    title: "Pay with GCash",
                    text: "In the live service, scan the operator’s QR and submit your reference. An administrator verifies payment before shopping.",
                  },
                  {
                    icon: Truck,
                    title: "We shop. You relax.",
                    text: "Your courier handpicks your order, checks any changes with you, and delivers to your doorstep.",
                  },
                ].map((s, i) => (
                  <div className="how-step" key={s.title}>
                    <span>
                      <s.icon size={22} />
                    </span>
                    <div>
                      <h3>
                        {i + 1}. {s.title}
                      </h3>
                      <p>{s.text}</p>
                    </div>
                  </div>
                ))}
                <button
                  className="primary full"
                  onClick={() => {
                    setModal(null);
                    setView("shop");
                  }}
                >
                  Let’s shop <ArrowRight size={16} />
                </button>
              </div>
            )}
            {modal === "location" && (
              <div className="modal-content">
                <span className="modal-symbol">
                  <MapPin size={30} />
                </span>
                <h2>A little closer to fresh.</h2>
                <p>
                  We’re starting right here in Davao City, shopping from
                  Bankerohan Public Market.
                </p>
                <label className="field-label">
                  Your delivery address
                  <input
                    placeholder="House no., street, barangay, Davao City"
                    value={address}
                    onChange={(e) => setAddress(e.target.value)}
                  />
                </label>
                <div className="notice">
                  <Info size={18} />
                  <span>
                    Service coverage and road distance must be verified at
                    checkout. Your selected address will be used for the checkout quote.
                  </span>
                </div>
                <button
                  className="primary full"
                  disabled={address.trim().length < 10}
                  onClick={() => {
                    setModal(null);
                    setToast("Delivery address saved.");
                  }}
                >
                  Save address <Check size={16} />
                </button>
              </div>
            )}
            {modal === "auth" && (
              <div className="modal-content">
                <span className="section-kicker">YOUR NEIGHBORHOOD MARKET</span>
                <h2>
                  {authMode === "login"
                    ? "Welcome back, suki."
                    : "Become a suki."}
                </h2>
                <p>Sign in to manage your orders and talk to your courier.</p>
                <form onSubmit={authenticate}>
                  {authMode === "register" && (
                    <>
                      <label className="field-label">
                        Full name
                        <input
                          name="name"
                          required
                          minLength={2}
                          maxLength={100}
                          autoComplete="name"
                        />
                      </label>
                      <label className="field-label">
                        Mobile number
                        <input
                          name="phone"
                          required
                          minLength={7}
                          maxLength={30}
                          autoComplete="tel"
                        />
                      </label>
                      <label className="field-label">
                        Account type
                        <select name="role">
                          <option value="customer">Customer</option>
                          <option value="vendor">
                            Vendor · requires approval
                          </option>
                        </select>
                      </label>
                      <label className="field-label">
                        Shop name (vendors only)
                        <input name="shop_name" maxLength={120} />
                      </label>
                    </>
                  )}
                  <label className="field-label">
                    Email address
                    <input
                      name="email"
                      type="email"
                      required
                      autoComplete="email"
                      maxLength={190}
                    />
                  </label>
                  <label className="field-label">
                    Password
                    <input
                      name="password"
                      type="password"
                      required
                      minLength={authMode === "register" ? 12 : 1}
                      maxLength={128}
                      autoComplete={
                        authMode === "register"
                          ? "new-password"
                          : "current-password"
                      }
                    />
                  </label>
                  {authError && (
                    <div className="notice" role="alert">
                      {authError}
                    </div>
                  )}
                  <button className="primary full" disabled={busy}>
                    {busy
                      ? "Please wait…"
                      : authMode === "login"
                        ? "Sign in"
                        : "Create account"}
                    <ArrowRight size={16} />
                  </button>
                </form>
                <button
                  className="text-link"
                  onClick={() => {
                    setAuthMode(authMode === "login" ? "register" : "login");
                    setAuthError("");
                  }}
                >
                  {authMode === "login"
                    ? "New here? Create an account"
                    : "Already a suki? Sign in"}
                </button>
              </div>
            )}
            {modal === "checkout" && (
              <div className="modal-content">
                <span className="section-kicker">ONE STEP CLOSER TO FRESH</span>
                <h2>Review your basket</h2>
                <p>
                  Fresh market prices are estimates. Any changes need your
                  approval.
                </p>
                <form onSubmit={placeOrder}>
                  <div className="checkout-lines">
                    {items.map((p) => (
                      <div key={p.id}>
                        <span>
                          {p.name}{" "}
                          <small>
                            × {cart[p.id]} {p.unit}
                          </small>
                        </span>
                        <b>{money(p.price * cart[p.id])}</b>
                      </div>
                    ))}
                  </div>
                  <label className="field-label">
                    Delivery address
                    <input
                      required
                      minLength={10}
                      maxLength={500}
                      placeholder="House no., street, barangay, Davao City"
                      value={address}
                      onChange={(e) => setAddress(e.target.value)}
                    />
                  </label>
                  <p className="coordinate-help">
                    Confirm your delivery coordinates from Google Maps, or use
                    your device location.
                  </p>
                  <div className="coordinate-fields">
                    <label className="field-label">
                      Latitude
                      <input
                        required
                        type="number"
                        step="any"
                        min={-90}
                        max={90}
                        value={lat}
                        onChange={(e) => setLat(e.target.value)}
                      />
                    </label>
                    <label className="field-label">
                      Longitude
                      <input
                        required
                        type="number"
                        step="any"
                        min={-180}
                        max={180}
                        value={lng}
                        onChange={(e) => setLng(e.target.value)}
                      />
                    </label>
                  </div>
                  <button
                    type="button"
                    className="text-link"
                    onClick={() =>
                      navigator.geolocation?.getCurrentPosition(
                        (p) => {
                          setLat(String(p.coords.latitude));
                          setLng(String(p.coords.longitude));
                        },
                        () =>
                          setApiError(
                            "Location permission unavailable. Enter your coordinates from Google Maps.",
                          ),
                      )
                    }
                  >
                    <MapPin size={14} /> Use my current location
                  </button>
                  <button
                    type="button"
                    className="primary full"
                    disabled={
                      busy || !lat || !lng || address.trim().length < 10
                    }
                    onClick={requestQuote}
                  >
                    {busy
                      ? "Calculating…"
                      : "Verify road distance & delivery fee"}
                  </button>
                  {quote && (
                    <div className="delivery-breakdown">
                      <div>
                        <span>Goods subtotal</span>
                        <b>{money(subtotal)}</b>
                      </div>
                      <div>
                        <span>One-way road distance</span>
                        <span>
                          {(quote.distance_meters / 1000).toFixed(2)} km
                        </span>
                      </div>
                      <div>
                        <span>Base fee</span>
                        <span>{money(quote.base_fee_centavos / 100)}</span>
                      </div>
                      <div>
                        <span>Distance charge</span>
                        <span>
                          {money(
                            (quote.delivery_fee_centavos -
                              quote.base_fee_centavos) /
                              100,
                          )}
                        </span>
                      </div>
                      <div>
                        <b>Delivery fee</b>
                        <b>{money(quote.delivery_fee_centavos / 100)}</b>
                      </div>
                      <div>
                        <b>Estimated total</b>
                        <b>
                          {money(subtotal + quote.delivery_fee_centavos / 100)}
                        </b>
                      </div>
                    </div>
                  )}
                  {apiError && (
                    <div className="notice" role="alert">
                      {apiError}
                    </div>
                  )}
                  <div className="notice">
                    <ShieldCheck size={20} />
                    <span>
                      We’ll give you the configured GCash QR after your order is
                      created. Shopping begins only after your payment is
                      manually verified.
                    </span>
                  </div>
                  <button
                    className="primary full"
                    type="submit"
                    disabled={busy || !quote || !eligible}
                  >
                    Place order <ArrowRight size={17} />
                  </button>
                </form>
              </div>
            )}
            {modal === "confirmed" && activeOrder && (
              <div className="modal-content confirmation">
                <span className="success-icon">
                  <Check size={32} />
                </span>
                <span className="section-kicker">A FRESH START</span>
                <h2>Your order is placed.</h2>
                <p>
                  Order <b>{activeOrder.id.slice(0, 8)}</b> is in your orders.
                  <br />
                  No payment has been taken.
                </p>
                <div className="notice">
                  <Info size={20} />
                  <span>
                    Open your order to scan the operator’s GCash QR and submit
                    your payment reference. Your payment will be reviewed before
                    shopping starts.
                  </span>
                </div>
                <button
                  className="primary full"
                  onClick={() => {
                    setModal(null);
                    setView("orders");
                  }}
                >
                  View my orders <ArrowRight size={16} />
                </button>
                <button
                  className="text-link"
                  onClick={() => {
                    setModal(null);
                    setView("shop");
                  }}
                >
                  Keep exploring the market
                </button>
              </div>
            )}
            {modal === "order" && activeOrder && (
              <div className="modal-content">
                <span className="section-kicker">YOUR FRESH FINDS</span>
                <h2>Order {activeOrder.id.slice(0, 8)}</h2>
                <span className="order-status">{activeOrder.status}</span>
                <p>
                  <MapPin size={15} /> {activeOrder.address}
                </p>
                <div className="checkout-lines">
                  {activeOrder.items.map((p, i) => (
                    <div key={i}>
                      <span>
                        {p.name} × {p.quantity} {p.unit}
                      </span>
                      <b>{money(p.price * p.quantity)}</b>
                    </div>
                  ))}
                  <div>
                    <b>Estimated goods subtotal</b>
                    <b>{money(activeOrder.subtotal)}</b>
                  </div>
                  <div>
                    <span>Delivery fee</span>
                    <b>
                      {money(
                        (activeOrder.raw?.order.delivery_fee_centavos || 0) /
                          100,
                      )}
                    </b>
                  </div>
                  <div>
                    <b>Estimated order total</b>
                    <b>
                      {money(
                        (activeOrder.raw?.order.estimated_total_centavos || 0) /
                          100,
                      )}
                    </b>
                  </div>
                </div>
                {activeOrder.raw?.order.final_goods_centavos != null && (
                  <div className="notice">
                    Final reconciled goods:{" "}
                    {money(activeOrder.raw.order.final_goods_centavos / 100)}
                  </div>
                )}
                {activeOrder.raw?.adjustments
                  .filter((a) => a[3] === "pending")
                  .map((a) => (
                    <div className="notice" key={a[0]}>
                      <div>
                        <b>Spending approval requested: {money(a[1] / 100)}</b>
                        <p>{a[2]}</p>
                        {user?.role === "customer" && (
                          <div>
                            <button
                              className="primary"
                              onClick={async () => {
                                try {
                                  await api(
                                    "/orders/" +
                                      activeOrder.id +
                                      "/adjustments/" +
                                      a[0],
                                    "POST",
                                    { approved: true },
                                  );
                                  await refreshOrder(activeOrder.id);
                                } catch (e) {
                                  setToast((e as Error).message);
                                }
                              }}
                            >
                              Approve
                            </button>
                            <button
                              onClick={async () => {
                                try {
                                  await api(
                                    "/orders/" +
                                      activeOrder.id +
                                      "/adjustments/" +
                                      a[0],
                                    "POST",
                                    { approved: false },
                                  );
                                  await refreshOrder(activeOrder.id);
                                } catch (e) {
                                  setToast((e as Error).message);
                                }
                              }}
                            >
                              Decline
                            </button>
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                {activeOrder.raw?.substitutions
                  .filter((a) => a[5] === "pending")
                  .map((a) => (
                    <div className="notice" key={a[0]}>
                      <div>
                        <b>
                          Substitution: {a[2]} · {money(a[4] / 100)} per unit
                        </b>
                        {user?.role === "customer" && (
                          <>
                            <button
                              className="primary"
                              onClick={async () => {
                                try {
                                  await api(
                                    "/orders/" +
                                      activeOrder.id +
                                      "/substitutions/" +
                                      a[0],
                                    "POST",
                                    { approved: true },
                                  );
                                  await refreshOrder(activeOrder.id);
                                } catch (e) {
                                  setToast((e as Error).message);
                                }
                              }}
                            >
                              Approve
                            </button>
                            <button
                              onClick={async () => {
                                try {
                                  await api(
                                    "/orders/" +
                                      activeOrder.id +
                                      "/substitutions/" +
                                      a[0],
                                    "POST",
                                    { approved: false },
                                  );
                                  await refreshOrder(activeOrder.id);
                                } catch (e) {
                                  setToast((e as Error).message);
                                }
                              }}
                            >
                              Decline
                            </button>
                          </>
                        )}
                      </div>
                    </div>
                  ))}
                {user?.role === "customer" &&
                  ["awaiting_payment", "price_adjustment"].includes(
                    activeOrder.raw?.order.status || "",
                  ) && (
                    <div className="payment-section">
                      <h3>Pay with GCash</h3>
                      {settings?.gcash_qr_url ? (
                        <img
                          className="payment-qr"
                          src={settings.gcash_qr_url}
                          alt="Operator’s GCash payment QR"
                        />
                      ) : (
                        <div className="notice">
                          The operator has not configured the payment QR.
                        </div>
                      )}
                      <p>
                        Confirm the recipient in GCash. Outstanding amount:{" "}
                        <b>
                          {money(
                            ((activeOrder.raw?.order
                              .authorized_goods_centavos || 0) +
                              (activeOrder.raw?.order.delivery_fee_centavos ||
                                0) -
                              (activeOrder.raw?.payments
                                .filter((p) => p.status === "paid")
                                .reduce(
                                  (sum, p) =>
                                    sum + (p.received_amount_centavos || 0),
                                  0,
                                ) || 0)) /
                              100,
                          )}
                        </b>
                        . Never share your PIN.
                      </p>
                      <label className="field-label">
                        Transaction reference
                        <input
                          value={reference}
                          minLength={6}
                          maxLength={100}
                          onChange={(e) => setReference(e.target.value)}
                        />
                      </label>
                      <button
                        className="primary full"
                        disabled={busy || reference.trim().length < 6}
                        onClick={submitPayment}
                      >
                        Submit for verification
                      </button>
                    </div>
                  )}
                {activeOrder.raw?.payments.map((p) => (
                  <div className="notice" key={p.id}>
                    <ShieldCheck size={17} />
                    <span>
                      Payment {p.reference}: {p.status.replaceAll("_", " ")}
                    </span>
                  </div>
                ))}
                {activeOrder.raw?.refunds.map((r) => (
                  <div className="notice" key={r[0]}>
                    Refund {money(r[1] / 100)} · {r[2]}
                  </div>
                ))}
                <button className="primary full" onClick={openChat}>
                  <MessageCircle size={17} /> Chat about this order
                </button>
                <button
                  className="text-link"
                  onClick={() =>
                    refreshOrder(activeOrder.id).catch((e) =>
                      setToast(e.message),
                    )
                  }
                >
                  Refresh order status
                </button>
                {user?.role === "customer" &&
                  ["awaiting_payment", "payment_verification", "paid"].includes(
                    activeOrder.raw?.order.status || "",
                  ) && (
                    <button
                      className="danger-link"
                      onClick={async () => {
                        try {
                          await api(
                            "/orders/" + activeOrder.id + "/status",
                            "PATCH",
                            { status: "canceled" },
                          );
                          await refreshOrder(activeOrder.id);
                          setToast(
                            "Order canceled. Any verified payment will have a refund recorded.",
                          );
                        } catch (e) {
                          setToast((e as Error).message);
                        }
                      }}
                    >
                      <Trash2 size={14} /> Cancel order
                    </button>
                  )}
              </div>
            )}
            {modal === "chat" && (
              <div className="modal-content">
                <span className="section-kicker">
                  ORDER {activeOrder?.id} · ORDER CHAT
                </span>
                <h2>A little help from your suki.</h2>
                <p>
                  Messages are saved securely for you and your assigned courier.
                </p>
                <div className="chat-messages">
                  {messages.map((m, i) => (
                    <div
                      key={i}
                      className={`chat-bubble ${m.mine ? "mine" : ""}`}
                    >
                      {m.text}
                      <small>{m.mine ? "You" : m.name}</small>
                    </div>
                  ))}
                </div>
                <form className="chat-input" onSubmit={sendChat}>
                  <input
                    aria-label="Your message"
                    placeholder="Add a delivery instruction..."
                    value={message}
                    maxLength={2000}
                    onChange={(e) => setMessage(e.target.value)}
                  />
                  <button
                    className="primary"
                    aria-label="Send message"
                    disabled={busy || !message.trim()}
                  >
                    <Send size={18} />
                  </button>
                </form>
              </div>
            )}
            {modal === "vendors" && (
              <div className="modal-content">
                <span className="section-kicker">THE HEART OF OUR MARKET</span>
                <h2>Meet your neighborhood suki.</h2>
                <p>Get to know the people behind your fresh market finds.</p>
                {Array.from(new Set(products.map((p) => p.vendor))).map(
                  (v, i) => (
                    <button
                      className="vendor-row"
                      key={v}
                      onClick={() => {
                        setSearch(v);
                        setCategory("All products");
                        setOnlyFavorites(false);
                        setView("shop");
                        setModal(null);
                        setTimeout(
                          () =>
                            document
                              .getElementById("catalog")
                              ?.scrollIntoView({ behavior: "smooth" }),
                          50,
                        );
                      }}
                    >
                      <span>
                        <Store size={21} />
                      </span>
                      <div>
                        <b>{v}</b>
                        <small>Bankerohan Public Market</small>
                      </div>
                      <ArrowRight size={16} />
                    </button>
                  ),
                )}
              </div>
            )}
            {modal === "vendor" && (
              <div className="modal-content">
                <span className="section-kicker">VENDOR WORKSPACE</span>
                <h2>Fresh picks, kept up to date.</h2>
                <p>
                  Manage the availability of your own listings. Updates are
                  saved to the market catalog.
                </p>
                {products
                  .filter((p) => p.vendor_id === user?.id)
                  .map((p) => (
                    <div className="vendor-product" key={p.id}>
                      <img src={p.image} alt={p.name} />
                      <div>
                        <b>{p.name}</b>
                        <small>
                          {money(p.price)} / {p.unit}
                        </small>
                      </div>
                      <button
                        className={`stock-toggle ${p.stock ? "on" : ""}`}
                        disabled={busy}
                        onClick={() => toggleAvailability(p)}
                      >
                        {p.stock ? "In stock" : "Unavailable"}
                      </button>
                    </div>
                  ))}
                <div className="notice">
                  <ShieldCheck size={18} />
                  <span>
                    Your account can update only its own listings. Vendor
                    actions are recorded in the audit log.
                  </span>
                </div>
              </div>
            )}
            {modal === "support" && (
              <div className="modal-content">
                <span className="modal-symbol">
                  <MessageCircle size={28} />
                </span>
                <h2>Your suki is here to help.</h2>
                <p>
                  Have a question about an order? Open it in My orders to see
                  the order details and chat.
                </p>
                <div className="notice">
                  <Info size={18} />
                  <span>
                    Use your order chat for delivery questions. Shopping is
                    authorized only after the operator verifies your GCash
                    payment.
                  </span>
                </div>
                <button
                  className="primary full"
                  onClick={() => {
                    setView("orders");
                    setModal(null);
                  }}
                >
                  Go to my orders <ArrowRight size={16} />
                </button>
              </div>
            )}
            {modal === "about" && (
              <div className="modal-content">
                <span className="section-kicker">BUILT FOR DAVAO</span>
                <h2>Your neighborhood, delivered.</h2>
                <p>
                  Suki brings Bankerohan’s fresh finds closer to home and helps
                  local market vendors reach more neighbors.
                </p>
                <div className="notice">
                  <Info size={20} />
                  <span>
                    Suki securely stores accounts, orders, payment verification
                    records and conversations in its backend. Google Maps and
                    operator payment details must be configured before checkout
                    becomes available.
                  </span>
                </div>
              </div>
            )}
          </section>
        </div>
      )}
    </>
  );
}
