create table users (
  id blob check(length(id) = 16) primary key,
  username text not null,
  created_at text not null default (strftime('%Y-%m-%dT%H:%M:%SZ'))
);

create table authenticators (
  passkey jsonb not null primary key,
  user_id blob check(length(user_id) = 16) not null references users(id),
  created_at text not null default (strftime('%Y-%m-%dT%H:%M:%SZ')),
  user_agent_short text
);
