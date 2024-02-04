create table users (
  id uuid not null primary key,
  username text not null
);

create table authenticators (
  passkey text not null primary key,
  user_id uuid not null references users(id)
);

insert into users (id, username) values ('5f34a917-7f94-4953-8a8a-b0c4f1ea2017', 'admin');