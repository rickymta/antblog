#!/usr/bin/env bash
# Thử vài câu lệnh PostgreSQL trên sqlite3 CLI để thấy chỗ khác biệt. Chạy: bash scripts/sqlite_contrast.sh
run() { echo "\$ $1"; sqlite3 :memory: "$1" 2>&1 | head -3; }
run "create table postings(entry_id int, amount int); create constraint trigger b after insert on postings deferrable initially deferred for each row execute function f();"
run "create table t(id integer primary key, k text unique); insert into t(k) values('a') on conflict(k) do nothing returning id; insert into t(k) values('a') on conflict(k) do nothing returning id; select 'on-conflict-do-nothing chay duoc, dong thu hai khong tra gi';"
run "begin isolation level serializable;"
run "set transaction isolation level serializable;"
run "create table p(a int); create trigger tr before update on p begin select raise(abort, 'append-only'); end; insert into p values(1); update p set a=2;"
run "create table x(id bigint generated always as identity primary key);"
run "create schema lab_x;"
run "create table a(v timestamptz not null default now());"
