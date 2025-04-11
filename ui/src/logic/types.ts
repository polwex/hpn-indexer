export type AsyncRes<T> = Promise<Result<T>>;
export type Result<T> = { ok: T } | { error: string };

export type AllProviders = Record<Category, Array<Provider>>;
export type Category = string;
export type Provider = {
  category: Category;
  site: string;
  description: string;
  name: string;
  providerName: string;
  providerId: string;
  price: string;
  // db data
  created?: number;
  id?: number;
};
export type ProviderJson = {
  category: Category;
  site: string;
  description: string;
  name: string;
  provider_name: string;
  provider_id: string;
  price: string;
  // db data
  created?: number;
  id?: number;
};
