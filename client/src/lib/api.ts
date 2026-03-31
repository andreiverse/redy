import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import type { paths } from "./api/v1";

const fetchClient = createFetchClient<paths>({
  baseUrl: process.env.API_URL ?? "http://localhost:8080",
});
export const $api = createClient(fetchClient);