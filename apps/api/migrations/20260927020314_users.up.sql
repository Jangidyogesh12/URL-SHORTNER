-- Add up migration script here
create table users (
    id uuid primary key,
    name varchar(50) unique not null,
    email varchar(255) unique not null,
    phone varchar(20) unique,
    password varchar(255) not null,
    is_active boolean not null default true,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index idx_users_email on users(email);
create index idx_users_phone on users(phone);
create index idx_users_username on users(name);
