alter table users
  add column created_at timestamp with timezone not null default current_timestamp
;

alter table authenticators
  add column created_at timestamp with timezone not null default current_timestamp
;
alter table authenticators
  add column user_agent_short text
; 