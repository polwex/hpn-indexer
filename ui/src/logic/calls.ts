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
export async function searchDB(query: string): AsyncRes<Provider[]> {
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
    );
    if ("error" in provider) continue;
    data.push(provider.ok);
  }
  return data;
}
function decodeProvider(
  category: string,
  name: string,
  n: string,
  d: string,
  s: string,
): Result<Provider> {
  console.log({ category, name, n, d, s });
  if (!n || !d || !s) return { error: "no data" };
  const namer = decodeDatakey(n);
  if ("error" in namer) return { error: "decoding error" };
  const descriptionr = decodeDatakey(d);
  if ("error" in descriptionr) return { error: "decoding error" };
  const siter = decodeDatakey(s);
  if ("error" in siter) return { error: "decoding error" };
  const provider: Provider = {
    category,
    name,
    providerName: namer.ok,
    description: descriptionr.ok,
    site: siter.ok,
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
