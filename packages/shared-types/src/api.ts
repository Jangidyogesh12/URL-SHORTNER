/**
 * Successful API responses are wrapped in a `data` envelope.
 *
 * This mirrors `ApiSuccessResponse<T>` in `apps/api/src/utils/api_response.rs`.
 */
export interface ApiSuccessResponse<T> {
  data: T;
}

/**
 * Error responses returned by the API.
 *
 * This mirrors `ApiErrorResponse` in `apps/api/src/utils/api_response.rs`.
 */
export interface ApiErrorResponse {
  message: string | null;
  code: number;
}
