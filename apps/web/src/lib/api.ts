import type {
  ApiErrorResponse,
  ApiSuccessResponse,
  TokenReadDto,
  UrlCreateDto,
  UrlEditDto,
  UrlQueryDto,
  UrlReadDto,
  UserLoginDto,
  UserReadDto,
  UserRegisterDto,
} from "shared-types";

export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

const API_BASE_PATH = "/api";

interface ApiRequestOptions extends Omit<RequestInit, "body" | "headers"> {
  token?: string;
  headers?: HeadersInit;
  body?: unknown;
}

function isApiErrorPayload(value: unknown): value is ApiErrorResponse {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as {
    message?: unknown;
    code?: unknown;
  };

  return (
    (typeof candidate.message === "string" || candidate.message === null) &&
    typeof candidate.code === "number"
  );
}

async function parseApiError(response: Response): Promise<ApiError> {
  let message = `Request failed with status ${response.status}`;

  try {
    const payload: unknown = await response.json();
    if (isApiErrorPayload(payload) && payload.message) {
      message = payload.message;
    }
  } catch {
    // Keep the fallback message when the body is empty or is not JSON.
  }

  return new ApiError(response.status, message);
}

async function apiRequest<T>(
  path: string,
  options: ApiRequestOptions = {},
): Promise<T> {
  const { token, headers: customHeaders, body, ...init } = options;
  const headers = new Headers(customHeaders);

  if (body !== undefined && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  const response = await fetch(`${API_BASE_PATH}${path}`, {
    ...init,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  if (!response.ok) {
    throw await parseApiError(response);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  const payload = (await response.json()) as ApiSuccessResponse<T>;
  return payload.data;
}

export function registerUser(payload: UserRegisterDto): Promise<UserReadDto> {
  return apiRequest<UserReadDto>("/register", {
    method: "POST",
    body: payload,
  });
}

export function loginUser(payload: UserLoginDto): Promise<TokenReadDto> {
  return apiRequest<TokenReadDto>("/auth", {
    method: "POST",
    body: payload,
  });
}

export function listUrls(token: string): Promise<UrlReadDto[]> {
  return apiRequest<UrlReadDto[]>("/urls", { token });
}

export function lookupUrl(
  input: UrlQueryDto,
  token: string,
): Promise<UrlReadDto> {
  const query = new URLSearchParams({ long_url: input.long_url }).toString();
  return apiRequest<UrlReadDto>(`/url?${query}`, { token });
}

export function createUrl(
  input: UrlCreateDto,
  token: string,
): Promise<UrlReadDto> {
  return apiRequest<UrlReadDto>("/create_url", {
    method: "POST",
    token,
    body: input,
  });
}

export function updateUrl(
  input: UrlEditDto,
  token: string,
): Promise<UrlReadDto> {
  return apiRequest<UrlReadDto>("/edit_url", {
    method: "PUT",
    token,
    body: input,
  });
}

export async function deleteUrl(
  input: UrlQueryDto,
  token: string,
): Promise<void> {
  const query = new URLSearchParams({ long_url: input.long_url }).toString();
  await apiRequest<void>(`/delete_url?${query}`, {
    method: "DELETE",
    token,
  });
}

export function isUnauthorized(error: unknown): boolean {
  return error instanceof ApiError && error.status === 401;
}

export function getErrorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  return fallback;
}
