-- Add up migration script here
create table urls(
    id uuid primary key,
    user_id uuid,
    original_url text not null,
    short_code varchar(8) not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    expires_at timestamptz not null,
    constraint fk_urls_user foreign key (user_id) reference users(id) on delete set null
);
create index idx_original_url on urls(original_url);
