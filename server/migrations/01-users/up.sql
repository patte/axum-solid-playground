create table users (
  id blob check(length(id) = 16) primary key,
  username text not null
);

create table authenticators (
  passkey jsonb not null primary key,
  user_id blob check(length(user_id) = 16) not null references users(id)
);
