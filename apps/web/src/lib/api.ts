import type { HelloResponse, User } from "shared-types";

export async function getHello(): Promise<HelloResponse> {
  const res = await fetch("/api/hello");
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

export async function getUsers(): Promise<User[]> {
  const res = await fetch("/api/users");
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}
