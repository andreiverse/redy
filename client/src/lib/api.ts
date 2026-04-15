import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import type { paths } from "./api/v1";

const backendApi = import.meta.env.VITE_BACKEND_API ?? "localhost:8080";
const baseUrl = backendApi.startsWith('http') ? backendApi : `http://${backendApi}`;

const customFetch = (input: RequestInfo | URL, init?: RequestInit) => {
  return fetch(input, {
    ...init,
    credentials: 'include',
  });
};

const fetchClient = createFetchClient<paths>({
  baseUrl,
  fetch: customFetch,
});

export const $api = createClient(fetchClient);
