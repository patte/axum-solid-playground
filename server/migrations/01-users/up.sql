create table users (
  id BLOB CHECK(length(id) = 16) primary key,
  username text not null
);

create table authenticators (
  passkey text not null primary key,
  user_id BLOB CHECK(length(user_id) = 16) not null references users(id)
);
