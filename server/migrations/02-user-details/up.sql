alter table users
  add column created_at text not null default (strftime('%Y-%m-%dT%H:%M:%SZ'))
;

alter table authenticators
  add column created_at text not null default (strftime('%Y-%m-%dT%H:%M:%SZ'))
;
alter table authenticators
  add column user_agent_short text
; 