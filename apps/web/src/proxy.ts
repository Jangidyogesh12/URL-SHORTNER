import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

const SHORT_CODE_PATTERN = /^[A-Za-z0-9]{8}$/;
const REDIRECTABLE_STATUSES = new Set([301, 302, 303, 307, 308]);

function apiBaseUrl(): string {
  const configured = process.env.API_URL?.trim().replace(/\/+$/, "");
  if (configured) {
    return configured;
  }

  return "http://localhost:8080";
}

export async function proxy(request: NextRequest) {
  const pathname = request.nextUrl.pathname;

  if (pathname.length !== 9) {
    return NextResponse.next();
  }

  const shortCode = pathname.slice(1);
  if (!SHORT_CODE_PATTERN.test(shortCode)) {
    return NextResponse.next();
  }

  let upstream: Response;
  try {
    upstream = await fetch(`${apiBaseUrl()}/${shortCode}`, {
      method: request.method,
      redirect: "manual",
    });
  } catch {
    return NextResponse.next();
  }

  if (upstream.status === 410) {
    return NextResponse.redirect(new URL("/expired", request.url));
  }

  if (upstream.status === 404) {
    return NextResponse.redirect(new URL("/missing", request.url));
  }

  const location = upstream.headers.get("location");
  if (!location || !REDIRECTABLE_STATUSES.has(upstream.status)) {
    return NextResponse.next();
  }

  let destination: URL;
  try {
    destination = new URL(location);
  } catch {
    return NextResponse.next();
  }

  if (destination.protocol !== "http:" && destination.protocol !== "https:") {
    return NextResponse.next();
  }

  return NextResponse.redirect(destination, 307);
}

export const config = {
  matcher: ["/:shortCode"],
};
