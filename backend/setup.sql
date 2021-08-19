CREATE TABLE users (
  id UUID PRIMARY KEY,
  username varchar not null,
  hashed_password varchar not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone not null
);

create unique index users_username on users(username);

CREATE TABLE auth_tokens (
  id UUID PRIMARY KEY,
  user_id uuid not null references users (id),
  token varchar not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone not null
);

create unique index auth_tokens_token on auth_tokens(token);

CREATE TABLE events (
  id UUID PRIMARY KEY,
  user_id uuid not null references users (id),
  content text not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone not null
);

CREATE TABLE follows (
  id UUID PRIMARY KEY,
  follower_id uuid not null references users (id),
  followed_id uuid not null references users (id),
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone not null
);