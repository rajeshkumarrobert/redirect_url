-- All URL collection
Create table urls (
  id INTEGER primary key generated always as identity,
  name text not null UNIQUE,
  value text not null UNIQUE,
  is_active boolean not null default true,
  created_on timestamp not null default now()
);
