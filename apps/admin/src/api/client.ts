export const API_BASE_URL = (
  import.meta.env.VITE_API_BASE_URL ?? "http://192.168.1.222:8000/api"
).replace(/\/$/, "");

export function assetUrl(path: string) {
  const origin = API_BASE_URL.replace(/\/api$/, "");
  return `${origin}${path.startsWith("/") ? path : `/${path}`}`;
}

export async function apiRequest<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });

  if (!response.ok) {
    const payload = (await response.json().catch(() => null)) as {
      error?: string;
    } | null;
    throw new Error(payload?.error ?? `请求失败（${response.status}）`);
  }

  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}
