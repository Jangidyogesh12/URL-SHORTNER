-- Add up migration script here
create table urls(
    id uuid primary key,
    user_id uuid,
    original_url text not null,
    short_code varchar(8) unique not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    expires_at timestamptz,
    constraint fk_urls_user foreign key (user_id) references users(id) on delete set null
);

create index idx_short_code on urls(short_code);
