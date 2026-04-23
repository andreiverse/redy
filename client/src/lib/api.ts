import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import type { paths } from "./api/v1";

const isServer = typeof window === "undefined";

export const baseUrl = isServer
  ? // SSR → must NOT use window
    (import.meta.env.VITE_BACKEND_API
      ? import.meta.env.VITE_BACKEND_API.startsWith("http")
        ? import.meta.env.VITE_BACKEND_API
        : `http://${import.meta.env.VITE_BACKEND_API}`
      : "http://localhost:8080")
  : // Browser → safe to use window
    (() => {
      const { protocol, hostname } = window.location;
      return import.meta.env.DEV
        ? `http://${import.meta.env.VITE_BACKEND_API ?? "localhost:8080"}`
        : `${protocol}//api-${hostname}`;
    })();

const customFetch = (input: RequestInfo | URL, init?: RequestInit) => {
  return fetch(input, {
    ...init,
    credentials: "include",
  });
};

const fetchClient = createFetchClient<paths>({
  baseUrl,
  fetch: customFetch,
});

export const $api = createClient(fetchClient);