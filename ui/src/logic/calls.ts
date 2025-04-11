import type {
  AllProviders,
  AsyncRes,
  Provider,
  ProviderJson,
  Result,
} from "./types";

export const API_PATH = "/indexer:hpn:sortugdev.os/api";

export async function fetchState(): AsyncRes<AllProviders> {
  try {
    const response = await fetch(API_PATH + "/state");
    const j = await response.json();
    const data = parseFromState(j);
    console.log(data);
    return { ok: data };
  } catch (e) {
    return { error: `${e}` };
  }
}

export async function fetchAll(): AsyncRes<Provider[]> {
  try {
    const response = await fetch(API_PATH + "/all");
    const j = await response.json();
    return { ok: j };
  } catch (e) {
    return { error: `${e}` };
  }
}

function parseFromState(j: any): AllProviders {
  let data: AllProviders = {};
  for (let hash in j) {
    const p = j[hash];
    const cat = data[p.category] || [];
    // if (p.name === "deezer") console.log("decoding", p);
    // TODO where is the current value? start or end of array?
    const provider = decodeProvider(
      p.category,
      p.name,
      p.facts["~provider-name"],
      p.facts["~description"],
      p.facts["~site"],
      p.facts["~provider-id"],
      p.facts["~price"],
    );
    if ("error" in provider) continue;
    cat.push(provider.ok);
    data[p.category] = cat;
  }
  return data;
}

export async function fetchCategory(cat: string): AsyncRes<Provider[]> {
  try {
    const response = await fetch(API_PATH + "/cat?cat=" + cat);
    const j = await response.json();
    return { ok: j };
  } catch (e) {
    return { error: `${e}` };
  }
}
export async function searchDB(query: string): AsyncRes<ProviderJson[]> {
  try {
    const response = await fetch(API_PATH + "/search?q=" + query);
    const j = await response.json();
    return { ok: j };
  } catch (e) {
    return { error: `${e}` };
  }
}

function parseSqlite(ps: ProviderJson[]): Provider[] {
  let data: Provider[] = [];
  for (let p of ps) {
    const provider = decodeProvider(
      p.category,
      p.name,
      p.provider_name,
      p.description,
      p.site,
      p.provider_id,
      p.price,
    );
    if ("error" in provider) continue;
    data.push(provider.ok);
  }
  return data;
}
function decodeProvider(
  category: string,
  name: string,
  providerName_hex: string,
  description_hex: string,
  site_hex: string,
  providerId_hex: string,
  price_hex: string,
): Result<Provider> {
  if (
    !providerName_hex ||
    !description_hex ||
    !site_hex ||
    !providerId_hex ||
    !price_hex
  )
    return { error: "no data" };
  const namer = decodeDatakey(providerName_hex);
  if ("error" in namer) return { error: "decoding error" };
  const descriptionr = decodeDatakey(description_hex);
  if ("error" in descriptionr) return { error: "decoding error" };
  const siter = decodeDatakey(site_hex);
  if ("error" in siter) return { error: "decoding error" };
  const idr = decodeDatakey(providerId_hex);
  if ("error" in idr) return { error: "decoding error" };
  const pricer = decodeDatakey(price_hex);
  if ("error" in pricer) return { error: "decoding error" };
  const provider: Provider = {
    category,
    name,
    providerName: namer.ok,
    description: descriptionr.ok,
    site: siter.ok,
    providerId: idr.ok,
    price: pricer.ok,
  };

  return { ok: provider };
}
function decodeDatakey(hexString: string): Result<string> {
  try {
    const cleanHex = hexString.startsWith("0x")
      ? hexString.slice(2)
      : hexString;
    const hexPairs = cleanHex.match(/.{1,2}/g);
    if (!hexPairs) return { error: "datakey decoding failed" };
    const bytes = new Uint8Array(hexPairs.map((byte) => parseInt(byte, 16)));
    const decoded = new TextDecoder("utf-8").decode(bytes);
    if (/^[\x20-\x7E]*$/.test(decoded)) {
      return { ok: decoded };
    } else return { error: "datakey decoding failed" };
  } catch (error) {
    return { error: `${error}` };
  }
}
